use std::path::PathBuf;

use iced::{Column, Container, Element, Length, Row};

use crate::styles;

#[allow(dead_code)]
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

pub fn file_index<'a, I>(files: I) -> Element<'a, FileMessage>
where
  I: Iterator<Item = &'a File>,
{
  let mut file_names = Column::new().spacing(2).push(styles::text("File Name"));
  let mut md5 = Column::new().spacing(2).push(styles::text("MD5"));

  for file in files {
    file_names = file_names.push(styles::text(file.file_name()));
    md5 = md5.push(styles::text(file.get_md5()));
  }

  let content = Row::new().push(file_names).push(md5).width(Length::Fill);

  Container::new(content).width(Length::Fill).into()
}

impl File {
  fn file_name(&self) -> &str {
    self.path.file_name().unwrap().to_str().unwrap()
  }

  fn get_md5(&self) -> String {
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
