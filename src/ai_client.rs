use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AiClientError {
    RequestError(String),
    ParseError(String),
    ConfigError(String),
}

impl fmt::Display for AiClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AiClientError::RequestError(msg) => write!(f, "Request error: {}", msg),
            AiClientError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            AiClientError::ConfigError(msg) => write!(f, "Config error: {}", msg),
        }
    }
}

impl Error for AiClientError {}

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

pub struct AiClient {
    provider: AiProvider,
    client: reqwest::Client,
}

impl AiClient {
    pub fn new(provider: AiProvider) -> Result<Self, AiClientError> {
        let client = reqwest::Client::new();
        Ok(AiClient { provider, client })
    }

    pub async fn generate_text(&self, prompt: &str) -> Result<String, AiClientError> {
        match &self.provider {
            AiProvider::Ollama { base_url, model } => {
                self.call_ollama(base_url, model, prompt).await
            }
            AiProvider::Anthropic { api_key, model } => {
                self.call_anthropic(api_key, model, prompt).await
            }
        }
    }

    async fn call_ollama(&self, base_url: &str, model: &str, prompt: &str) -> Result<String, AiClientError> {
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
            None => Err(AiClientError::ParseError("No choices in response".to_string())),
        }
    }

    async fn call_anthropic(&self, _api_key: &str, _model: &str, _prompt: &str) -> Result<String, AiClientError> {
        // TODO: Implement Anthropic API integration
        // For now, return an error indicating it's not implemented
        Err(AiClientError::ConfigError("Anthropic API integration not yet implemented".to_string()))
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
}