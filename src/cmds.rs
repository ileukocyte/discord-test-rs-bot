use lazy_static::lazy_static;

use serenity::{
    client::Context,
    model::channel::Message,
    async_trait,
};

use std::error::Error;

lazy_static! {
    pub static ref COMMANDS: Vec<Box<dyn Command>> = vec![
        Box::new(dev::ShutdownCommand),
        Box::new(general::HelpCommand),
        Box::new(general::PingCommand),
        Box::new(general::UptimeCommand),
        Box::new(utility::WeatherCommand),
        //Box::new(dev::TestCommand),
    ];
}

pub type CommandUsages<'a> = Vec<Vec<&'a str>>;

#[async_trait]
pub trait Command: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn category(&self) -> CommandCategory;

    fn aliases(&self) -> Vec<&str> {
        Vec::new()
    }

    fn usages(&self) -> CommandUsages {
        Vec::new()
    }

    fn is_developer(&self) -> bool {
        self.category() == CommandCategory::Developer
    }

    async fn invoke(
        &self,
        cx: &Context,
        message: &Message,
        args: &[&str],
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[derive(Debug, PartialEq)]
pub enum CommandCategory {
    Developer,
    General,
    Utility,
}

impl ToString for CommandCategory {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

mod dev {
    use crate::{
        cmds::{Command, CommandCategory},
        utils::discord::{DefaultEmbedReplies, EmbedType},
    };

    use serenity::{
        client::Context,
        model::channel::Message,
        async_trait,
    };

    use std::{
        error::Error,
        process::exit,
    };

    use tokio::time::Duration;

    pub struct ShutdownCommand;
    //pub struct TestCommand;

    #[async_trait]
    impl Command for ShutdownCommand {
        fn name(&self) -> &'static str {
            "shutdown"
        }

        fn description(&self) -> &'static str {
            "Shuts the bot down"
        }

        fn category(&self) -> CommandCategory {
            CommandCategory::Developer
        }

        async fn invoke(
            &self,
            cx: &Context,
            message: &Message,
            _args: &[&str],
        ) -> Result<(), Box<dyn Error + Send + Sync>> {
            const CHECK_MARK: char = '\u{2705}';
            const CROSS_MARK: char = '\u{274E}';

            let msg = message.channel_id.send_default_reply(
                &cx.http,
                "Are you sure?",
                EmbedType::Confirmation,
            ).await.unwrap_or_else(|_| exit(0));

            msg.react(&cx.http, CHECK_MARK).await?;
            msg.react(&cx.http, CROSS_MARK).await?;

            let check_reaction = message.channel_id
                .await_reaction(&cx.shard)
                .message_id(msg.id)
                .author_id(message.author.id)
                .filter(|r| {
                    r.emoji.unicode_eq(&CHECK_MARK.to_string())
                        || r.emoji.unicode_eq(&CROSS_MARK.to_string())
                })
                .timeout(Duration::from_secs(60))
                .await;

            match check_reaction {
                Some(reaction) => {
                    msg.delete(&cx.http).await?;

                    if reaction.as_inner_ref().emoji.unicode_eq(&CHECK_MARK.to_string()) {
                        exit(0)
                    }
                }
                None => {
                    msg.delete(&cx.http).await?;
                }
            }

            Ok(())
        }
    }

    /*#[async_trait]
    impl Command for TestCommand {
        fn name(&self) -> &'static str {
            "test"
        }

        fn description(&self) -> &'static str {
            "N/A"
        }

        fn category(&self) -> CommandCategory {
            CommandCategory::Developer
        }

        async fn invoke(
            &self,
            cx: &Context,
            message: &Message,
            _args: &[&str]
        ) -> Result<(), Box<dyn Error + Send + Sync>> {
            /*use serenity::collector::ComponentInteractionCollectorBuilder;
            use serenity::futures::StreamExt;
            use serenity::model::interactions::message_component::ButtonStyle;
            use serenity::model::prelude::InteractionApplicationCommandCallbackDataFlags;

            let msg = message.channel_id.send_message(&cx.http, |m| {
                m.content("test").components(|c| {
                    c.create_action_row(|r| {
                        r.create_button(|button| {
                            button
                                .style(ButtonStyle::Secondary)
                                .emoji('\u{1F98D}'.into())
                                .label("Test")
                                .custom_id(format!("{}-test", self.name()))
                        })
                    })
                })
            }).await?;

            /*let events: Vec<_> = ComponentInteractionCollectorBuilder::new(&cx)
                .message_id(msg.id)
                .author_id(message.author.id)
                .collect_limit(1)
                .await
                .collect()
                .await;

            if let Some(e) = events.first() {
                e.create_interaction_response(&cx.http, |m| {
                    m.interaction_response_data(|d| {
                        d
                            .content(&e.data.custom_id)
                            .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                    })
                }).await?;
            }*/

            let mut events = ComponentInteractionCollectorBuilder::new(&cx)
                .message_id(msg.id)
                .author_id(message.author.id)
                //.collect_limit(1)
                .await;

            while let Some(e) = events.next().await {
                e.create_interaction_response(&cx.http, |m| {
                    m.interaction_response_data(|d| {
                        d
                            .content(&e.data.custom_id)
                            .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                    })
                }).await?;
            }*/

            message.channel_id.send_default_reply(&cx.http, "test", EmbedType::Success).await?;
            message.channel_id.send_default_reply(&cx.http, "test", EmbedType::Failure).await?;
            message.channel_id.send_default_reply(&cx.http, "test", EmbedType::Confirmation).await?;
            message.channel_id.send_default_reply(&cx.http, "test", EmbedType::Warning).await?;

            Ok(())
        }
    }*/
}

