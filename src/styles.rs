use iced::{button, container, text_input, Background, Color, Text, Vector};

const SURFACE: Color = Color::from_rgb(
  0x21 as f32 / 255.0,
  0x21 as f32 / 255.0,
  0x21 as f32 / 255.0,
);

const ACTIVE: Color = Color::from_rgb(
  0x30 as f32 / 255.0,
  0x30 as f32 / 255.0,
  0x30 as f32 / 255.0,
);

const HOVERED: Color = Color::from_rgb(
  0x42 as f32 / 255.0,
  0x42 as f32 / 255.0,
  0x42 as f32 / 255.0,
);

pub struct Container {
  pub hovered: bool,
}

impl container::StyleSheet for Container {
  fn style(&self) -> container::Style {
    if self.hovered {
      container::Style {
        background: Some(Background::Color(ACTIVE)),
        ..container::Style::default()
      }
    } else {
      container::Style {
        background: Some(Background::Color(SURFACE)),
        ..container::Style::default()
      }
    }
  }
}

pub enum Button {
  Primary,
}

impl button::StyleSheet for Button {
  fn active(&self) -> button::Style {
    button::Style {
      background: Some(Background::Color(match self {
        Button::Primary => ACTIVE,
      })),
      border_radius: 2,
      text_color: Color::WHITE,
      ..button::Style::default()
    }
  }

  fn hovered(&self) -> button::Style {
    button::Style {
      background: Some(Background::Color(match self {
        Button::Primary => HOVERED,
      })),
      ..self.active()
    }
  }
}

pub enum TextInput {
  Primary,
}

impl text_input::StyleSheet for TextInput {
  fn active(&self) -> text_input::Style {
    text_input::Style {
      background: Background::Color(ACTIVE),
      ..text_input::Style::default()
    }
  }

  fn focused(&self) -> text_input::Style {
    text_input::Style {
      border_color: Color::from_rgb(0.5, 0.5, 0.5),
      ..self.active()
    }
  }

  fn placeholder_color(&self) -> Color {
    Color::from_rgb(0.3, 0.3, 0.3)
  }

  fn value_color(&self) -> Color {
    Color::from_rgb(0.7, 0.7, 0.7)
  }

  fn selection_color(&self) -> Color {
    Color::from_rgb(0.8, 0.8, 1.0)
  }
}
