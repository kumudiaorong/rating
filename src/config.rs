use crate::logger;
use serde_derive::{Deserialize, Serialize};
use std::fs;
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub baud_rate: u32,
    pub timeout: i32,
    pub maxdev: i32,
    pub trycnt: i32,
}
impl Config {
    pub fn new() -> Self {
        if let Ok(str) = fs::read_to_string("config.toml") {
            if let Ok(config) = toml::from_str(&str) {
                return config;
            }
        }
        logger::warn("Failed to load config.toml, use default config");
        Self::default()
    }
}
impl Default for Config {
    fn default() -> Self {
        Self {
            baud_rate: 9600,
            timeout: 20,
            maxdev: 99,
            trycnt: 3,
        }
    }
}
