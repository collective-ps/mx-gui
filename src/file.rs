use std::path::PathBuf;

use iced::{Column, Container, Element, Length, Text};

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

#[derive(Debug, Default)]
pub struct File {
  pub id: u64,
  pub path: PathBuf,
  pub state: FileState,
  pub md5: Option<md5::Digest>,
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

impl File {
  pub fn view(&mut self) -> Element<FileMessage> {
    let content = if self.md5.is_some() {
      Column::new()
        .push(Text::new(format!("{}", self.path.display())).size(14))
        .push(Text::new(format!("Checksum: {:x}", self.md5.unwrap())).size(14))
    } else {
      Column::new()
        .push(Text::new(format!("{}", self.path.display())).size(14))
        .push(Text::new("Calculating MD5 checksum...").size(14))
    };

    Container::new(content).width(Length::Fill).into()
  }

  pub fn update(&mut self, message: FileMessage) {
    match message {
      FileMessage::Analyzed(analysis) => {
        self.md5 = Some(analysis.md5);
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
