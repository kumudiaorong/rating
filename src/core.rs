use crate::config;
use crate::logger;
use std::{
    collections::{hash_map, hash_set},
    fs,
};

pub enum State {
    Prepare,
    Ok(Box<dyn serialport::SerialPort>),
}
pub enum DevState {
    Right,
    Rate(i32),
}
use msg::msg_type::Type;
use std::sync::mpsc;
pub fn available_ports() -> Vec<String> {
    if let Ok(v) = serialport::available_ports() {
        return v.iter().map(|p| p.port_name.clone()).collect();
    }
    Vec::new()
}
enum Pollfor {
    Right,
    Query,
}
use crate::msg;
use prost::Message;

pub struct Core {
    sender: mpsc::Sender<Vec<u8>>,
    receiver: mpsc::Receiver<Vec<u8>>,
    config: config::Config,
    state: State,
    pub ratidx: hash_map::HashMap<i32, DevState>,
}
impl Core {
    pub fn new(sender: mpsc::Sender<Vec<u8>>, receiver: mpsc::Receiver<Vec<u8>>) -> Self {
        logger::trace("Core::new()");
        Self {
            config: config::Config::new(),
            sender,
            receiver,
            state: State::Prepare,
            ratidx: hash_map::HashMap::new(),
        }
    }
    pub fn open<'a>(&mut self, path: impl Into<std::borrow::Cow<'a, str>>) -> Result<(), ()> {
        match self.state {
            State::Prepare => {
                match serialport::new(path, self.config.baud_rate)
                    .timeout(std::time::Duration::from_millis(self.config.timeout as u64))
                    .open()
                {
                    Ok(port) => {
                        self.state = State::Ok(port);
                        Ok(())
                    }
                    Err(_) => {
                        logger::warn("Failed to open serial port");
                        Err(())
                    }
                }
            }
            _ => Err(()),
        }
    }
    pub fn config(&mut self) {}
    fn poll(&mut self, phase: Pollfor) {
        match self.state {
            State::Ok(ref mut port) => {
                for i in match &phase {
                    Pollfor::Right => (1..(self.config.maxdev + 1))
                        .filter(|i| match self.ratidx.get(i) {
                            Some(_) => false,
                            _ => true,
                        })
                        .collect::<Vec<_>>(),
                    Pollfor::Query => self
                        .ratidx
                        .iter()
                        .filter_map(|d| match d.1 {
                            DevState::Right => Some(d.0.clone()),
                            _ => None,
                        })
                        .collect(),
                } {
                    let mut check = match &phase {
                        Pollfor::Right => [0x5A, i as u8, 0x08, 0x00, 0x00],
                        Pollfor::Query => [0x5A, 0x00, 0x03, i as u8, 0x00],
                    };
                    check[4] = check.iter().sum();

                    let trace = |opt: &str, pf: &Pollfor, ok: bool| {
                        logger::trace(
                            format!(
                                "{} {} {} {}",
                                opt,
                                i,
                                match pf {
                                    Pollfor::Right => "right",
                                    Pollfor::Query => "query",
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
                                        Pollfor::Right if buf == check => {
                                            trace("receive", &phase, true);
                                            self.ratidx.insert(i, DevState::Right);
                                            break;
                                        }
                                        Pollfor::Query
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
                                            trace("receive", &phase, true);
                                            self.ratidx.insert(i, DevState::Rate(buf[3] as i32));
                                            break;
                                        }
                                        _ => trace("receive", &phase, false),
                                    },
                                    _ => trace("receive", &phase, false),
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
        self.poll(Pollfor::Right);
    }
    pub fn query(&mut self) {
        self.poll(Pollfor::Query);
    }
    pub fn run(&mut self) {
        use msg::MsgType;
        logger::trace("Core::run()");
        let mut ok = true;
        loop {
            match self.receiver.recv() {
                Ok(rcev) => match MsgType::decode(rcev.as_slice()) {
                    Ok(tp) => match tp.r#type() {
                        Type::Check => {
                            ok = true;
                        }
                        other => match other {
                            Type::Open => match self.receiver.recv() {
                                Ok(path) => {
                                    if let Ok(port) = msg::Port::decode(path.as_slice()) {
                                        self.open(port.path);
                                    }
                                    self.sender
                                        .send(MsgType::new(Type::Error).encode_to_vec())
                                        .unwrap();
                                }
                                Err(_) => self
                                    .sender
                                    .send(MsgType::new(Type::Error).encode_to_vec())
                                    .unwrap(),
                            },
                            Type::Right => {
                                self.right();
                            }
                            Type::Query => {
                                self.query();
                                self.sender
                                    .send(MsgType::new(Type::Query).encode_to_vec())
                                    .unwrap();
                                self.sender
                                    .send(
                                        msg::RateList::new(
                                            self.ratidx
                                                .iter()
                                                .map(|d| {
                                                    msg::rate_list::Rate::new(
                                                        d.0.clone(),
                                                        match d.1 {
                                                            DevState::Rate(r) => r.to_string(),
                                                            _ => "ready".to_string(),
                                                        },
                                                    )
                                                })
                                                .collect(),
                                        )
                                        .encode_to_vec(),
                                    )
                                    .unwrap();
                            }
                            _ => (),
                        },
                    },
                    Err(_) => ok = false,
                },
                Err(_) => (),
            }
        }
    }
}
