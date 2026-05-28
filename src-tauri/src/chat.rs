use crate::runtime::InferenceState;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Instant;
use tauri::{AppHandle, Emitter, State};

#[derive(Deserialize, Serialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Clone)]
pub struct ChatToken {
    pub delta: String,
}

#[derive(Serialize, Clone)]
pub struct ChatMetrics {
    pub ttft_ms: u64,
    pub tokens: u32,
    pub tok_per_s: f32,
}

#[tauri::command]
pub async fn chat_stream(
    app: AppHandle,
    state: State<'_, InferenceState>,
    messages: Vec<ChatMessage>,
    temperature: f32,
    top_p: f32,
) -> Result<ChatMetrics, String> {
    let port = state
        .port
        .lock()
        .unwrap()
        .ok_or_else(|| "no model loaded".to_string())?;

    let url = format!("http://127.0.0.1:{port}/v1/chat/completions");
    let body = json!({
        "messages": messages,
        "temperature": temperature,
        "top_p": top_p,
        "stream": true,
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("server status {}", resp.status()));
    }

    let started = Instant::now();
    let mut first_token_at: Option<Instant> = None;
    let mut tokens: u32 = 0;
    let mut buffer = String::new();
    let mut stream = resp.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let bytes = chunk.map_err(|e| e.to_string())?;
        buffer.push_str(&String::from_utf8_lossy(&bytes));

        while let Some(idx) = buffer.find('\n') {
            let line = buffer[..idx].trim().to_string();
            buffer.drain(..=idx);

            let payload = match line.strip_prefix("data:").map(str::trim) {
                Some(p) => p,
                None => continue,
            };

            if payload == "[DONE]" {
                continue;
            }

            let value: Value = match serde_json::from_str(payload) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if let Some(delta) = value
                .get("choices")
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("delta"))
                .and_then(|d| d.get("content"))
                .and_then(|c| c.as_str())
            {
                if !delta.is_empty() {
                    if first_token_at.is_none() {
                        first_token_at = Some(Instant::now());
                    }
                    tokens += 1;
                    let _ = app.emit(
                        "chat:token",
                        ChatToken {
                            delta: delta.to_string(),
                        },
                    );
                }
            }
        }
    }

    let total_secs = started.elapsed().as_secs_f32().max(0.001);
    let ttft_ms = first_token_at
        .map(|t| t.duration_since(started).as_millis() as u64)
        .unwrap_or(0);
    let tok_per_s = if total_secs > 0.0 {
        tokens as f32 / total_secs
    } else {
        0.0
    };

    Ok(ChatMetrics {
        ttft_ms,
        tokens,
        tok_per_s,
    })
}
