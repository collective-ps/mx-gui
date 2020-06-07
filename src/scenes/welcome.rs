use iced::{
  button, text_input, Align, Button, Color, Column, Command, Container, Element, Length, Text,
  TextInput,
};

use crate::api;
use crate::message::Message;
use crate::styles;

#[derive(Debug, Default)]
pub struct WelcomeScene {
  api_key: String,
  api_key_input: text_input::State,
  next_button: button::State,
  error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum WelcomeMessage {
  ApiKeyInputChanged(String),
  SetDisplayError(String),
  NextScene,
}

impl WelcomeScene {
  pub fn update(&mut self, message: WelcomeMessage) -> Command<Message> {
    match message {
      WelcomeMessage::ApiKeyInputChanged(new_value) => {
        self.api_key = new_value;
        self.error = None;
      }
      WelcomeMessage::SetDisplayError(error) => self.error = Some(error),
      WelcomeMessage::NextScene => {
        let api_key = self.api_key.clone();

        let cmd = Command::perform(
          async move {
            let config = api::Config::new(api_key);
            let response = api::User::get(&config).await;
            response
          },
          |resp| match resp {
            Ok(_) => Message::NextScene,
            Err(e) => Message::WelcomeMessage(WelcomeMessage::SetDisplayError(e.to_string())),
          },
        );

        return cmd;
      }
    };

    Command::none()
  }

  pub fn view(&mut self) -> Element<WelcomeMessage> {
    let mut welcome = Column::new()
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

    if let Some(error_msg) = self.error.as_ref() {
      welcome = welcome.push(Text::new(error_msg).color(Color::WHITE));
    }

    if self.api_key.len() > 5 {
      welcome = welcome.push(
        Button::new(&mut self.next_button, Text::new("Next"))
          .padding(8)
          .style(styles::Button::Primary)
          .on_press(WelcomeMessage::NextScene),
      );
    }

    let container = Container::new(welcome)
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
