use crate::http_utils::create_ai_http_client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug)]
pub enum AiClientError {
    RequestError(String),
    HttpError { status_code: u16, message: String },
    ParseError(String),
    ConfigError(String),
}

impl fmt::Display for AiClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AiClientError::RequestError(msg) => write!(f, "Request error: {}", msg),
            AiClientError::HttpError {
                status_code,
                message,
            } => write!(f, "HTTP {} error: {}", status_code, message),
            AiClientError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            AiClientError::ConfigError(msg) => write!(f, "Config error: {}", msg),
        }
    }
}

impl Error for AiClientError {}

#[derive(Debug, Clone, Serialize)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        RetryConfig {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AiProvider {
    Ollama { base_url: String, model: String },
    Anthropic { api_key: String, model: String },
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    stream: bool,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: i32,
    temperature: f32,
    messages: Vec<AnthropicMessage>,
}

#[derive(Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
}

#[derive(Serialize, Deserialize)]
struct AnthropicContent {
    text: String,
}

pub struct AiClient {
    provider: AiProvider,
    client: reqwest::Client,
    retry_config: RetryConfig,
}

impl AiClient {
    pub fn new(provider: AiProvider) -> Result<Self, AiClientError> {
        Self::new_with_retry_config(provider, RetryConfig::default())
    }

    pub fn new_with_retry_config(
        provider: AiProvider,
        retry_config: RetryConfig,
    ) -> Result<Self, AiClientError> {
        let client = create_ai_http_client()
            .map_err(|e| AiClientError::RequestError(e.to_string()))?;
        Ok(AiClient {
            provider,
            client,
            retry_config,
        })
    }

    pub async fn generate_text(&self, prompt: &str) -> Result<String, AiClientError> {
        match &self.provider {
            AiProvider::Ollama { base_url, model } => {
                self.call_ollama(base_url, model, prompt).await
            }
            AiProvider::Anthropic { api_key, model } => {
                self.call_anthropic_with_retry(api_key, model, prompt).await
            }
        }
    }

    fn is_retryable_error(&self, status_code: u16) -> bool {
        match status_code {
            429 | 500 | 502 | 503 | 504 | 529 => true, // Rate limit, server errors, overloaded
            _ => false,
        }
    }

    async fn call_anthropic_with_retry(
        &self,
        api_key: &str,
        model: &str,
        prompt: &str,
    ) -> Result<String, AiClientError> {
        let mut delay_ms = self.retry_config.initial_delay_ms;
        let mut last_error = None;

        println!("Starting Anthropic API call with retry config: max_retries={}, initial_delay_ms={}, max_delay_ms={}, backoff_multiplier={}", 
               self.retry_config.max_retries, self.retry_config.initial_delay_ms, 
               self.retry_config.max_delay_ms, self.retry_config.backoff_multiplier);

        for attempt in 0..=self.retry_config.max_retries {
            println!(
                "Anthropic API attempt {} of {}",
                attempt + 1,
                self.retry_config.max_retries + 1
            );

            match self.call_anthropic(api_key, model, prompt).await {
                Ok(result) => {
                    if attempt > 0 {
                        println!(
                            "Anthropic API call succeeded on attempt {} after {} previous failures",
                            attempt + 1,
                            attempt
                        );
                    } else {
                        println!("Anthropic API call succeeded on first attempt");
                    }
                    return Ok(result);
                }
                Err(err) => {
                    last_error = Some(err);

                    // Log the error details
                    match &last_error {
                        Some(AiClientError::HttpError {
                            status_code,
                            message,
                        }) => {
                            println!(
                                "Anthropic API attempt {} failed with HTTP {}: {}",
                                attempt + 1,
                                status_code,
                                message
                            );
                        }
                        Some(other_err) => {
                            println!(
                                "Anthropic API attempt {} failed with error: {}",
                                attempt + 1,
                                other_err
                            );
                        }
                        None => {}
                    }

                    // Don't retry on the last attempt
                    if attempt == self.retry_config.max_retries {
                        println!(
                            "Anthropic API exhausted all {} attempts, giving up",
                            self.retry_config.max_retries + 1
                        );
                        break;
                    }

                    // Check if error is retryable
                    let should_retry = match &last_error {
                        Some(AiClientError::HttpError { status_code, .. }) => {
                            let retryable = self.is_retryable_error(*status_code);
                            if retryable {
                                println!(
                                    "HTTP {} is retryable, will retry after backoff",
                                    status_code
                                );
                            } else {
                                println!("HTTP {} is not retryable, giving up", status_code);
                            }
                            retryable
                        }
                        _ => {
                            println!("Error type is not retryable, giving up");
                            false
                        }
                    };

                    if !should_retry {
                        break;
                    }

                    // Sleep with exponential backoff
                    println!(
                        "Backing off for {}ms before retry attempt {} (backoff multiplier: {})",
                        delay_ms,
                        attempt + 2,
                        self.retry_config.backoff_multiplier
                    );
                    sleep(Duration::from_millis(delay_ms)).await;

                    let next_delay_ms = ((delay_ms as f64 * self.retry_config.backoff_multiplier)
                        as u64)
                        .min(self.retry_config.max_delay_ms);

                    println!(
                        "Next backoff delay: {}ms (capped at {}ms)",
                        next_delay_ms, self.retry_config.max_delay_ms
                    );
                    delay_ms = next_delay_ms;
                }
            }
        }

        // Return the last error if all retries failed
        println!("All Anthropic API retry attempts failed");
        Err(last_error.unwrap_or_else(|| AiClientError::RequestError("All retry attempts failed with no recorded error".to_string())))
    }

