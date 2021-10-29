mod cmds;
mod config;
mod utils;

use crate::{
    cmds::{Command, COMMANDS},
    utils::{
        discord::{DefaultEmbedReplies, EmbedType},
        string::strip_str,
    },
};

use lazy_static::lazy_static;

use serenity::{
    client::Context,
    model::{
        channel::{Message, MessageType},
        gateway::{Activity, Ready},
        prelude::OnlineStatus,
    },
    prelude::EventHandler,
    Client,
    async_trait,
};

use std::{
    error::Error,
    sync::atomic::{AtomicUsize, Ordering},
    time::SystemTime,
};

use tracing::info;

lazy_static! {
    static ref START_TIME: SystemTime = SystemTime::now();
}

static CONNECT_COUNT: AtomicUsize = AtomicUsize::new(0);

struct Handler;

#[allow(unused_must_use)]
#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, cx: Context, message: Message) {
        let starts_with_prefix = message.content.starts_with(config::PREFIX);
        let author_is_not_bot = !message.author.bot;
        let is_from_guild = message.guild_id.is_some();
        let is_regular = message.kind == MessageType::Regular;

        if starts_with_prefix && author_is_not_bot && is_from_guild && is_regular {
            let mut args: Vec<&str> = message.content.split(" ").collect();

            if let Some(cmd_name) = args.first() {
                if let Some(cmd_name) = cmd_name.to_lowercase().strip_prefix(config::PREFIX) {
                    let commands: Vec<&Box<dyn Command>> = (*COMMANDS).iter()
                        .filter(|c| c.name() == cmd_name || c.aliases().contains(&cmd_name))
                        .collect();

                    if let Some(cmd) = commands.first() {
                        if cmd.is_developer()
                            && !(*config::DEVELOPERS).lock().unwrap().contains(message.author.id.as_u64())
                        {
                            message.channel_id.send_default_reply(
                                &cx.http,
                                "You do not have permissions to execute the command!",
                                EmbedType::Failure,
                            ).await;

                            return;
                        }

                        args.remove(0);

                        if let Err(e) = cmd.invoke(&cx, &message, &args).await {
                            if let Some(limited_message) =
                                strip_str(e.to_string().as_str(), 2000, true)
                            {
                                message.channel_id.send_default_reply(
                                    &cx.http,
                                    &limited_message,
                                    EmbedType::Failure,
                                ).await;
                            }
                        }
                    }
                }
            }
        }
    }

    async fn ready(&self, cx: Context, _data_about_bot: Ready) {
        cx.set_presence(
            Some(Activity::watching(format!("{}help", config::PREFIX))),
            OnlineStatus::DoNotDisturb,
        ).await;

        if CONNECT_COUNT.fetch_add(1, Ordering::SeqCst) == 0 {
            if let Ok(application_info) = cx.http.get_current_application_info().await {
                let mut devs = config::DEVELOPERS.lock().unwrap();

                devs.push(*application_info.owner.id.as_u64());
            }
        }

        info!("Connected to Discord!");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    {
        std::env::set_var("RUST_LOG", "INFO");

        tracing_subscriber::fmt::init();

        lazy_static::initialize(&config::DISCORD_TOKEN);
        lazy_static::initialize(&START_TIME);

        info!("Starting!");
    }

    let token = config::DISCORD_TOKEN.as_str();

    // Required for using Discord interactions
    let id = base64::decode(token.split(".").collect::<Vec<&str>>()[0])?;

    let mut client = Client::builder(token)
        .application_id(String::from_utf8(id)?.parse()?)
        .event_handler(Handler)
        .await?;

    client.start().await?;

    Ok(())
}