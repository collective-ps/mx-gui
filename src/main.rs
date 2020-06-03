use std::path::PathBuf;

use iced::{
    executor, Align, Application, Column, Command, Container, Element, Length, Settings,
    Subscription, Text,
};

use iced_native::window::Event as WindowEvent;
use iced_native::Event;

pub fn main() {
    Events::run(Settings::default())
}

#[derive(Debug, Default)]
struct Events {
    last: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
enum Message {
    EventOccurred(iced_native::Event),
}

impl Application for Events {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Events, Command<Message>) {
        (Events::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("mx")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::EventOccurred(event) => {
                match event {
                    Event::Window(WindowEvent::FileDropped(path)) => {
                        self.last.push(path);
                    }
                    _ => {}
                }

                if self.last.len() > 5 {
                    let _ = self.last.remove(0);
                }
            }
        };

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced_native::subscription::events().map(Message::EventOccurred)
    }

    fn view(&mut self) -> Element<Message> {
        let events = self
            .last
            .iter()
            .fold(Column::new().spacing(10), |column, event| {
                column.push(Text::new(format!("{:?}", event)).size(40))
            });

        let content = Column::new()
            .align_items(Align::Center)
            .spacing(20)
            .push(events);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
