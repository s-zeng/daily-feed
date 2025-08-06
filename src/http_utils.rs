//! HTTP utilities for the daily-feed application
//! Provides configured HTTP clients with proper timeouts and error handling

use reqwest::Client;
use std::time::Duration;

/// Default timeout for HTTP requests (30 seconds)
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Default connect timeout (10 seconds)
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// User agent string for all HTTP requests
const USER_AGENT: &str = "daily-feed/0.1.0";

/// Creates a configured HTTP client with appropriate timeouts
pub fn create_http_client() -> Result<Client, reqwest::Error> {
    Client::builder()
        .timeout(DEFAULT_TIMEOUT)
        .connect_timeout(DEFAULT_CONNECT_TIMEOUT)
        .user_agent(USER_AGENT)
        .build()
}

/// Creates a configured HTTP client with custom timeout
#[allow(dead_code)]
pub fn create_http_client_with_timeout(timeout: Duration) -> Result<Client, reqwest::Error> {
    Client::builder()
        .timeout(timeout)
        .connect_timeout(DEFAULT_CONNECT_TIMEOUT)
        .user_agent(USER_AGENT)
        .build()
}

/// Creates a configured HTTP client for AI operations with longer timeout
pub fn create_ai_http_client() -> Result<Client, reqwest::Error> {
    let ai_timeout = Duration::from_secs(120); // 2 minutes for AI operations
    Client::builder()
        .timeout(ai_timeout)
        .connect_timeout(DEFAULT_CONNECT_TIMEOUT)
        .user_agent(USER_AGENT)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_http_client() {
        let client = create_http_client();
        assert!(client.is_ok());
    }

    #[test]
    fn test_create_http_client_with_timeout() {
        let timeout = Duration::from_secs(60);
        let client = create_http_client_with_timeout(timeout);
        assert!(client.is_ok());
    }

    #[test]
    fn test_create_ai_http_client() {
        let client = create_ai_http_client();
        assert!(client.is_ok());
    }
}