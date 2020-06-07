use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Configuration used for making API requests.
#[derive(Debug)]
pub struct Config {
  pub host: String,
  pub api_token: String,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      host: "https://spin-archive.org".to_owned(),
      ..Default::default()
    }
  }
}

#[derive(Error, Debug)]
pub enum ApiError {
  #[error("Resource was not found")]
  NotFound,

  #[error("API key was not valid")]
  ApiKeyError,

  #[error("Error decoding JSON")]
  JsonError,
}

#[derive(Serialize, Deserialize)]
pub struct User {
  id: i32,
  username: String,
  role: String,
}

impl User {
  pub async fn get(config: &Config) -> Result<Self, ApiError> {
    let endpoint = format!("{}/api/v1/me", config.host);

    let response = Client::new()
      .get(&endpoint)
      .header("content-type", "application/json")
      .header("authorization", format!("Bearer {}", config.api_token))
      .send()
      .await;

    match response {
      Ok(response) => match response.json().await {
        Ok(json) => Ok(json),
        Err(_) => Err(ApiError::JsonError),
      },
      Err(error) => {
        if let Some(status) = error.status() {
          match status {
            StatusCode::FORBIDDEN => Err(ApiError::ApiKeyError),
            _ => Err(ApiError::NotFound),
          }
        } else {
          Err(ApiError::NotFound)
        }
      }
    }
  }
}
