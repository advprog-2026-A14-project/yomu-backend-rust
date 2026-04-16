use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Shared HTTP client for making calls to Java Core service
#[derive(Clone)]
pub struct HttpClient {
    inner: Client,
    base_url: String,
    api_key: String,
}

impl HttpClient {
    /// Create a new HTTP client
    #[must_use]
    #[allow(clippy::expect_used)]
    pub fn new(base_url: String, api_key: String) -> Self {
        let inner = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client - this is unrecoverable at startup");

        Self {
            inner,
            base_url,
            api_key,
        }
    }

    /// Get the base URL
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Make a GET request with API key header
    pub async fn get(&self, path: &str) -> Result<String, HttpError> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .inner
            .get(&url)
            .header("x-api-key", &self.api_key)
            .send()
            .await
            .map_err(|e| HttpError::Request(e.to_string()))?;

        if !response.status().is_success() {
            return Err(HttpError::Status(response.status().as_u16()));
        }

        response
            .text()
            .await
            .map_err(|e| HttpError::Body(e.to_string()))
    }

    /// Make a POST request with API key header and JSON body
    pub async fn post<T: Serialize>(&self, path: &str, body: &T) -> Result<String, HttpError> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .inner
            .post(&url)
            .header("x-api-key", &self.api_key)
            .json(body)
            .send()
            .await
            .map_err(|e| HttpError::Request(e.to_string()))?;

        if !response.status().is_success() {
            return Err(HttpError::Status(response.status().as_u16()));
        }

        response
            .text()
            .await
            .map_err(|e| HttpError::Body(e.to_string()))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum HttpError {
    Request(String),
    Status(u16),
    Body(String),
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpError::Request(msg) => write!(f, "HTTP request failed: {msg}"),
            HttpError::Status(code) => {
                write!(f, "HTTP status {code}: {}", status_message(*code))
            }
            HttpError::Body(msg) => write!(f, "Failed to read response body: {msg}"),
        }
    }
}

impl std::error::Error for HttpError {}

fn status_message(code: u16) -> &'static str {
    match code {
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        409 => "Conflict",
        500 => "Internal Server Error",
        _ => "Unknown Error",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_message_returns_known_messages() {
        assert_eq!(status_message(400), "Bad Request");
        assert_eq!(status_message(401), "Unauthorized");
        assert_eq!(status_message(404), "Not Found");
        assert_eq!(status_message(500), "Internal Server Error");
        assert_eq!(status_message(999), "Unknown Error");
    }

    #[test]
    fn http_error_display() {
        let err = HttpError::Request("connection refused".to_string());
        assert!(err.to_string().contains("HTTP request failed"));
        assert!(err.to_string().contains("connection refused"));

        let err = HttpError::Status(404);
        assert!(err.to_string().contains("404"));
        assert!(err.to_string().contains("Not Found"));

        let err = HttpError::Body("invalid json".to_string());
        assert!(err.to_string().contains("Failed to read response body"));
        assert!(err.to_string().contains("invalid json"));
    }

    #[test]
    fn http_client_new_creates_valid_client() {
        let client = HttpClient::new("http://localhost:8080".to_string(), "test-key".to_string());
        assert_eq!(client.base_url(), "http://localhost:8080");
    }
}