    async fn call_ollama(
        &self,
        base_url: &str,
        model: &str,
        prompt: &str,
    ) -> Result<String, AiClientError> {
        let url = format!("{}/v1/chat/completions", base_url);
        let request = ChatCompletionRequest {
            model: model.to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: 0.0,
            stream: false,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AiClientError::RequestError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AiClientError::RequestError(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let chat_response: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| AiClientError::ParseError(e.to_string()))?;

        match chat_response.choices.first() {
            Some(choice) => Ok(choice.message.content.clone()),
            None => Err(AiClientError::ParseError(
                "No choices in response".to_string(),
            )),
        }
    }

    async fn call_anthropic(
        &self,
        api_key: &str,
        model: &str,
        prompt: &str,
    ) -> Result<String, AiClientError> {
        let url = "https://api.anthropic.com/v1/messages";
        let request = AnthropicRequest {
            model: model.to_string(),
            max_tokens: 20000,
            temperature: 0.0,
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        };

        let response = self
            .client
            .post(url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AiClientError::RequestError(e.to_string()))?;

        if !response.status().is_success() {
            let status_code = response.status().as_u16();
            let error_body = response.text().await.unwrap_or_default();
            return Err(AiClientError::HttpError {
                status_code,
                message: error_body,
            });
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| AiClientError::ParseError(e.to_string()))?;

        match anthropic_response.content.first() {
            Some(content) => Ok(content.text.clone()),
            None => Err(AiClientError::ParseError(
                "No content in response".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ollama_client_creation() {
        let provider = AiProvider::Ollama {
            base_url: "http://127.0.0.1:1234".to_string(),
            model: "llama2".to_string(),
        };

        let client = AiClient::new(provider);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_anthropic_client_creation() {
        let provider = AiProvider::Anthropic {
            api_key: "test-key".to_string(),
            model: "claude-3-sonnet-20240229".to_string(),
        };

        let client = AiClient::new(provider);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_anthropic_request_structure() {
        let request = AnthropicRequest {
            model: "claude-3-sonnet-20240229".to_string(),
            max_tokens: 4000,
            temperature: 0.0,
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: "Hello, world!".to_string(),
            }],
        };

        let json = serde_json::to_string_pretty(&request).unwrap();
        insta::assert_snapshot!(json);
    }

    #[tokio::test]
    async fn test_anthropic_response_parsing() {
        let response_json = r#"{
            "content": [
                {
                    "text": "Hello! How can I help you today?",
                    "type": "text"
                }
            ],
            "id": "msg_123",
            "model": "claude-3-sonnet-20240229", 
            "role": "assistant",
            "stop_reason": "end_turn",
            "stop_sequence": null,
            "type": "message",
            "usage": {
                "input_tokens": 10,
                "output_tokens": 10
            }
        }"#;

        let response: AnthropicResponse = serde_json::from_str(response_json).unwrap();
        insta::assert_json_snapshot!(response);
    }

    #[tokio::test]
    async fn test_retry_config_default() {
        let config = RetryConfig::default();
        insta::assert_json_snapshot!(config);
    }

    #[tokio::test]
    async fn test_retry_config_custom() {
        let config = RetryConfig {
            max_retries: 5,
            initial_delay_ms: 500,
            max_delay_ms: 60000,
            backoff_multiplier: 3.0,
        };
        insta::assert_json_snapshot!(config);
    }

    #[test]
    fn test_retryable_error_codes() {
        let provider = AiProvider::Anthropic {
            api_key: "test-key".to_string(),
            model: "claude-3-sonnet-20240229".to_string(),
        };
        let client = AiClient::new(provider).unwrap();

        // Test retryable status codes
        assert!(client.is_retryable_error(429)); // Rate limit
        assert!(client.is_retryable_error(500)); // Internal server error
        assert!(client.is_retryable_error(502)); // Bad gateway
        assert!(client.is_retryable_error(503)); // Service unavailable
        assert!(client.is_retryable_error(504)); // Gateway timeout
        assert!(client.is_retryable_error(529)); // Overloaded

        // Test non-retryable status codes
        assert!(!client.is_retryable_error(400)); // Bad request
        assert!(!client.is_retryable_error(401)); // Unauthorized
        assert!(!client.is_retryable_error(403)); // Forbidden
        assert!(!client.is_retryable_error(404)); // Not found

        let retryable_codes = vec![429u16, 500, 502, 503, 504, 529];
        let non_retryable_codes = vec![400u16, 401, 403, 404, 422];

        let test_result = (retryable_codes, non_retryable_codes);
        insta::assert_json_snapshot!(test_result);
    }

    #[tokio::test]
    async fn test_retry_logging_structure() {
        // Test the structure of retry-related log messages without actually making API calls
        let retry_config = RetryConfig {
            max_retries: 2,
            initial_delay_ms: 100,
            max_delay_ms: 1000,
            backoff_multiplier: 2.0,
        };

        // Simulate the log messages that would be generated during retry
        let log_messages = vec![
            format!("Starting Anthropic API call with retry config: max_retries={}, initial_delay_ms={}, max_delay_ms={}, backoff_multiplier={}", 
                   retry_config.max_retries, retry_config.initial_delay_ms, 
                   retry_config.max_delay_ms, retry_config.backoff_multiplier),
            "Anthropic API attempt 1 of 3".to_string(),
            "Anthropic API attempt 1 failed with HTTP 529: Service overloaded".to_string(),
            "HTTP 529 is retryable, will retry after backoff".to_string(),
            "Backing off for 100ms before retry attempt 2 (backoff multiplier: 2)".to_string(),
            "Next backoff delay: 200ms (capped at 1000ms)".to_string(),
            "Anthropic API attempt 2 of 3".to_string(),
            "Anthropic API attempt 2 failed with HTTP 529: Service overloaded".to_string(),
            "HTTP 529 is retryable, will retry after backoff".to_string(),
            "Backing off for 200ms before retry attempt 3 (backoff multiplier: 2)".to_string(),
            "Next backoff delay: 400ms (capped at 1000ms)".to_string(),
            "Anthropic API attempt 3 of 3".to_string(),
            "Anthropic API call succeeded on attempt 3 after 2 previous failures".to_string(),
        ];

        // Test scenarios for different outcomes
        let scenarios = vec![
            ("successful_after_retries", log_messages),
            ("exhausted_retries", vec![
                "Starting Anthropic API call with retry config: max_retries=2, initial_delay_ms=100, max_delay_ms=1000, backoff_multiplier=2".to_string(),
                "Anthropic API attempt 1 of 3".to_string(),
                "Anthropic API attempt 1 failed with HTTP 529: Service overloaded".to_string(),
                "Anthropic API attempt 2 of 3".to_string(),
                "Anthropic API attempt 2 failed with HTTP 529: Service overloaded".to_string(),
                "Anthropic API attempt 3 of 3".to_string(),
                "Anthropic API attempt 3 failed with HTTP 529: Service overloaded".to_string(),
                "Anthropic API exhausted all 3 attempts, giving up".to_string(),
                "All Anthropic API retry attempts failed".to_string(),
            ]),
            ("non_retryable_error", vec![
                "Starting Anthropic API call with retry config: max_retries=2, initial_delay_ms=100, max_delay_ms=1000, backoff_multiplier=2".to_string(),
                "Anthropic API attempt 1 of 3".to_string(),
                "Anthropic API attempt 1 failed with HTTP 401: Unauthorized".to_string(),
                "HTTP 401 is not retryable, giving up".to_string(),
                "All Anthropic API retry attempts failed".to_string(),
            ]),
        ];

        insta::assert_json_snapshot!(scenarios);
    }
}
