use crate::core;
use iced::widget::scrollable::{Properties, Scrollbar, Scroller};
use iced::widget::{
    button, column, container, horizontal_space, pick_list, progress_bar, radio, row, scrollable,
    slider, text, vertical_space, Column,
};
use iced::{alignment, executor, theme, Alignment, Color};
use iced::{subscription, Application, Command, Element, Length, Settings, Subscription, Theme};
use iced::{widget, window};
#[derive(Debug, Clone)]
pub enum AppMessage {
    Selected(String),
    OpenSerial,
    ReCheck,
    ReQuery,
    // Receive(crate::msg::RateList),
    // AvailablePorts(Vec<String>),
    Tick,
}
use msg::MsgHeader;
use std::collections::LinkedList;
use std::sync::mpsc;
pub struct App {
    sender: mpsc::Sender<Vec<u8>>,
    receiver: mpsc::Receiver<Vec<u8>>,
    choosed: Option<String>,
    available_ports: Vec<String>,
    ratelist: crate::msg::RateList,
    count: i32,
    state: String,
}
use crate::msg;
use msg::msg_header::MsgType;
use prost::Message;
impl App {
    fn send<T: Message>(&mut self, msg: T, err: &str) -> bool {
        match self.sender.send(msg.encode_to_vec()) {
            Ok(_) => true,
            Err(_) => {
                self.state = err.to_string();
                false
            }
        }
    }
    fn send_header(&mut self, tp: MsgType, err: &str) -> bool {
        self.send(MsgHeader::new(tp), err)
    }
}
use crate::msg::rate_list;
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
                available_ports: core::available_ports(),
                ratelist: msg::RateList::default(),
                count: 0,
                state: String::from("Ok"),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Rating")
    }

    fn update(&mut self, message: AppMessage) -> Command<AppMessage> {
        self.count += 1;
        match message {
            AppMessage::Selected(path) => self.choosed = Some(path),
            AppMessage::OpenSerial => {
                if self.choosed.is_some() {
                    let _ = self
                        .send_header(MsgType::Open, "send open request failed")
                        .then(|| {
                            self.send(
                                msg::Port::new(self.choosed.clone().unwrap()),
                                "send path request failed",
                            )
                            .then(|| {
                                self.send_header(MsgType::Right, "send right request failed")
                                    .then(|| {
                                        self.send_header(
                                            MsgType::Query,
                                            "send query request failed",
                                        )
                                        .then(|| {
                                            self.state = String::from("send query request success")
                                        })
                                    })
                            })
                        });
                }
            }
            AppMessage::ReCheck => {
                self.send_header(MsgType::Right, "send right request failed");
            }
            AppMessage::ReQuery => {
                self.send_header(MsgType::Query, "send query request failed")
                    .then(|| self.state = String::from("send query request success"));
            }
            AppMessage::Tick => {
                self.available_ports = core::available_ports();
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
        Command::none()
    }
    fn subscription(&self) -> Subscription<AppMessage> {
        iced::time::every(std::time::Duration::from_millis(10)).map(|_| AppMessage::Tick)
    }
    fn view(&self) -> Element<AppMessage> {
        let available_list = container(
            pick_list(
                &self.available_ports,
                self.choosed.clone(),
                AppMessage::Selected,
            )
            .placeholder("choose a port"),
        )
        .height(40)
        .width(Length::Shrink)
        .center_x()
        .center_y();

        // container(row![
        //     // container("tes").style(theme::Container::Custom(Box::new(Container::default()))),
        //     add_boder("tes"),
        //     container(text("tes"))
        //         .style(iced::theme::Container::Custom(Box::new(MyContainerStyle)))
        //         .padding(2),
        //     column![
        //         row![
        //             text("idx")
        //                 .width(100)
        //                 .horizontal_alignment(alignment::Horizontal::Center)
        //                 .vertical_alignment(alignment::Vertical::Center),
        //             text("score")
        //                 .width(100)
        //                 .horizontal_alignment(alignment::Horizontal::Center)
        //                 .vertical_alignment(alignment::Vertical::Center)
        //         ],
        //         // .spacing(40),
        //         scrollable(
        //             // text(self.choosed.clone().unwrap_or("".to_string())).width(Length::Fill),
        //             widget::Column::with_children(
        //                 self.ratelist
        //                     .rates
        //                     .iter()
        //                     .map(|r| {
        //                         add_boder(row![
        //                             text(r.addr)
        //                                 .width(100)
        //                                 .horizontal_alignment(alignment::Horizontal::Center)
        //                                 .vertical_alignment(alignment::Vertical::Center),
        //                             text(r.state.as_str())
        //                                 .width(100)
        //                                 .horizontal_alignment(alignment::Horizontal::Center)
        //                                 .vertical_alignment(alignment::Vertical::Center)
        //                         ])
        //                     })
        //                     .collect::<Vec<Element<AppMessage>>>()
        //             )
        //             // .width(Length::Shrink)
        //             .align_items(Alignment::Center) // .padding([0, 0, 0, 0])
        //                                             // .spacing(40),
        //         )
        //         .height(Length::Fill), // .vertical_scroll(Properties::default())
        //                                // .into()
        //     ],
        //     available_list,
        //     button("open")
        //         .on_press(AppMessage::OpenSerial)
        //         .width(Length::Shrink)
        // ])
        // .center_x()
        // .center_y()
        // .width(Length::Shrink)
        // .height(Length::Shrink)
        // .padding([5, 0, 0, 0])
        // .into()

        // let mut rates = vec![create_row("idx", "addr", "state")];
        // rates.extend(
        //     self.ratelist
        //         .rates
        //         .iter()
        //         .enumerate()
        //         .map(|(i, r)| create_row(i, r.addr, r.state.as_str()))
        //         .collect(),
        // );
        let allokscores = self.ratelist.rates.iter().filter_map(|r| match r.state() {
            rate_list::State::Ok => Some(r.score),
            _ => None,
        });
        let allokscoreslen = allokscores.clone().count();
        let sumscore = allokscores.clone().sum::<i32>();
        // let maxscore = allokscores.clone().max().unwrap_or(-1);
        // let minscore = allokscores.clone().min().unwrap_or(-1);
        // let avgscore = sumscore / allokscoreslen.min(1) as i32;
        let displayscores = column![
            row![std_text("sum"), std_text(allokscores.clone().sum::<i32>())].spacing(10),
            row![
                std_text("max"),
                std_text(allokscores.clone().max().unwrap_or(-1))
            ]
            .spacing(10),
            row![
                std_text("min"),
                std_text(allokscores.clone().min().unwrap_or(-1))
            ]
            .spacing(10),
            row![
                std_text("avg"),
                std_text(sumscore / allokscoreslen.max(1) as i32)
            ]
            .spacing(10),
            row![
                button(std_text("recheck")).on_press(AppMessage::ReCheck),
                button(std_text("requery")).on_press(AppMessage::ReQuery),
            ]
            .spacing(10),
        ];
        let ratesheader = create_row("idx", "addr", "score", "state");
        let ratesbody = scrollable(Column::with_children(
            self.ratelist
                .rates
                .iter()
                .enumerate()
                .map(|(i, r)| {
                    create_row(
                        i.to_string(),
                        r.addr.to_string(),
                        r.score.to_string(),
                        match r.state() {
                            rate_list::State::Ok => "Ok",
                            rate_list::State::Ready => "Ready",
                            rate_list::State::Error => "Error",
                        }
                        .to_string(),
                    )
                })
                .collect(),
        ));
        let rates = column!(
            row![
                available_list,
                button(std_text("open")).on_press(AppMessage::OpenSerial)
            ]
            .spacing(10),
            ratesheader,
            ratesbody
        )
        .spacing(5)
        .align_items(alignment::Alignment::Center);
        container(row![rates, displayscores,].spacing(5))
            .center_x()
            .center_y()
            .width(Length::Shrink)
            .height(Length::Shrink)
            .padding([5, 5, 5, 5])
            .into()

        // row![
        //     button("Increment").on_press(AppMessage::IncrementPressed),
        //     text(self.value).size(50),
        //     button("Decrement").on_press(AppMessage::DecrementPressed)
        // ]
        // .padding(20)
        // .align_items(Alignment::Center)
        // .into()
    }
}

fn add_boder<'a>(c: impl Into<Element<'a, AppMessage>>) -> Element<'a, AppMessage> {
    container(c)
        .padding(2)
        .style(iced::theme::Container::Custom(Box::new(MyContainerStyle)))
        .into()
}
fn std_text<'a>(t: impl ToString) -> Element<'a, AppMessage> {
    text(t)
        .width(80)
        .height(30)
        .horizontal_alignment(alignment::Horizontal::Center)
        .vertical_alignment(alignment::Vertical::Center)
        .into()
}
fn create_row<'a, T: ToString>(idx: T, addr: T, score: T, state: T) -> Element<'a, AppMessage> {
    add_boder(row![
        std_text(idx),
        std_text(addr),
        std_text(score),
        std_text(state),
    ])
}

use iced::Background;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Custom {
    #[default]
    Default,
}
impl Custom {
    pub fn default() -> Self {
        Self::default()
    }
}
impl container::StyleSheet for Custom {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            /// The text [`Color`] of the container.
            // pub text_color: Option<Color>,
            /// The [`Background`] of the container.
            // pub background: Option<Background>,
            /// The border radius of the container.
            /// The border width of the container.
            /// The border [`Color`] of the container.
            // border_color: Color::,
            border_radius: 20.0,
            border_width: 20.0,
            border_color: Color::TRANSPARENT,
            ..Default::default()
        }
    }
}
#[derive(Default)]
struct ContainerExample;

#[derive(Default)]
struct MyContainerStyle;

impl iced::widget::container::StyleSheet for MyContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            border_radius: 1.0,
            border_width: 1.0,
            // background: Some(iced::Background::Color(iced::Color::BLACK)),
            border_color: iced::Color::BLACK,
            // text_color: Some(Color::WHITE),
            ..Default::default()
        }
    }
}
