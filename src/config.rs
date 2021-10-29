#![allow(dead_code)]

use lazy_static::lazy_static;

use serenity::utils::Color;

use std::{
    env::var,
    sync::Mutex,
};

pub const PREFIX: &str = "<";

lazy_static! {
    pub static ref DISCORD_TOKEN: String = var("DISCORD_TOKEN").unwrap();
    pub static ref WEATHER_API_KEY: String = var("WEATHER_API_KEY").unwrap();

    pub static ref DEVELOPERS: Mutex<Vec<u64>> = Mutex::new(Vec::new());
}

pub const SUCCESS_COLOR: Color = Color::from_rgb(0x70, 0x55, 0x44);
pub const FAILURE_COLOR: Color = Color::from_rgb(0xef, 0x43, 0x3f);
pub const CONFIRMATION_COLOR: Color = Color::from_rgb(0x78, 0xb4, 0x54);
pub const WARNING_COLOR: Color = Color::from_rgb(0xff, 0xf2, 0x36);