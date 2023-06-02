use crate::{config, logger, msg};
use msg::{msg_header::MsgType, rate_list, MsgHeader};
use prost::Message;
use std::collections::hash_map;
use std::sync::mpsc;
enum State {
    Prepare,
    Ok(Box<dyn serialport::SerialPort>),
}
pub enum DevState {
    Right,
    Rate(i32),
}
enum Pollfor {
    Right,
    Query,
}
pub struct Core {
    sender: mpsc::Sender<Vec<u8>>,
    receiver: mpsc::Receiver<Vec<u8>>,
    config: config::Config,
    state: State,
    ratidx: hash_map::HashMap<i32, DevState>,
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
    pub fn open<'a>(&mut self, path: impl Into<std::borrow::Cow<'a, str>>) -> bool {
        match serialport::new(path, self.config.baud_rate)
            .timeout(std::time::Duration::from_millis(self.config.timeout as u64))
            .open()
        {
            Ok(port) => {
                self.state = State::Ok(port);
                true
            }
            Err(_) => {
                logger::warn("Failed to open serial port");
                false
            }
        }
    }
    fn poll(&mut self, phase: Pollfor) {
        match self.state {
            State::Ok(ref mut port) => {
                for i in match &phase {
                    Pollfor::Right => (1..(self.config.max_dev + 1))
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

                    for _ in 0..self.config.try_cnt {
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
                                                && buf[3] != 0x6f
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
    pub fn run(&mut self) {
        logger::trace("Core::run()");
        loop {
            if let Ok(rcev) = self.receiver.recv() {
                if let Ok(h) = MsgHeader::decode(rcev.as_slice()) {
                    match h.tp() {
                        MsgType::Open => {
                            if let Ok(path) = self.receiver.recv() {
                                if let Ok(port) = msg::Port::decode(path.as_slice()) {
                                    self.open(port.path);
                                }
                            }
                        }
                        MsgType::Close => match self.state {
                            State::Ok(_) => {
                                self.state = State::Prepare;
                            }
                            _ => (),
                        },
                        MsgType::Right => self.poll(Pollfor::Right),
                        MsgType::Query => {
                            self.poll(Pollfor::Query);
                            if let Ok(_) = self
                                .sender
                                .send(MsgHeader::new(MsgType::Query).encode_to_vec())
                            {
                                let _ = self.sender.send(
                                    msg::RateList::new(
                                        self.ratidx
                                            .iter()
                                            .map(|d| {
                                                let (score, state) = match d.1 {
                                                    DevState::Rate(r) if r > &100 => {
                                                        (r, rate_list::State::Error)
                                                    }
                                                    DevState::Rate(r) => (r, rate_list::State::Ok),
                                                    _ => (&-1, rate_list::State::Ready),
                                                };
                                                msg::rate_list::Rate::new(
                                                    d.0.clone(),
                                                    *score,
                                                    state,
                                                )
                                            })
                                            .collect(),
                                    )
                                    .encode_to_vec(),
                                );
                            }
                        }
                        MsgType::Next => {
                            self.ratidx.iter_mut().for_each(|(_, s)| {
                                *s = DevState::Right;
                            });
                        }
                        MsgType::Reset => match self.state {
                            State::Ok(ref mut port) => {
                                for _ in 0..self.config.try_cnt {
                                    if let Err(_) =
                                        port.write(&[0x5a as u8, 0x00, 0x01, 0x00, 0x5b])
                                    {
                                        logger::warn("Failed to reset serial port");
                                    }
                                }
                                self.ratidx.clear();
                            }
                            _ => (),
                        },
                        MsgType::Reload => {
                            self.config.reload();
                        }
                    }
                }
            }
        }
    }
}
