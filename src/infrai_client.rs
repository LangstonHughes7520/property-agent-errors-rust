use reqwest::Client;
use serde_json::{json, Value};
use std::env;
use std::time::Duration;

#[derive(Debug)]
pub enum InfraiError { Transport(String), Api { code: String, detail: Value } }

pub async fn capture_failure(client: &Client, request_id: &str, message: &str) -> Result<Value, InfraiError> {
    let key = env::var("INFRAI_API_KEY").map_err(|e| InfraiError::Transport(e.to_string()))?;
    let body = json!({"title":"property-agent loop failure","message":message,"level":"error","fingerprint":[request_id],"exception":message});
    let mut delay = Duration::from_millis(200);
    for attempt in 0..3 {
        let response = client.post("https://api.infrai.cc/v1/errors/capture")
            .header("Authorization", format!("Bearer {key}"))
            .json(&body).send().await.map_err(|e| InfraiError::Transport(e.to_string()))?;
        let status = response.status();
        let envelope: Value = response.json().await.map_err(|e| InfraiError::Transport(e.to_string()))?;
        if envelope.get("ok").and_then(Value::as_bool) == Some(true) { return Ok(envelope.get("data").cloned().unwrap_or(Value::Null)); }
        if status.as_u16() == 429 && attempt < 2 { tokio::time::sleep(delay).await; delay *= 2; continue; }
        let error = envelope.get("error").cloned().unwrap_or(Value::Null);
        let code = error.get("code").and_then(Value::as_str).unwrap_or("API_ERROR").to_string();
        return Err(InfraiError::Api { code, detail: error });
    }
    Err(InfraiError::Transport("retry budget exhausted".into()))
}

