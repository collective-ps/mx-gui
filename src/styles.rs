use iced::{container, Background, Color};

pub struct Container {
  pub hovered: bool,
}

impl container::StyleSheet for Container {
  fn style(&self) -> container::Style {
    if self.hovered {
      container::Style {
        background: Some(Background::Color(Color::from_rgb8(0x36, 0x39, 0x3F))),
        ..container::Style::default()
      }
    } else {
      container::Style::default()
    }
  }
}