mod general {
    use chrono::{DateTime, Utc};

    use crate::{
        cmds::{Command, CommandCategory, CommandUsages, COMMANDS},
        config,
        utils::{
            discord::{DefaultEmbedReplies, EmbedType},
            time::as_text,
        },
        START_TIME,
    };

    use itertools::Itertools;

    use serenity::{
        client::Context,
        model::channel::Message,
        async_trait,
    };

    use std::{
        error::Error,
        time::SystemTime,
    };

    pub struct HelpCommand;
    pub struct PingCommand;
    pub struct UptimeCommand;

    #[async_trait]
    impl Command for HelpCommand {
        fn name(&self) -> &'static str {
            "help"
        }

        fn description(&self) -> &'static str {
            "Sends a list of the bot's commands or provides help for the specified command"
        }

        fn category(&self) -> CommandCategory {
            CommandCategory::General
        }

        fn usages(&self) -> CommandUsages {
            vec![vec!["command name (optional)"]]
        }

        async fn invoke(
            &self,
            cx: &Context,
            message: &Message,
            args: &[&str],
        ) -> Result<(), Box<dyn Error + Send + Sync>> {
            match args.first().map(|a| a.to_lowercase()) {
                Some(cmd_name) => {
                    let commands: Vec<&Box<dyn Command>> = (*COMMANDS)
                        .iter()
                        .filter(|c| c.name() == cmd_name || c.aliases().contains(&cmd_name.as_str()))
                        .collect();

                    if let Some(cmd) = commands.first() {
                        let mut title = format!("{}{}", config::PREFIX, cmd.name());

                        if cmd.is_developer() {
                            title.push_str(" (developer-only)");
                        }

                        let bot_pfp = cx.http.get_current_user().await?.face();

                        message.channel_id.send_message(&cx.http, |m| {
                            m.embed(|embed| {
                                let mut fields = vec![("Category", cmd.category().to_string(), false)];

                                if !cmd.aliases().is_empty() {
                                    let mut aliases = cmd.aliases();

                                    aliases.sort();

                                    fields.push(("Aliases", aliases.join(", "), false));
                                }

                                if !cmd.usages().is_empty() {
                                    let usages = cmd.usages().iter().map(|u| {
                                        u.iter()
                                            .map(|a| format!("<{}>", a))
                                            .collect::<Vec<String>>()
                                            .join(" ")
                                    }).collect::<Vec<String>>();

                                    let usages = usages.iter()
                                        .map(|u| format!("{}{} {}", config::PREFIX, cmd.name(), u))
                                        .collect::<Vec<String>>()
                                        .join("\n");

                                    fields.push(("Usages", usages, false));
                                }

                                embed
                                    .color(config::SUCCESS_COLOR)
                                    .author(|a| a.name(title).icon_url(bot_pfp))
                                    .description(cmd.description())
                                    .fields(fields)
                            })
                        }).await?;
                    } else {
                        message.channel_id.send_default_reply(
                            &cx.http,
                            "No command has been found by the query!",
                            EmbedType::Failure,
                        ).await?;
                    }
                }
                None => {
                    let bot = cx.http.get_current_user().await?;

                    let mut cmds_grouped: Vec<(CommandCategory, Vec<&Box<dyn Command>>)> = Vec::new();

                    for (key, group) in &COMMANDS.iter().group_by(|c| c.category()) {
                        cmds_grouped.push((key, group.collect()));
                    }

                    cmds_grouped.sort_by_key(|(c, _)| c.to_string());

                    message.channel_id.send_message(&cx.http, |m| {
                        m.embed(|embed| {
                            let fields = cmds_grouped.into_iter().map(|(c, cmds)| {
                                let mut cmds = cmds
                                    .into_iter()
                                    .map(|cmd| cmd.name())
                                    .collect::<Vec<&str>>();

                                cmds.sort();

                                (format!("{} Commands", c.to_string()), cmds.join(", "), false)
                            });

                            embed
                                .author(|a| a.name(format!("{} Help", bot.name)).icon_url(bot.face()))
                                .color(config::SUCCESS_COLOR)
                                .fields(fields)
                        })
                    }).await?;
                }
            }

            Ok(())
        }
    }

    #[async_trait]
    impl Command for PingCommand {
        fn name(&self) -> &'static str {
            "ping"
        }

        fn description(&self) -> &'static str {
            "Sends the bot's current response latency"
        }

        fn category(&self) -> CommandCategory {
            CommandCategory::General
        }

        fn aliases(&self) -> Vec<&str> {
            vec!["latency"]
        }

        async fn invoke(
            &self,
            cx: &Context,
            message: &Message,
            _args: &[&str],
        ) -> Result<(), Box<dyn Error + Send + Sync>> {
            let now = SystemTime::now();
            let mut msg = message.channel_id.say(&cx.http, "*Measuring\u{2026}*").await?;
            let ping = now.elapsed()?.as_millis();

            msg.edit(&cx.http, |e| {
                e.content(String::default()).embed(|embed| {
                    embed
                        .color(config::SUCCESS_COLOR)
                        .author(|a| a.name("Rest Ping"))
                        .description(format!("{} ms", ping))
                })
            }).await?;

            Ok(())
        }
    }

    #[async_trait]
    impl Command for UptimeCommand {
        fn name(&self) -> &'static str {
            "uptime"
        }

        fn description(&self) -> &'static str {
            "Sends the bot's current uptime"
        }

        fn category(&self) -> CommandCategory {
            CommandCategory::General
        }

        async fn invoke(
            &self,
            cx: &Context,
            message: &Message,
            _args: &[&str],
        ) -> Result<(), Box<dyn Error + Send + Sync>> {
            let uptime = SystemTime::now().duration_since(*START_TIME)?;

            message.channel_id.send_message(&cx.http, |m| {
                m.embed(|embed| {
                    let start_time: DateTime<Utc> = (*START_TIME).into();

                    embed
                        .author(|a| a.name("Uptime"))
                        .description(as_text(uptime.as_millis() as i64))
                        .color(config::SUCCESS_COLOR)
                        .footer(|f| f.text("Last Reboot"))
                        .timestamp(&start_time)
                })
            }).await?;

            Ok(())
        }
    }
}

