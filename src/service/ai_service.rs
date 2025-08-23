use serde::{Deserialize, Serialize};
use crate::error::AppError;
use reqwest::Client;

pub struct AiService {
    pub api_url: String,
    pub api_key: String,
    pub client: Client,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AIPaylod{
    pub messages: Vec<Message>,
    pub model: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}
#[derive(Debug, Deserialize)]
struct AiResponse {
    choices: Vec<Choice>,  // 对应 JSON 中的 "choices" 数组
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: AiMessage,    // 对应 "choices[0].message"
}

#[derive(Debug, Deserialize)]
struct AiMessage {
    content: String,       // 对应 "choices[0].message.content"
}
impl AiService {
    pub fn new(api_url: String, api_key: String) -> Self {
        Self {
            api_url,
            api_key,
            client: Client::new(),
        }
    }

    pub async fn analyze_report(&self, request: AIPaylod) -> Result<String, AppError> {
        let response = self.client
            .post(&self.api_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::InternalServerError(format!("AI API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::InternalServerError(format!(
                "AI API returned error: {}",
                response.status()
            )));
        }

        // response.text().await.map_err(|e| AppError::InternalServerError(format!("Failed to read AI API response: {}", e)))
        let ai_response: AiResponse = response.json()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to parse AI response: {}", e)))?;

        // 提取第一个选择的 content
        ai_response.choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| AppError::InternalServerError("AI response has no choices".to_string()))
    }
}