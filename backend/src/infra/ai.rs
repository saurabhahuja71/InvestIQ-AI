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
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }

    pub fn has_remote(&self) -> bool {
        self.api_key
            .as_ref()
            .map(|k| !k.trim().is_empty())
            .unwrap_or(false)
    }

    pub async fn chat(
        &self,
        user_messages: Vec<ChatMessage>,
        grounded_context: Option<String>,
    ) -> AppResult<String> {
        let mut messages = vec![ChatMessage {
            role: "system".into(),
            content: SYSTEM_PROMPT.into(),
        }];

        if let Some(ctx) = grounded_context.clone() {
            messages.push(ChatMessage {
                role: "system".into(),
                content: format!("Grounded application context (authoritative):\n{ctx}"),
            });
        }

        messages.extend(user_messages);

        if !self.has_remote() {
            return Ok(local_engine_reply(&messages, grounded_context.as_deref()));
        }

        let api_key = self
            .api_key
            .as_ref()
            .filter(|k| !k.trim().is_empty())
            .ok_or_else(|| AppError::Internal("AI API key missing".into()))?;

        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let body = ChatRequest {
            model: self.model.clone(),
            messages: messages.clone(),
            temperature: 0.4,
        };

        let res = self
            .http
            .post(&url)
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await;

        match res {
            Ok(res) if res.status().is_success() => {
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
            Ok(res) => {
                let status = res.status();
                let text = res.text().await.unwrap_or_default();
                tracing::warn!(%status, body = %text, "remote AI failed; falling back to local engine");
                Ok(local_engine_reply(&messages, grounded_context.as_deref()))
            }
            Err(e) => {
                tracing::warn!(error = %e, "remote AI network error; falling back to local engine");
                Ok(local_engine_reply(&messages, grounded_context.as_deref()))
            }
        }
    }
}

fn sanitize_reply(s: &str) -> String {
    let lowered = s.to_lowercase();
    if lowered.contains("guaranteed returns") || lowered.contains("risk-free profit") {
        return format!(
            "{s}\n\nNote: No investment outcome is guaranteed. {INVESTMENT_DISCLAIMER}"
        );
    }
    s.to_string()
}

/// On-device educational engine when no LLM key is configured.
/// Uses grounded context and keyword routing — always educational, never guarantees.
fn local_engine_reply(messages: &[ChatMessage], grounded: Option<&str>) -> String {
    let last_user = messages
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .map(|m| m.content.as_str())
        .unwrap_or("");
    let q = last_user.to_lowercase();
    let ctx = grounded.unwrap_or("");

    let body = if q.contains("portfolio") || q.contains("allocation") || q.contains("diversif")
        || ctx.contains("allocation_by_class")
    {
        format!(
            "### Portfolio review (educational)\n\n\
             I analyzed the portfolio metrics available in your account context.\n\n\
             **What to look at**\n\
             - Concentration: if one asset class or sector is well above ~40%, risk is less diversified.\n\
             - Unrealized P&L and overall return show mark-to-market vs cost — not a forecast.\n\
             - XIRR/CAGR (when present) summarize historical cash-flow performance only.\n\n\
             **Context snapshot**\n```\n{}\n```\n\n\
             **Ideas to consider (not recommendations)**\n\
             - Rebalance gradually if one sleeve dominates.\n\
             - Keep an emergency cash buffer separate from risk assets.\n\
             - Match holdings to time horizon and risk tolerance.\n",
            truncate(ctx, 1200)
        )
    } else if q.contains("ipo") || q.contains("drhp") || q.contains("gmp") || ctx.contains("Pros")
        || ctx.contains("financials")
    {
        format!(
            "### IPO notes (educational)\n\n\
             **Grey Market Premium (GMP)** is unofficial and not exchange-endorsed.\n\n\
             **How to read an IPO**\n\
             - Business model & sector risks\n\
             - Financial trajectory in DRHP/RHP (growth, margins, debt)\n\
             - Use of proceeds and dilution\n\
             - Valuation vs peers (never a guarantee of listing gains)\n\
             - Subscription demand is interest, not quality\n\n\
             **Data available**\n```\n{}\n```\n\n\
             Always verify facts in the official RHP and your risk capacity.\n",
            truncate(ctx, 1200)
        )
    } else if q.contains("trade") || q.contains("mistake") || q.contains("journal")
        || q.contains("win rate")
    {
        format!(
            "### Trading journal insights (educational)\n\n\
             Common process leaks to review:\n\
             - Entering without a plan (FOMO / revenge tags)\n\
             - Skipping stop-loss or cutting winners too early\n\
             - Position sizes that break risk rules\n\
             - Strategy drift (many tags, no edge)\n\n\
             **Your recent trade context**\n```\n{}\n```\n\n\
             Focus on process metrics (plan adherence, R-multiple) before outcome luck.\n",
            truncate(ctx, 1200)
        )
    } else if q.contains("market") || q.contains("today") {
        "### Market overview framing (educational)\n\n\
         I do not stream live market news without a data feed. When reviewing “today’s market”:\n\
         - Separate index moves from your specific holdings.\n\
         - Avoid reacting to single-day noise.\n\
         - Check macro calendar (policy, results) only as context, not signals.\n\
         - Your portfolio and journal data matter more than headlines.\n"
            .into()
    } else if q.contains("watchlist") {
        "### Watchlist ideas (educational)\n\n\
         Build a watchlist from: open IPOs you researched, holdings you want price alerts on, \
         and journal symbols you trade often. Limit to a number you can review weekly. \
         A watchlist is not a buy list.\n"
            .into()
    } else {
        format!(
            "### InvestIQ AI\n\n\
             You asked: \"{last_user}\"\n\n\
             I can help with:\n\
             - Summarizing IPO risk/reward framing (GMP is unofficial)\n\
             - Explaining portfolio concentration and returns metrics\n\
             - Reviewing trading journal patterns\n\
             - General investment education\n\n\
             {}\n\
             Ask a more specific question or open a screen (Portfolio / IPO / Journal) so I can use your data.\n",
            if ctx.is_empty() {
                "No extra account context was attached to this message.".to_string()
            } else {
                format!("Attached context:\n```\n{}\n```", truncate(ctx, 800))
            }
        )
    };

    format!("{body}\n---\n{INVESTMENT_DISCLAIMER}")
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_engine_mentions_disclaimer() {
        let msgs = vec![ChatMessage {
            role: "user".into(),
            content: "Review my portfolio risk".into(),
        }];
        let r = local_engine_reply(&msgs, Some(r#"{"allocation_by_class":[]}"#));
        assert!(r.contains("not financial advice") || r.contains("INVESTMENT") || r.contains("disclaimer") || r.to_lowercase().contains("not financial"));
    }
}
