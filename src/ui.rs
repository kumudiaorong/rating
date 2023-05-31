use crate::core;
use iced::widget;
use iced::widget::scrollable::{Properties, Scrollbar, Scroller};
use iced::widget::{
    button, column, container, horizontal_space, pick_list, progress_bar, radio, row, scrollable,
    slider, text, vertical_space,
};
use iced::{alignment, executor, theme, Alignment, Color};
use iced::{Application, Command, Element, Length, Settings, Theme};
#[derive(Debug, Clone)]
pub enum Message {
    Selected(String),
    OpenSerial,
}
pub struct App {
    core: core::Core,
    choosed: Option<String>,
    available_ports: Vec<String>,
    count: i32,
}
impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                core: core::Core::new(),
                choosed: None,
                available_ports: core::available_ports(),
                count: 0,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Rating")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        self.count += 1;
        match message {
            Message::Selected(path) => {
                self.choosed = Some(path);
            }
            Message::OpenSerial => {
                if let Some(ref path) = self.choosed {
                    println!("open {}", path);
                    self.core.open(path);
                    println!("right {}", path);
                    self.core.right();
                    println!("query {}", path);
                    self.core.query();
                    println!("over {}", path);
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let available_list = container(
            pick_list(
                &self.available_ports,
                self.choosed.clone(),
                Message::Selected,
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
                        self.core
                            .ratidx
                            .iter()
                            .map(|(k, v)| Element::from(row![
                                text(k)
                                    .width(100)
                                    .horizontal_alignment(alignment::Horizontal::Center)
                                    .vertical_alignment(alignment::Vertical::Center),
                                match v {
                                    core::DevState::Right => text("ready"),
                                    core::DevState::Rate(s) => text(s),
                                }
                                .width(100)
                                .horizontal_alignment(alignment::Horizontal::Center)
                                .vertical_alignment(alignment::Vertical::Center),
                            ]))
                            .collect::<Vec<Element<Message>>>()
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
                .on_press(Message::OpenSerial)
                .width(Length::Shrink)
        ])
        .center_x()
        .center_y()
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([5, 0, 0, 0])
        .into()

        // row![
        //     button("Increment").on_press(Message::IncrementPressed),
        //     text(self.value).size(50),
        //     button("Decrement").on_press(Message::DecrementPressed)
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
