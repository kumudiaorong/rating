use crate::logger;
use serde_derive::{Deserialize, Serialize};
use std::fs;
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub baud_rate: u32,
    pub timeout: i32,
    pub max_dev: i32,
    pub try_cnt: i32,
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
    pub fn reload(&mut self) {
        if let Ok(str) = fs::read_to_string("config.toml") {
            if let Ok(config) = toml::from_str(&str) {
                *self = config;
            }
        }
    }
    pub fn save(&self) {
        if let Ok(str) = toml::to_string(&self) {
            if let Ok(_) = fs::write("config.toml", str) {
                logger::info("Config saved");
            }
        }
    }
}
impl Default for Config {
    fn default() -> Self {
        Self {
            baud_rate: 9600,
            timeout: 20,
            max_dev: 99,
            try_cnt: 3,
        }
    }
}
pub fn available_ports() -> Vec<String> {
    if let Ok(v) = serialport::available_ports() {
        return v.iter().map(|p| p.port_name.clone()).collect();
    }
    Vec::new()
}
pub const BAUD_RATES: [u32; 9] = [1200, 1800, 2400, 4800, 9600, 19200, 38400, 57600, 115200];
