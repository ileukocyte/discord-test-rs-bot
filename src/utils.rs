#![allow(dead_code)]

pub mod time {
    use chrono::Duration;

    use crate::utils::string::singular_or_plural;

    // should have been `usize`, but `chrono::Duration` only accepts `i64`-values
    pub fn as_text(millis: i64) -> String {
        let mut output = String::new();

        let duration = Duration::milliseconds(millis);

        let days = duration.num_days();
        let hours = duration.num_hours() % 24;
        let minutes = duration.num_minutes() % 60;
        let seconds = duration.num_seconds() % 60;

        let mut already_present = false;

        if days > 0 {
            output.push_str(format!("{} {}", days, singular_or_plural("day", days)).as_str());

            already_present = true;
        }

        if hours > 0 {
            if already_present && (minutes != 0 || seconds != 0) {
                output.push_str(", ");
            }

            if !output.is_empty() && (minutes == 0 || seconds == 0) {
                if output.contains(", ") {
                    output.push_str(",");
                }

                output.push_str(" and ");
            }

            output.push_str(format!("{} {}", hours, singular_or_plural("hour", hours)).as_str());

            already_present = true;
        }

        if minutes > 0 {
            if already_present && seconds != 0 {
                output.push_str(", ");
            }

            if !output.is_empty() && seconds == 0 {
                if output.contains(", ") {
                    output.push_str(",");
                }

                output.push_str(" and ");
            }

            output.push_str(format!("{} {}", minutes, singular_or_plural("minute", minutes)).as_str());

            already_present = true;
        }

        if seconds > 0 {
            if already_present {
                if output.contains(", ") {
                    output.push_str(",");
                }

                output.push_str(" and ");
            }

            output.push_str(format!("{} {}", seconds, singular_or_plural("second", seconds)).as_str());
        }

        if output.is_empty() && !already_present {
            output.push_str("0 seconds");
        }

        output
    }
}

pub mod string {
    pub fn strip_str(s: &str, mut lim: usize, add_ellipsis: bool) -> Option<String> {
        if add_ellipsis {
            if lim < 4 {
                return None;
            }

            lim -= 3;
        }

        let stripped = s
            .get(..lim)
            .map(|str| {
                add_ellipsis
                    .then(|| format!("{}...", str))
                    .unwrap_or_else(|| str.to_owned())
            })
            .unwrap_or_else(|| s.to_owned());

        Some(stripped)
    }

    pub fn singular_or_plural(s: &str, n: i64) -> String {
        if n == 1 {
            s.to_owned()
        } else {
            format!("{}s", s)
        }
    }

    pub fn singular_or_plural_u64(s: &str, n: u64) -> String {
        if n == 1 {
            s.to_owned()
        } else {
            format!("{}s", s)
        }
    }
}

pub mod discord {
    use crate::config;

    use serenity::{
        http::Http,
        model::{
            channel::Message,
            id::ChannelId,
        },
        utils::Color,
        Result,
        async_trait,
    };

    #[async_trait]
    pub trait DefaultEmbedReplies {
        async fn send_default_reply<'http, D: ToString + Send>(
            self,
            http: impl AsRef<Http> + Send + Sync + 'http,
            description: D,
            embed_type: EmbedType,
        ) -> Result<Message>;
    }

    #[async_trait]
    impl DefaultEmbedReplies for ChannelId {
        async fn send_default_reply<'http, D: ToString + Send>(
            self,
            http: impl AsRef<Http> + Send + Sync + 'http,
            description: D,
            embed_type: EmbedType,
        ) -> Result<Message> {
            self.send_message(&http, |m| {
                m.embed(|embed| {
                    embed
                        .author(|a| a.name(format!("{}!", embed_type.to_string())))
                        .color(embed_type.get_color())
                        .description(description)
                })
            }).await
        }
    }

    #[derive(Debug)]
    pub enum EmbedType {
        Success,
        Failure,
        Confirmation,
        Warning,
    }

    impl ToString for EmbedType {
        fn to_string(&self) -> String {
            format!("{:?}", self)
        }
    }

    impl EmbedType {
        pub fn get_color(&self) -> Color {
            match self {
                EmbedType::Success => config::SUCCESS_COLOR,
                EmbedType::Failure => config::FAILURE_COLOR,
                EmbedType::Confirmation => config::CONFIRMATION_COLOR,
                EmbedType::Warning => config::WARNING_COLOR,
            }
        }
    }
}

pub mod misc {
    pub(crate) fn get_wind_direction(deg: f32) -> Option<String> {
        let deg = deg as i32;

        if let 0..=360 = deg {
            let name = match deg {
                0..=25 | 336..=360 => "North",
                26..=70 => "Northeast",
                71..=110 => "East",
                111..=155 => "Southeast",
                156..=200 => "South",
                201..=250 => "Southwest",
                251..=290 => "West",
                291..=335 => "Northwest",
                _ => "",
            };

            Some(name.to_owned())
        } else {
            None
        }
    }
}