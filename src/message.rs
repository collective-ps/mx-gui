use crate::scenes::WelcomeMessage;
use crate::widgets::file::{AnalyzeResult, FileMessage};

#[derive(Debug, Clone)]
pub enum Message {
  EventOccurred(iced_native::Event),
  FileAnalyzed(u64, AnalyzeResult),
  FileMessage(u64, FileMessage),
  WelcomeMessage(WelcomeMessage),
  NextScene,
  Noop,
}