mod utility {
    use chrono::{FixedOffset, NaiveDateTime, TimeZone, Utc};

    use crate::{
        cmds::{Command, CommandCategory, CommandUsages},
        config,
        utils::{
            discord::{DefaultEmbedReplies, EmbedType},
            misc::get_wind_direction,
        },
    };

    use num_format::{Locale, ToFormattedString};

    use openweather_async::{OpenWeather, Units};

    use serenity::{
        client::Context,
        model::channel::Message,
        async_trait,
    };

    use std::error::Error;

    pub struct WeatherCommand;

    #[async_trait]
    impl Command for WeatherCommand {
        fn name(&self) -> &'static str {
            "weather"
        }

        fn description(&self) -> &'static str {
            "Sends the weather in the specified location"
        }

        fn category(&self) -> CommandCategory {
            CommandCategory::Utility
        }

        fn usages(&self) -> CommandUsages {
            vec![vec!["location"]]
        }

        async fn invoke(
            &self,
            cx: &Context,
            message: &Message,
            args: &[&str],
        ) -> Result<(), Box<dyn Error + Send + Sync>> {
            if !args.is_empty() {
                let api_key = config::WEATHER_API_KEY.as_str();
                let openweather_api = OpenWeather::new(api_key, Units::Metric);
                let weather = openweather_api.get_by_city(&args.join(" ")).await
                    .map_err(|_| "No location has been found by the query!")?;
                let bot_pfp = cx.http.get_current_user().await?.face();

                message.channel_id.send_message(&cx.http, |m| {
                    m.embed(|embed| {
                        let url = format!("https://openweathermap.org/city/{}", weather.id);
                        let location = if let Some(ref sys) = weather.sys {
                            format!("{}, {}", weather.name, sys.country)
                        } else {
                            weather.name
                        };

                        let mut fields: Vec<(&str, String, bool)> = Vec::new();

                        if let Some(weather_data) = weather.weather {
                            if let Some(weather_data) = weather_data.first() {
                                let main = &weather_data.main;

                                fields.push(("Condition", main.to_string(), true));
                            }
                        }

                        {
                            let temp_c = weather.main.temp;
                            let temp_f = temp_c * 1.8 + 32.0;
                            let temp_output = {
                                let degree_char = 0xb0 as char;

                                format!(
                                    "{c}{0}C/{f}{0}F",
                                    degree_char,
                                    c = temp_c as usize,
                                    f = temp_f as usize,
                                )
                            };

                            fields.push(("Temperature", temp_output, true));
                        }

                        {
                            let wind = weather.wind;
                            let mut wind_output = format!("{} m/s", wind.speed.round() as usize);

                            if let Some(direction_name) = get_wind_direction(wind.deg) {
                                wind_output.push_str(format!(", {}", direction_name).as_str());
                            }

                            fields.push(("Wind", wind_output, true));
                        }

                        {
                            let humidity = weather.main.humidity as usize;

                            fields.push(("Humidity", format!("{}%", humidity), true));
                        }

                        if let Some(cloudiness) = weather.clouds.all {
                            fields.push(("Cloudiness", format!("{}%", cloudiness), true));
                        }

                        {
                            let pressure = weather.main.pressure as usize;
                            let pressure = format!("{} mbar", pressure.to_formatted_string(&Locale::en));

                            fields.push(("Pressure", pressure, true));
                        }

                        if let Some(timezone_offset_secs) = weather.timezone {
                            let offset = FixedOffset::east(timezone_offset_secs);

                            if let Some(ref sys) = weather.sys {
                                if let Some(sunrise_secs) = sys.sunrise {
                                    let naive_sunrise = NaiveDateTime::from_timestamp(sunrise_secs as i64, 0);
                                    let sunrise = offset.from_utc_datetime(&naive_sunrise);

                                    fields.push(("Sunrise", sunrise.format("%I:%M %p").to_string(), true));
                                }

                                if let Some(sunset_secs) = sys.sunset {
                                    let naive_sunset = NaiveDateTime::from_timestamp(sunset_secs as i64, 0);
                                    let sunset = offset.from_utc_datetime(&naive_sunset);

                                    fields.push(("Sunrise", sunset.format("%I:%M %p").to_string(), true));
                                }
                            }

                            let local_date_time = offset.from_utc_datetime(&Utc::now().naive_utc());
                            let formatted = local_date_time.format("%b %d, %Y, %r (UTC%:z)");

                            fields.push(("Current Date", formatted.to_string(), false));
                        }

                        embed
                            .color(config::SUCCESS_COLOR)
                            .author(|a| a.name(location).icon_url(bot_pfp).url(url))
                            .fields(fields)
                            .footer(|f| f.text("Provided by OpenWeather"))
                    })
                }).await?;
            } else {
                message.channel_id.send_default_reply(
                    &cx.http,
                    "You have provided no arguments!",
                    EmbedType::Failure,
                ).await?;
            }

            Ok(())
        }
    }
}