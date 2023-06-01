use crate::core;
use iced::widget;
use iced::widget::scrollable::{Properties, Scrollbar, Scroller};
use iced::widget::{
    button, column, container, horizontal_space, pick_list, progress_bar, radio, row, scrollable,
    slider, text, vertical_space,
};
use iced::{alignment, executor, theme, Alignment, Color};
use iced::{subscription, Application, Command, Element, Length, Settings, Subscription, Theme};
#[derive(Debug, Clone)]
pub enum AppMessage {
    Selected(String),
    OpenSerial,
    // Receive(crate::msg::RateList),
    // AvailablePorts(Vec<String>),
    Tick,
}
use msg::MsgType;
use std::sync::mpsc;
pub struct App {
    sender: mpsc::Sender<Vec<u8>>,
    receiver: mpsc::Receiver<Vec<u8>>,
    choosed: Option<String>,
    available_ports: Vec<String>,
    ratelist: crate::msg::RateList,
    count: i32,
}
use crate::msg;
use msg::msg_type::Type;
use prost::Message;
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
            AppMessage::Selected(path) => {
                self.choosed = Some(path);
            }
            AppMessage::OpenSerial => {
                if let Some(ref path) = self.choosed {
                    self.sender
                        .send(msg::MsgType::new(msg::msg_type::Type::Open).encode_to_vec())
                        .unwrap();
                    self.sender
                        .send(msg::Port::new(path.clone()).encode_to_vec())
                        .unwrap();
                    self.sender
                        .send(msg::MsgType::new(msg::msg_type::Type::Right).encode_to_vec())
                        .unwrap();
                    self.sender
                        .send(msg::MsgType::new(msg::msg_type::Type::Query).encode_to_vec())
                        .unwrap();
                }
            }
            AppMessage::Tick => {
                self.available_ports = core::available_ports();
                match self.receiver.try_recv() {
                    Ok(rcev) => match MsgType::decode(rcev.as_slice()) {
                        Ok(tp) => match tp.r#type() {
                            Type::Check => (),
                            other => match other {
                                Type::Query => match self.receiver.recv() {
                                    Ok(rcev) => match msg::RateList::decode(rcev.as_slice()) {
                                        Ok(rl) => self.ratelist = rl,
                                        Err(_) => (),
                                    },
                                    Err(_) => (),
                                },
                                _ => (),
                            },
                        },
                        Err(_) => (),
                    },
                    Err(_) => (),
                }
            }
        }
        Command::none()
    }
    fn subscription(&self) -> Subscription<AppMessage> {
        iced::time::every(std::time::Duration::from_millis(100)).map(|_| AppMessage::Tick)
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
        .width(Length::Shrink)
        .center_x()
        .center_y();
        // container("tes").style(iced::theme::Container::Custom(Box::new(MyContainerStyle))).padding(2),

        container(row![
            // container("tes").style(theme::Container::Custom(Box::new(Container::default()))),
            container("tes")
                .style(iced::theme::Container::Custom(Box::new(MyContainerStyle)))
                .padding(2),
            column![
                row![
                    text("idx")
                        .width(100)
                        .horizontal_alignment(alignment::Horizontal::Center)
                        .vertical_alignment(alignment::Vertical::Center),
                    text("score")
                        .width(100)
                        .horizontal_alignment(alignment::Horizontal::Center)
                        .vertical_alignment(alignment::Vertical::Center)
                ],
                scrollable(
                    // text(self.choosed.clone().unwrap_or("".to_string())).width(Length::Fill),
                    widget::Column::with_children(
                        self.ratelist
                            .rates
                            .iter()
                            .map(|r| {
                                Element::from(row![
                                    text(r.idx)
                                        .width(100)
                                        .horizontal_alignment(alignment::Horizontal::Center)
                                        .vertical_alignment(alignment::Vertical::Center),
                                    text(r.state.as_str())
                                        .width(100)
                                        .horizontal_alignment(alignment::Horizontal::Center)
                                        .vertical_alignment(alignment::Vertical::Center)
                                ])
                            })
                            .collect::<Vec<Element<AppMessage>>>()
                    )
                    .width(Length::Shrink)
                    .align_items(Alignment::Center) // .padding([0, 0, 0, 0])
                                                    // .spacing(40),
                )
                .height(Length::Fill), // .vertical_scroll(Properties::default())
                                       // .into()
            ],
            available_list,
            button("open")
                .on_press(AppMessage::OpenSerial)
                .width(Length::Shrink)
        ])
        .center_x()
        .center_y()
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([5, 0, 0, 0])
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
