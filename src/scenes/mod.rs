mod welcome;

pub use welcome::{WelcomeMessage, WelcomeScene};

#[derive(Debug)]
pub enum Scenes {
  Welcome,
  FileIndex,
}

impl Default for Scenes {
  fn default() -> Self {
    Self::Welcome
  }
}
