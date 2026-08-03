use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

pub const INVESTMENT_DISCLAIMER: &str = "This is not financial advice. Past performance does not guarantee future results. InvestIQ AI does not provide guaranteed returns. Markets involve risk of loss. Always do your own research and consult a licensed advisor when needed.";

const SYSTEM_PROMPT: &str = r#"You are InvestIQ AI, an investment education and analysis assistant.

Rules you MUST follow:
1. Never promise, guarantee, or imply guaranteed returns or risk-free profits.
2. Never tell the user to "definitely buy" or "definitely sell" as a certainty.
3. Grey Market Premium (GMP) is unofficial — always say so if discussed.
4. Prefer explaining risks, uncertainties, and data limitations.
5. When reviewing portfolios or trades, be constructive and educational.
6. End material recommendations-style answers with a short reminder that this is not financial advice.
7. If context data is missing, say so instead of inventing holdings or prices.
"#;

#[derive(Clone)]
pub struct AiClient {
    base_url: String,
    api_key: Option<String>,
    model: String,
    http: reqwest::Client,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatMessage,
}

impl AiClient {
    pub fn new(base_url: String, api_key: Option<String>, model: String) -> Self {
        Self {
            base_url,
            api_key,
            model,
            http: reqwest::Client::new(),
        }
    }

    pub async fn chat(&self, user_messages: Vec<ChatMessage>, grounded_context: Option<String>) -> AppResult<String> {
        let mut messages = vec![ChatMessage {
            role: "system".into(),
            content: SYSTEM_PROMPT.into(),
        }];

        if let Some(ctx) = grounded_context {
            messages.push(ChatMessage {
                role: "system".into(),
                content: format!("Grounded application context (authoritative):\n{ctx}"),
            });
        }

        messages.extend(user_messages);

        // Offline / no key: deterministic educational stub
        if self.api_key.as_ref().map(|k| k.is_empty()).unwrap_or(true) {
            return Ok(stub_reply(&messages));
        }

        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let body = ChatRequest {
            model: self.model.clone(),
            messages,
            temperature: 0.4,
        };

        let res = self
            .http
            .post(url)
            .bearer_auth(self.api_key.as_ref().unwrap())
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("ai http: {e}")))?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!("ai status {status}: {text}")));
        }

        let parsed: ChatResponse = res
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("ai parse: {e}")))?;

        let content = parsed
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_else(|| "I could not generate a response.".into());

        Ok(sanitize_reply(&content))
    }
}

fn sanitize_reply(s: &str) -> String {
    // Soft guard against prohibited absolute-return language
    let lowered = s.to_lowercase();
    if lowered.contains("guaranteed returns") || lowered.contains("risk-free profit") {
        return format!(
            "{s}\n\nNote: No investment outcome is guaranteed. {INVESTMENT_DISCLAIMER}"
        );
    }
    s.to_string()
}

fn stub_reply(messages: &[ChatMessage]) -> String {
    let last_user = messages
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .map(|m| m.content.as_str())
        .unwrap_or("");

    format!(
        "*(AI provider not configured — educational stub)*\n\n\
         You asked: \"{last_user}\"\n\n\
         I can help summarize IPOs, explain financial statements at a high level, \
         review portfolio concentration, and highlight journal patterns — always as education, \
         never as a guarantee of returns.\n\n\
         {INVESTMENT_DISCLAIMER}"
    )
}
