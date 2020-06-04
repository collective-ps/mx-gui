use std::path::PathBuf;

use iced::{Element, Text};

#[derive(Debug)]
pub enum FileState {
  Analyzing,
  Pending,
  Uploading,
  Succeeded,
  Failed,
}

impl Default for FileState {
  fn default() -> Self {
    FileState::Analyzing
  }
}

#[derive(Debug)]
pub struct File {
  pub path: PathBuf,
  pub state: FileState,
}

#[derive(Debug, Clone)]
pub enum FileMessage {
  Remove,
}

impl File {
  pub fn view(&mut self) -> Element<FileMessage> {
    Text::new(format!("{}", self.path.display()))
      .size(14)
      .into()
  }
}
