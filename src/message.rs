use crate::widgets::file::{AnalyzeResult, FileMessage};

#[derive(Debug, Clone)]
pub enum Message {
  EventOccurred(iced_native::Event),
  FileAnalyzed(u64, AnalyzeResult),
  ApiKeyInputChanged(String),
  NextScene,
  Noop,
}
