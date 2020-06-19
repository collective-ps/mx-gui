use crate::api::{Config, User};
use crate::scenes::WelcomeMessage;
use crate::widgets::file::{AnalyzeResult, FileMessage, FileState};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Filter {
  Pending,
  Duplicate,
  Completed,
  Failed,
  Queued,
}

impl Filter {
  pub fn states(&self) -> Vec<FileState> {
    match *self {
      Filter::Pending => vec![
        FileState::Analyzing,
        FileState::Analyzed,
        FileState::Pending,
        FileState::CheckingDuplicate,
      ],
      Filter::Duplicate => vec![FileState::Duplicate],
      Filter::Completed => vec![FileState::Completed],
      Filter::Failed => vec![FileState::Failed],
      Filter::Queued => vec![FileState::Queued, FileState::Uploading],
    }
  }
}

impl Default for Filter {
  fn default() -> Self {
    Self::Pending
  }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Message {
  EventOccurred(iced_native::Event),
  FileAnalyzed(u64, AnalyzeResult),
  FileMessage(u64, FileMessage),
  WelcomeMessage(WelcomeMessage),
  SetConfigAndUser(Config, User),
  DuplicateCheckResponse(Vec<String>, Vec<String>),
  SetFilter(Filter),
  SelectFile(usize),
  SetTags(String),
  Enqueue,
  StartUpload,
  SuccessfulUpload(u64),
  FailedUpload(u64),
  BeginUploadBatch,
  Noop,
}
