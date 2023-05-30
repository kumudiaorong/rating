use iced::widget::{container, text};
use iced::{executor, Length};
use iced::{Application, Color, Command, Element, Settings};

#[derive(Default)]
struct ContainerExample;

#[derive(Debug, Clone, Copy)]
enum Message {}

#[derive(Default)]
struct MyContainerStyle;

impl iced::widget::container::StyleSheet for MyContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(iced::Color::BLACK)),
            text_color: Some(Color::WHITE),
            ..Default::default()
        }
    }
}

impl Application for ContainerExample {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    type Theme = iced::Theme;

    fn new(_flags: ()) -> (ContainerExample, Command<Self::Message>) {
        (ContainerExample::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Container Style Example")
    }

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let hello = text("Hello world").size(40.);
        container(hello)
            .style(iced::theme::Container::Custom(Box::new(MyContainerStyle)))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

pub fn main() -> iced::Result {
    ContainerExample::run(Settings::default())
}
