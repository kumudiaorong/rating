use crate::{config, logger, msg, msg::rate_list};
use iced::widget::{self, column, row, Column};
use iced::{
    alignment, executor, window, Application, Command, Element, Length, Subscription, Theme,
};
use msg::msg_header::MsgType;
use msg::MsgHeader;
use prost::Message;
use std::sync::mpsc;
#[derive(Debug, Clone)]
pub enum AppMessage {
    Tick,
    OpenSerial,
    ReCheck,
    ReQuery,
    ReSet,
    Save,
    Apply,
    PortSelected(String),
    CfgBaudRate(u32),
    CfgTimeout(String),
    CfgMaxDev(String),
    CfgTryCnt(String),
}
pub struct App {
    sender: mpsc::Sender<Vec<u8>>,
    receiver: mpsc::Receiver<Vec<u8>>,
    choosed: Option<String>,
    available_ports: Vec<String>,
    ratelist: msg::RateList,
    state: String,
    config: config::Config,
    is_open: bool,
}
impl App {
    fn send<T: Message>(&mut self, msg: T, err: &str) -> bool {
        match self.sender.send(msg.encode_to_vec()) {
            Ok(_) => true,
            Err(_) => {
                self.state = err.to_string();
                logger::warn(err);
                false
            }
        }
    }
    fn send_header(&mut self, tp: MsgType, err: &str) -> bool {
        self.send(MsgHeader::new(tp), err)
    }
    fn call(&mut self) {
        self.send(
            msg::Port::new(self.choosed.clone().unwrap()),
            "send path request failed",
        )
        .then(|| {
            self.send_header(MsgType::Right, "send right request failed")
                .then(|| {
                    self.send_header(MsgType::Query, "send query request failed")
                        .then(|| self.state = String::from("send query request success"))
                })
        });
    }
    fn open(&mut self) {
        if self.is_open {
            self.send_header(MsgType::Close, "send close request failed")
                .then(|| self.is_open = false);
        } else if self.choosed.is_some() {
            let _ = self
                .send_header(MsgType::Open, "send open request failed")
                .then(|| {
                    self.is_open = true;
                    self.call()
                });
        }
    }
    fn proc_ticks(&mut self) {
        self.available_ports = config::available_ports();
        if let Ok(rcev) = self.receiver.try_recv() {
            if let Ok(h) = MsgHeader::decode(rcev.as_slice()) {
                match h.tp() {
                    MsgType::Query => {
                        if let Ok(rcev) = self.receiver.recv() {
                            if let Ok(rl) = msg::RateList::decode(rcev.as_slice()) {
                                self.ratelist = rl;
                                self.ratelist.rates.sort_by(|l, r| l.addr.cmp(&r.addr));
                                self.state = String::from("receive query response success")
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    }
}
impl Application for App {
    type Executor = executor::Default;
    type Message = AppMessage;
    type Theme = Theme;
    type Flags = (mpsc::Sender<Vec<u8>>, mpsc::Receiver<Vec<u8>>);

    fn new(flags: Self::Flags) -> (Self, Command<AppMessage>) {
        (
            Self {
                sender: flags.0,
                receiver: flags.1,
                choosed: None,
                available_ports: config::available_ports(),
                ratelist: msg::RateList::default(),
                state: String::from("Ok"),
                config: config::Config::default(),
                is_open: false,
            },
            window::resize(750, 245),
        )
    }

    fn title(&self) -> String {
        String::from("Rating")
    }

    fn update(&mut self, message: AppMessage) -> Command<AppMessage> {
        match message {
            AppMessage::Tick => self.proc_ticks(),
            AppMessage::OpenSerial => self.open(),
            AppMessage::ReSet => {
                self.send_header(MsgType::Reset, "send reset request failed");
            }
            AppMessage::ReCheck => {
                self.send_header(MsgType::Right, "send right request failed");
            }
            AppMessage::ReQuery => {
                self.send_header(MsgType::Query, "send query request failed")
                    .then(|| self.state = String::from("send query request success"));
            }
            AppMessage::Save => self.config.save(),
            AppMessage::Apply => {
                self.send_header(MsgType::Reload, "send load request failed")
                    .then(|| self.call());
            }
            AppMessage::PortSelected(path) => self.choosed = Some(path),
            AppMessage::CfgBaudRate(rate) => self.config.baud_rate = rate,
            AppMessage::CfgTimeout(timeout) => {
                self.config.timeout = timeout.parse().unwrap_or(self.config.timeout)
            }
            AppMessage::CfgMaxDev(max_dev) => {
                self.config.max_dev = max_dev.parse().unwrap_or(self.config.max_dev)
            }
            AppMessage::CfgTryCnt(try_cnt) => {
                self.config.try_cnt = try_cnt.parse().unwrap_or(self.config.try_cnt)
            }
        }
        Command::none()
    }
    fn subscription(&self) -> Subscription<AppMessage> {
        iced::time::every(std::time::Duration::from_millis(100)).map(|_| AppMessage::Tick)
    }
    fn view(&self) -> Element<AppMessage> {
        let available_list = widget::pick_list(
            &self.available_ports,
            self.choosed.clone(),
            AppMessage::PortSelected,
        )
        .placeholder("choose a port");

        let ratesheader = add_boder(row![
            std_text("idx"),
            std_text("addr"),
            std_text("score"),
            std_text("state"),
        ]);
        let ratesbody = add_boder(
            widget::scrollable(
                Column::with_children(
                    self.ratelist
                        .rates
                        .iter()
                        .enumerate()
                        .map(|(i, r)| create_row(i, r.addr, r.score, r.state()))
                        .collect(),
                )
                .width(Length::Fixed(320.0)),
            )
            .height(180),
        );
        let rates = column!(ratesheader, ratesbody)
            .spacing(5)
            .align_items(alignment::Alignment::Center);

        let allokscores = self.ratelist.rates.iter().filter_map(|r| match r.state() {
            rate_list::State::Ok => Some(r.score),
            _ => None,
        });
        let allokscoreslen = allokscores.clone().count();
        let sumscore = allokscores.clone().sum::<i32>();
        let display = column![
            add_boder(
                column![
                    creat_info("sum", allokscores.clone().sum::<i32>()),
                    creat_info("max", allokscores.clone().max().unwrap_or(-1)),
                    creat_info("min", allokscores.clone().min().unwrap_or(-1)),
                    creat_info("avg", sumscore / allokscoreslen.max(1) as i32),
                ]
                .spacing(5)
            ),
            row![
                widget::button(std_text(if self.is_open { "close" } else { "open" }))
                    .on_press(AppMessage::OpenSerial),
                widget::button(std_text("reset")).on_press(AppMessage::ReSet),
            ]
            .spacing(5),
            row![
                widget::button(std_text("recheck")).on_press(AppMessage::ReCheck),
                widget::button(std_text("requery")).on_press(AppMessage::ReQuery),
            ]
            .spacing(5),
        ]
        .spacing(5);

        let cfg = column![
            row![std_text("port"), available_list].spacing(5),
            row![
                std_text("baud rate"),
                widget::pick_list(
                    &config::BAUD_RATES[..],
                    Some(self.config.baud_rate),
                    AppMessage::CfgBaudRate,
                ),
            ]
            .spacing(5),
            row![
                std_text("timeout"),
                widget::text_input("", self.config.timeout.to_string().as_str())
                    .width(Length::Fixed(135.0))
                    .on_input(AppMessage::CfgTimeout),
            ]
            .spacing(5),
            row![
                std_text("max dev"),
                widget::text_input("", self.config.max_dev.to_string().as_str())
                    .width(Length::Fixed(135.0))
                    .on_input(AppMessage::CfgMaxDev),
            ]
            .spacing(5),
            row![
                std_text("try times"),
                widget::text_input("", self.config.try_cnt.to_string().as_str())
                    .width(Length::Fixed(135.0))
                    .on_input(AppMessage::CfgTryCnt),
            ]
            .spacing(5),
            widget::vertical_space(15),
            row![
                widget::button(std_text("save")).on_press(AppMessage::Save),
                widget::button(std_text("apply")).on_press(AppMessage::Apply),
            ]
            .spacing(40),
        ]
        .spacing(5);

        widget::container(row![rates, display, cfg].spacing(5))
            .center_x()
            .center_y()
            .width(Length::Shrink)
            .height(Length::Shrink)
            .padding([5, 5, 5, 5])
            .into()
    }
}

fn add_boder<'a>(c: impl Into<Element<'a, AppMessage>>) -> Element<'a, AppMessage> {
    widget::container(c)
        .padding(5)
        .style(iced::theme::Container::Custom(Box::new(Boder)))
        .into()
}

fn creat_info(name: impl ToString, val: i32) -> Element<'static, AppMessage> {
    row![std_text(name), std_text(val)].spacing(5).into()
}

fn std_text<'a>(t: impl ToString) -> Element<'a, AppMessage> {
    widget::text(t)
        .width(80)
        .height(30)
        .horizontal_alignment(alignment::Horizontal::Center)
        .vertical_alignment(alignment::Vertical::Center)
        .into()
}
fn create_row<'a>(
    idx: usize,
    addr: i32,
    score: i32,
    state: rate_list::State,
) -> Element<'a, AppMessage> {
    row![
        std_text(idx),
        std_text(addr),
        std_text(score),
        std_text(match state {
            rate_list::State::Ok => "Ok",
            rate_list::State::Ready => "Ready",
            rate_list::State::Error => "Error",
        }),
    ]
    .into()
}

#[derive(Default)]
struct Boder;

impl widget::container::StyleSheet for Boder {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> widget::container::Appearance {
        widget::container::Appearance {
            border_radius: 1.0,
            border_width: 1.0,
            border_color: iced::Color::BLACK,
            ..Default::default()
        }
    }
}
