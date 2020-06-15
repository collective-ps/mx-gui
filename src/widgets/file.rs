use std::path::PathBuf;

use iced::{
  button, text_input, Button, Column, Container, Element, Length, Row, VerticalAlignment,
};

use crate::message::Message;
use crate::styles;
use crate::FileSelection;

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum FileState {
  Analyzing,
  Analyzed,
  CheckingDuplicate,
  Pending,
  Uploading,
  Completed,
  Failed,
  Duplicate,
  Queued,
}

impl Default for FileState {
  fn default() -> Self {
    FileState::Analyzing
  }
}

impl std::fmt::Display for FileState {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match *self {
      FileState::Analyzing => write!(f, "Analyzing"),
      FileState::Analyzed => write!(f, "Analyzed"),
      FileState::CheckingDuplicate => write!(f, "Checking Duplicate"),
      FileState::Pending => write!(f, "Pending"),
      FileState::Uploading => write!(f, "Uploading"),
      FileState::Completed => write!(f, "Completed"),
      FileState::Failed => write!(f, "Failed"),
      FileState::Duplicate => write!(f, "Duplicate"),
      FileState::Queued => write!(f, "Queued"),
    }
  }
}

#[derive(Debug, Default)]
pub struct File {
  pub id: u64,
  pub path: PathBuf,
  pub state: FileState,
  pub md5: Option<md5::Digest>,
  pub tags: String,
  pub tag_input: text_input::State,
  pub button: button::State,
}

#[derive(Debug, Clone)]
pub enum FileMessage {
  Analyzed(FileAnalysis),
}

#[derive(Debug, Clone)]
pub struct FileAnalysis {
  pub id: u64,
  md5: md5::Digest,
}

#[derive(Debug, Clone)]
pub enum AnalyzeError {
  FileOpen,
  FileRead,
}

pub type AnalyzeResult = Result<FileAnalysis, AnalyzeError>;

pub fn file_index<'a>(
  file_selection: &FileSelection,
  files: Vec<&'a mut File>,
) -> Element<'a, Message> {
  let mut file_names = Column::new()
    .spacing(2)
    .push(styles::text("File Name").vertical_alignment(VerticalAlignment::Center));
  let mut status = Column::new()
    .spacing(2)
    .push(styles::text("Status").vertical_alignment(VerticalAlignment::Center));
  let mut md5 = Column::new()
    .spacing(2)
    .push(styles::text("MD5").vertical_alignment(VerticalAlignment::Center));
  let mut tags = Column::new()
    .spacing(2)
    .push(styles::text("Tags").vertical_alignment(VerticalAlignment::Center));

  for (idx, file) in files.into_iter().enumerate() {
    let file_md5 = file.get_md5();
    let file_name = file.truncated_file_name();
    let selected = match file_selection {
      FileSelection::None => false,
      FileSelection::Single(matching_idx) => idx == *matching_idx,
      FileSelection::Multiple(indices) => indices.contains(&idx),
    };

    file_names = file_names.push(
      Container::new(
        Button::new(&mut file.button, styles::text(file_name))
          .style(styles::Button::Transparent)
          .on_press(Message::SelectFile(idx))
          .padding(2),
      )
      .padding(0)
      .style(styles::HoveredContainer::new(selected)),
    );
    status = status.push(
      Container::new(styles::text(file.state.to_string()))
        .padding(2)
        .style(styles::HoveredContainer::new(selected)),
    );
    md5 = md5.push(
      Container::new(styles::text(file_md5))
        .padding(2)
        .style(styles::HoveredContainer::new(selected)),
    );
    tags = tags.push(
      Container::new(styles::text(&file.tags))
        .padding(2)
        .style(styles::HoveredContainer::new(selected)),
    );
  }

  let content = Row::new()
    .push(file_names)
    .push(status)
    .push(md5)
    .push(tags)
    .spacing(6)
    .width(Length::Fill);

  Container::new(content).width(Length::Fill).into()
}

impl File {
  fn truncated_file_name(&self) -> String {
    let truncation = 75;
    let name = self.path.file_name().unwrap().to_str().unwrap();
    let length = name.chars().count();

    if length > truncation {
      let mut file_name: String = name.chars().take(truncation).collect();
      file_name.push_str("...");
      file_name
    } else {
      name.to_owned()
    }
  }

  #[allow(dead_code)]
  pub fn file_name(&self) -> &str {
    self.path.file_name().unwrap().to_str().unwrap()
  }

  pub fn get_md5(&self) -> String {
    if self.md5.is_some() {
      format!("{:x}", self.md5.unwrap())
    } else {
      "Not calculated".to_string()
    }
  }

  pub fn update(&mut self, message: FileMessage) {
    match message {
      FileMessage::Analyzed(analysis) => {
        self.md5 = Some(analysis.md5);
        self.state = FileState::Analyzed;
      }
    }
  }

  pub async fn analyze_file(id: u64, path: PathBuf) -> AnalyzeResult {
    use tokio::fs::File;
    use tokio::prelude::*;

    let mut context = md5::Context::new();
    let mut buffer = [0u8; 4 * 1024];
    let mut file = File::open(path).await.map_err(|_| AnalyzeError::FileOpen)?;

    loop {
      let size = file
        .read(&mut buffer[..])
        .await
        .map_err(|_| AnalyzeError::FileRead)?;

      if size == 0 {
        break;
      }

      context.consume(&buffer[..size]);
    }

    let digest = context.compute();

    Ok(FileAnalysis { id, md5: digest })
  }
}
