use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::infra::ai::{ChatMessage, INVESTMENT_DISCLAIMER};
use crate::middleware::AuthUser;
use crate::modules::common::ApiResponse;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/conversations", get(list_conversations).post(create_conversation))
        .route("/conversations/{id}", get(get_conversation).delete(delete_conversation))
        .route("/chat", post(chat))
}

#[derive(Debug, Serialize, FromRow)]
struct ConversationRow {
    id: Uuid,
    user_id: Uuid,
    title: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
struct MessageRow {
    id: Uuid,
    conversation_id: Uuid,
    role: String,
    content: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateConversation {
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub conversation_id: Option<Uuid>,
    pub message: String,
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub conversation_id: Uuid,
    pub reply: String,
    pub disclaimer: &'static str,
}

async fn list_conversations(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<ApiResponse<Vec<ConversationRow>>>> {
    let rows = sqlx::query_as::<_, ConversationRow>(
        r#"
        SELECT id, user_id, title, created_at, updated_at
        FROM ai_conversations WHERE user_id = $1
        ORDER BY updated_at DESC LIMIT 50
        "#,
    )
    .bind(user.user_id)
    .fetch_all(state.db())
    .await?;
    Ok(Json(ApiResponse::ok(rows)))
}

async fn create_conversation(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateConversation>,
) -> AppResult<Json<ApiResponse<ConversationRow>>> {
    let row = sqlx::query_as::<_, ConversationRow>(
        r#"
        INSERT INTO ai_conversations (user_id, title)
        VALUES ($1, $2)
        RETURNING id, user_id, title, created_at, updated_at
        "#,
    )
    .bind(user.user_id)
    .bind(&body.title)
    .fetch_one(state.db())
    .await?;
    Ok(Json(ApiResponse::ok(row)))
}

async fn get_conversation(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let conv = sqlx::query_as::<_, ConversationRow>(
        r#"SELECT id, user_id, title, created_at, updated_at FROM ai_conversations WHERE id = $1 AND user_id = $2"#,
    )
    .bind(id)
    .bind(user.user_id)
    .fetch_optional(state.db())
    .await?
    .ok_or_else(|| AppError::NotFound("conversation not found".into()))?;

    let messages = sqlx::query_as::<_, MessageRow>(
        r#"
        SELECT id, conversation_id, role, content, created_at
        FROM ai_messages WHERE conversation_id = $1 ORDER BY created_at
        "#,
    )
    .bind(id)
    .fetch_all(state.db())
    .await?;

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "conversation": conv,
        "messages": messages,
        "disclaimer": INVESTMENT_DISCLAIMER
    }))))
}

async fn delete_conversation(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<&'static str>>> {
    let res = sqlx::query(r#"DELETE FROM ai_conversations WHERE id = $1 AND user_id = $2"#)
        .bind(id)
        .bind(user.user_id)
        .execute(state.db())
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("conversation not found".into()));
    }
    Ok(Json(ApiResponse::ok("deleted")))
}

async fn chat(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<ChatRequest>,
) -> AppResult<Json<ApiResponse<ChatResponse>>> {
    if body.message.trim().is_empty() {
        return Err(AppError::Validation("message required".into()));
    }

    let conv_id = if let Some(id) = body.conversation_id {
        sqlx::query_scalar::<_, Uuid>(
            r#"SELECT id FROM ai_conversations WHERE id = $1 AND user_id = $2"#,
        )
        .bind(id)
        .bind(user.user_id)
        .fetch_optional(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound("conversation not found".into()))?
    } else {
        let title = body.message.chars().take(60).collect::<String>();
        sqlx::query_scalar::<_, Uuid>(
            r#"INSERT INTO ai_conversations (user_id, title) VALUES ($1, $2) RETURNING id"#,
        )
        .bind(user.user_id)
        .bind(&title)
        .fetch_one(state.db())
        .await?
    };

    sqlx::query(
        r#"INSERT INTO ai_messages (conversation_id, role, content, context_refs) VALUES ($1, 'user', $2, $3)"#,
    )
    .bind(conv_id)
    .bind(&body.message)
    .bind(body.context.clone().unwrap_or(serde_json::json!({})))
    .execute(state.db())
    .await?;

    let history = sqlx::query_as::<_, MessageRow>(
        r#"
        SELECT id, conversation_id, role, content, created_at
        FROM ai_messages WHERE conversation_id = $1 ORDER BY created_at DESC LIMIT 20
        "#,
    )
    .bind(conv_id)
    .fetch_all(state.db())
    .await?;

    let mut msgs: Vec<ChatMessage> = history
        .into_iter()
        .rev()
        .filter(|m| m.role == "user" || m.role == "assistant")
        .map(|m| ChatMessage {
            role: m.role,
            content: m.content,
        })
        .collect();

    // Ensure last message is current (already inserted)
    if msgs.is_empty() {
        msgs.push(ChatMessage {
            role: "user".into(),
            content: body.message.clone(),
        });
    }

    let grounded = body.context.map(|c| c.to_string());
    let reply = state.ai().chat(msgs, grounded).await?;

    sqlx::query(
        r#"INSERT INTO ai_messages (conversation_id, role, content) VALUES ($1, 'assistant', $2)"#,
    )
    .bind(conv_id)
    .bind(&reply)
    .execute(state.db())
    .await?;

    sqlx::query(r#"UPDATE ai_conversations SET updated_at = NOW() WHERE id = $1"#)
        .bind(conv_id)
        .execute(state.db())
        .await?;

    Ok(Json(ApiResponse::ok(ChatResponse {
        conversation_id: conv_id,
        reply,
        disclaimer: INVESTMENT_DISCLAIMER,
    })))
}
