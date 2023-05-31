// use crate::serial;

// pub mod backend {
//     pub struct app {
//         #[cfg(windows)]
//         ser: serial::Serial,
//         ser: serialport::SerialPortBuilder,
//     }
//     impl app {
//         pub fn new() -> Self {
//             Self {
//                 // ser: serial::Serial::new("/dev/ttyUSB0", libc::B115200),
//                 // ser: serial::Serial::new("/dev/ttyUSB0", libc::B115200),
//             }
//         }
//         pub fn run(&mut self) {
//             let mut buf = [0u8; 1024];
//             loop {
//                 if let Some(data) = self.ser.read(&mut buf) {
//                     println!("{:?}", data);
//                 }
//             }
//         }
//     }
// }
use crate::logger;
use serde_derive::{Deserialize, Serialize};
use std::{
    collections::{hash_map, hash_set},
    fs,
};
#[derive(Debug, Deserialize, Serialize)]
struct Config {
    baud_rate: u32,
    timeout: i32,
    maxdev: i32,
    trycnt: i32,
}
impl Config {
    pub fn new() -> Self {
        if let Ok(str) = fs::read_to_string("config.toml") {
            if let Ok(config) = toml::from_str(&str) {
                println!("{:#?}", config);
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
pub enum State {
    Prepare,
    Err(String),
    Ok(Box<dyn serialport::SerialPort>),
}
pub enum DevState {
    Right,
    Rate(i32),
}
pub struct Core {
    config: Config,
    state: State,
    pub ratidx: hash_map::HashMap<i32, DevState>,
}
pub fn available_ports() -> Vec<String> {
    if let Ok(v) = serialport::available_ports() {
        return v.iter().map(|p| p.port_name.clone()).collect();
    }
    Vec::new()
}
impl Default for Core {
    fn default() -> Self {
        Self {
            config: Config::default(),
            state: State::Prepare,
            ratidx: hash_map::HashMap::default(),
        }
    }
}
enum pollfor {
    Right,
    Query,
}
impl Core {
    pub fn new() -> Self {
        logger::trace("Core::new()");
        Self {
            config: Config::new(),
            ..Default::default()
        }
    }
    pub fn open<'a>(&mut self, path: impl Into<std::borrow::Cow<'a, str>>) {
        match self.state {
            State::Prepare => {
                match serialport::new(path, self.config.baud_rate)
                    .timeout(std::time::Duration::from_millis(self.config.timeout as u64))
                    .open()
                {
                    Ok(port) => self.state = State::Ok(port),
                    Err(_) => {
                        logger::warn("Failed to open serial port");
                        self.state = State::Err("Failed to open serial port".to_string());
                    }
                }
            }
            _ => return,
        }
    }
    pub fn config(&mut self) {}
    fn poll(&mut self, phase: pollfor) {
        match self.state {
            State::Ok(ref mut port) => {
                for i in match &phase {
                    pollfor::Right => (1..(self.config.maxdev + 1))
                        .filter(|i| match self.ratidx.get(i) {
                            Some(_) => false,
                            _ => true,
                        })
                        .collect::<Vec<_>>(),
                    pollfor::Query => self
                        .ratidx
                        .iter()
                        .filter_map(|d| match d.1 {
                            DevState::Right => Some(d.0.clone()),
                            _ => None,
                        })
                        .collect(),
                } {
                    let mut check = match &phase {
                        pollfor::Right => [0x5A, i as u8, 0x08, 0x00, 0x00],
                        pollfor::Query => [0x5A, 0x00, 0x03, i as u8, 0x00],
                    };
                    check[4] = check.iter().sum();

                    let trace = |opt: &str, pf: &pollfor, ok: bool| {
                        logger::trace(
                            format!(
                                "{} {} {} {}",
                                opt,
                                i,
                                match pf {
                                    pollfor::Right => "right",
                                    pollfor::Query => "query",
                                },
                                match ok {
                                    true => "ok",
                                    false => "failed",
                                }
                            )
                            .as_str(),
                        );
                    };

                    for _ in 0..self.config.trycnt {
                        match port.write(&check) {
                            Ok(5) => {
                                trace("send", &phase, true);

                                let mut buf = [0u8; 5];
                                match port.read_exact(buf.as_mut()) {
                                    Ok(_) => match phase {
                                        pollfor::Right if buf == check => {
                                            trace("recieve", &phase, true);
                                            self.ratidx.insert(i, DevState::Right);
                                            break;
                                        }
                                        pollfor::Query
                                            if buf[0] == 0x5A
                                                && buf[1] == i as u8
                                                && buf[2] == 0x03
                                                && buf[4]
                                                    == (buf
                                                        .iter()
                                                        .take(4)
                                                        .map(|c| *c as u16)
                                                        .sum::<u16>()
                                                        % 256)
                                                        as u8 =>
                                        {
                                            trace("recieve", &phase, true);
                                            self.ratidx.insert(i, DevState::Rate(buf[3] as i32));
                                            break;
                                        }
                                        _ => trace("recieve", &phase, false),
                                    },
                                    _ => trace("recieve", &phase, false),
                                }
                            }
                            Ok(_) | Err(_) => {
                                trace("send", &phase, false);
                                let _ = port.flush();
                                continue;
                            }
                        }
                    }
                }
            }
            _ => return,
        }
    }
    pub fn right(&mut self) {
        self.poll(pollfor::Right);
    }
    pub fn query(&mut self) {
        self.poll(pollfor::Query);
    }
}
