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
    Err,
    Ok,
}
pub struct Core {
    config: Config,
    state: State,
    erridx: hash_set::HashSet<i32>,
    extidx: hash_set::HashSet<i32>,
    pub ratidx: hash_map::HashMap<i32, i32>,
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
            erridx: hash_set::HashSet::default(),
            extidx: hash_set::HashSet::default(),
            ratidx: hash_map::HashMap::default(),
        }
    }
}
impl Core {
    pub fn new() -> Self {
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
                    Err(_) => self.state = State::Err("Failed to open serial port".to_string()),
                }
            }
            _ => return,
        }
    }
    pub fn config(&mut self) {}
    pub fn right(&mut self) {
        match self.state {
            State::Ok(ref mut port) => {
                for i in 1..(self.config.maxdev + 1) {
                    if self.extidx.contains(&i) || self.ratidx.contains_key(&i) {
                        continue;
                    }
                    let mut check = [0x5A, i as u8, 0x08, 0x00, 0x00];
                    check[4] = check.iter().sum();
                    for _ in 0..self.config.trycnt {
                        match port.write(&check) {
                            Ok(5) => {
                                println!("get {}", i);
                                let mut buf = [0u8; 5];
                                let mut sum = 0;
                                loop {
                                    match port.read(buf[sum..].as_mut()) {
                                        Ok(cnt) => {
                                            if cnt == 0 {
                                                break;
                                            }
                                            sum += cnt;
                                        }
                                        Err(_) => {
                                            break;
                                        }
                                    }
                                }
                                println!("get read {} {}", i, sum);
                                if sum != 0 {
                                    if sum == 5 && buf == check {
                                        println!("right {}", i);
                                        self.extidx.insert(i);
                                        break;
                                    }
                                    println!("err {} {} {:#?}", i, sum, buf);
                                    self.erridx.insert(i);
                                    break;
                                }
                            }
                            Ok(_) | Err(_) => {
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
    pub fn query(&mut self) {
        match self.state {
            State::Ok(ref mut port) => {
                for i in self.extidx.iter() {
                    if self.ratidx.contains_key(&i) {
                        continue;
                    }
                    let mut check = [0x5A, 0x00, 0x03, *i as u8, 0x00];
                    check[4] = check.iter().sum();
                    for _ in 0..self.config.trycnt {
                        match port.write(&check) {
                            Ok(5) => {
                                println!("query {}", i);
                                let mut buf = [0u8; 5];
                                let mut sum = 0;
                                loop {
                                    match port.read(buf[sum..].as_mut()) {
                                        Ok(cnt) => {
                                            if cnt == 0 {
                                                break;
                                            }
                                            sum += cnt;
                                        }
                                        Err(_) => {
                                            break;
                                        }
                                    }
                                }

                                if sum == 5
                                    && buf[0] == 0x5A
                                    && buf[1] == *i as u8
                                    && buf[2] == 0x03
                                    && buf[4]
                                        == (buf.iter().take(4).map(|c| *c as u16).sum::<u16>()
                                            % 256) as u8
                                {
                                    println!("rate {}", i);
                                    self.ratidx.insert(i.clone(), buf[3] as i32);
                                    break;
                                }
                            }
                            Ok(_) | Err(_) => {
                                let _ = port.flush();
                                continue;
                            }
                        }
                    }
                }
            }
            _ => (),
        }
    }
}
