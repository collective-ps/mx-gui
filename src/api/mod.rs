use std::path::PathBuf;
use std::time::Duration;

use reqwest::{Body, Client, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;
use tokio::fs::File;
use tokio::time::delay_for;
use tokio_util::codec::{BytesCodec, FramedRead};

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

fn client() -> Client {
  Client::builder().pool_max_idle_per_host(5).build().unwrap()
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

    let response = client()
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

    let response = client()
      .post(&endpoint)
      .header("content-type", "application/json")
      .header("authorization", format!("Bearer {}", config.api_token))
      .json(&json!({ "checksums": checksums }))
      .send()
      .await;

    handle_response(response).await
  }
}

#[derive(Deserialize, Debug)]
pub struct Upload {
  pub id: String,
  pub url: String,
}

impl Upload {
  pub async fn new(config: &Config, path: &PathBuf, md5_hash: &str) -> Result<Self, ApiError> {
    let endpoint = format!("{}/api/v1/uploads", config.host);

    let metadata = std::fs::metadata(&path).unwrap();
    let file_name = path.file_name().unwrap().to_str().unwrap();
    let file_size = metadata.len() as i64;

    let _ = delay_for(Duration::from_millis(100)).await;

    let new_upload_request = json!({
      "file_name": file_name,
      "content_length": file_size,
      "md5_hash": md5_hash,
    });

    let response = client()
      .post(&endpoint)
      .header("content-type", "application/json")
      .header("authorization", format!("Bearer {}", config.api_token))
      .json(&new_upload_request)
      .send()
      .await;

    handle_response(response).await
  }

  pub async fn upload_file(path: &PathBuf, url: &str) -> Result<(), ApiError> {
    let file = File::open(path).await.unwrap();
    let stream = FramedRead::new(file, BytesCodec::new());
    let body = Body::wrap_stream(stream);

    let _ = delay_for(Duration::from_millis(100)).await;

    let response = client().put(url).body(body).send().await;

    response
      .map_err(|_| ApiError::ServerUnavailable)
      .map(|_| ())
  }

  pub async fn finalize(
    config: &Config,
    id: &str,
    tags: &str,
    source: &str,
    description: &str,
  ) -> Result<Self, ApiError> {
    let endpoint = format!("{}/api/v1/uploads/finalize", config.host);

    let finalize_request = json!({
      "id": id,
      "tags": tags,
      "source": source,
      "description": description,
    });

    let _ = delay_for(Duration::from_millis(100)).await;

    let response = client()
      .post(&endpoint)
      .header("content-type", "application/json")
      .header("authorization", format!("Bearer {}", config.api_token))
      .json(&finalize_request)
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
        dbg!(&status);

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
