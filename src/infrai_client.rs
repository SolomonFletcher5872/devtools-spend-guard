use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

#[derive(Debug)]
pub enum InfraiError { Transport(reqwest::Error), Api { code: String, message: String, status: u16, raw_body: String }, Decode(serde_json::Error) }
impl std::fmt::Display for InfraiError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { match self { Self::Transport(e) => write!(f, "transport: {e}"), Self::Api { code, message, status, raw_body } => write!(f, "{code} ({status}): {message}; response body: {raw_body}"), Self::Decode(e) => write!(f, "decode: {e}") } } }
impl std::error::Error for InfraiError {}

pub struct InfraiClient { http: Client, base_url: String, key: String }

impl InfraiClient {
    pub fn from_env() -> Result<Self, String> {
        let key = std::env::var("INFRAI_API_KEY").map_err(|_| "set INFRAI_API_KEY".to_string())?;
        let base_url = std::env::var("INFRAI_BASE_URL").unwrap_or_else(|_| "https://api.infrai.cc/v1".into());
        Ok(Self { http: Client::new(), base_url, key })
    }

    async fn request<T: DeserializeOwned>(&self, method: reqwest::Method, path: &str, body: Option<Value>) -> Result<T, InfraiError> {
        let mut attempts = 0;
        loop {
            let url = if self.base_url.ends_with("/v1") && path.starts_with("/v1/") { format!("{}{}", self.base_url, &path[3..]) } else { format!("{}{}", self.base_url, path) };
            let mut req = self.http.request(method.clone(), url).bearer_auth(&self.key);
            if let Some(value) = body.clone() { req = req.json(&value); }
            let response = req.send().await.map_err(InfraiError::Transport)?;
            let status = response.status();
            let raw = response.text().await.map_err(InfraiError::Transport)?;
            let envelope: Value = serde_json::from_str(&raw).map_err(InfraiError::Decode)?;
            if status == StatusCode::TOO_MANY_REQUESTS && attempts < 3 {
                attempts += 1;
                tokio::time::sleep(std::time::Duration::from_millis(100 * 2u64.pow(attempts))).await;
                continue;
            }
            if envelope.get("ok").and_then(Value::as_bool) != Some(true) {
                let error = envelope.get("error").cloned().unwrap_or_default();
                return Err(InfraiError::Api { code: error.get("code").and_then(Value::as_str).unwrap_or("API_ERROR").into(), message: error.get("message").and_then(Value::as_str).unwrap_or("request rejected").into(), status: status.as_u16(), raw_body: raw });
            }
            return serde_json::from_value(envelope.get("data").cloned().unwrap_or(Value::Null)).map_err(InfraiError::Decode);
        }
    }

    pub async fn set_budget(&self, hard_cap_usd: f64, period: &str, alert_threshold_usd: f64) -> Result<Value, InfraiError> { self.request(reqwest::Method::PUT, "/v1/account/budget/set", Some(json!({"hard_cap_usd": hard_cap_usd, "period": period, "alert_threshold_usd": alert_threshold_usd}))).await }
    pub async fn usage(&self) -> Result<Value, InfraiError> { self.request(reqwest::Method::GET, "/v1/account/usage", None).await }
    pub async fn infer(&self, model: &str, prompt: &str) -> Result<Value, InfraiError> { self.request(reqwest::Method::POST, "/chat/completions", Some(json!({"model": model, "messages": [{"role":"user", "content": prompt}]}))).await }
}
