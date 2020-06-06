#[derive(Debug)]
pub enum Scenes {
  Index,
  FileIndex,
}

impl Default for Scenes {
  fn default() -> Self {
    Self::Index
  }
}
