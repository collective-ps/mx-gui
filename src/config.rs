use tokio::fs;

#[tokio::main]
pub async fn read_api_key() -> Result<String, anyhow::Error> {
  let key = fs::read_to_string("spin-archive.key").await?;
  Ok(key)
}

#[tokio::main]
pub async fn write_api_key(key: &str) -> Result<(), anyhow::Error> {
  fs::write("spin-archive.key", &key).await?;
  Ok(())
}
