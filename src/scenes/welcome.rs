use iced::{
  button, text_input, Align, Button, Color, Column, Command, Container, Element, Length, Text,
  TextInput,
};

use crate::message::Message;
use crate::styles;

#[derive(Debug, Default)]
pub struct WelcomeScene {
  api_key: String,
  api_key_input: text_input::State,
  next_button: button::State,
}

#[derive(Debug, Clone)]
pub enum WelcomeMessage {
  ApiKeyInputChanged(String),
  NextScene,
}

impl WelcomeScene {
  pub fn update(&mut self, message: WelcomeMessage) -> Command<Message> {
    match message {
      WelcomeMessage::ApiKeyInputChanged(new_value) => self.api_key = new_value,
      WelcomeMessage::NextScene => return Command::perform(async {}, |_| Message::NextScene),
    };

    Command::none()
  }

  pub fn view(&mut self) -> Element<WelcomeMessage> {
    let mut button = Column::new()
      .push(Text::new("spin-archive.org - MX").color(Color::WHITE))
      .push(
        TextInput::new(
          &mut self.api_key_input,
          "API key",
          &self.api_key,
          WelcomeMessage::ApiKeyInputChanged,
        )
        .style(styles::TextInput::Primary)
        .padding(8),
      )
      .max_width(300)
      .spacing(12)
      .align_items(Align::Center);

    if self.api_key.len() > 5 {
      button = button.push(
        Button::new(&mut self.next_button, Text::new("Next"))
          .padding(8)
          .style(styles::Button::Primary)
          .on_press(WelcomeMessage::NextScene),
      );
    }

    let container = Container::new(button)
      .width(Length::Fill)
      .height(Length::Fill)
      .center_x()
      .center_y();

    Container::new(container)
      .width(Length::Fill)
      .height(Length::Fill)
      .style(styles::Container { hovered: false })
      .into()
  }
}
