use crate::widgets::file::{AnalyzeResult, FileMessage};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Message {
  EventOccurred(iced_native::Event),
  FileAnalyzed(u64, AnalyzeResult),
  ApiKeyInputChanged(String),
  FileMessage(u64, FileMessage),
  NextScene,
  Noop,
}
