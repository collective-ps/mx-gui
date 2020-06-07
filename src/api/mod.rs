use reqwest::{Client, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

/// Configuration used for making API requests.
#[derive(Debug, Clone)]
pub struct Config {
  pub host: String,
  pub api_token: String,
}

impl Config {
  pub fn new(api_token: String) -> Self {
    Self {
      api_token,
      host: "https://spin-archive.org".to_owned(),
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

  #[error("Server is unreachable at this time")]
  ServerUnavailable,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
  pub id: i32,
  pub username: String,
  pub role: String,
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

    handle_response(response).await
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Checksums {
  pub checksums: Vec<String>,
}

impl Checksums {
  pub async fn check(checksums: &Vec<String>, config: &Config) -> Result<Self, ApiError> {
    let endpoint = format!("{}/api/v1/uploads/checksum", config.host);

    let response = Client::new()
      .post(&endpoint)
      .header("content-type", "application/json")
      .header("authorization", format!("Bearer {}", config.api_token))
      .json(&json!({ "checksums": checksums }))
      .send()
      .await;

    handle_response(response).await
  }
}

async fn handle_response<T: DeserializeOwned>(
  response: Result<Response, reqwest::Error>,
) -> Result<T, ApiError> {
  match response {
    Ok(response) => match response.status() {
      StatusCode::FORBIDDEN => {
        return Err(ApiError::ApiKeyError);
      }
      status => {
        if status.is_success() {
          match response.json().await {
            Ok(json) => Ok(json),
            Err(_) => Err(ApiError::JsonError),
          }
        } else {
          Err(ApiError::NotFound)
        }
      }
    },
    Err(_) => Err(ApiError::ServerUnavailable),
  }
}
