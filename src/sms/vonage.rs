use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use super::{SmsError, SmsProvider};

pub struct VonageProvider {
    client: Client,
    base_url: String,
    api_key: String,
    api_secret: String,
    from_number: String,
}

impl VonageProvider {
    pub fn new(base_url: &str, api_key: &str, api_secret: &str, from_number: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            api_secret: api_secret.to_string(),
            from_number: from_number.to_string(),
        }
    }
}

#[async_trait]
impl SmsProvider for VonageProvider {
    async fn send_sms(&self, to: &str, body: &str) -> Result<(), SmsError> {
        let url = format!("{}/messages", self.base_url);

        let payload = json!({
            "to": to,
            "from": self.from_number,
            "channel": "sms",
            "message_type": "text",
            "text": body,
        });

        let response = self
            .client
            .post(&url)
            .basic_auth(&self.api_key, Some(&self.api_secret))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| SmsError::SendFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(SmsError::SendFailed(format!(
                "HTTP {status}: {body}"
            )));
        }

        let resp_json: serde_json::Value = response.json().await.unwrap_or_default();
        let message_uuid = resp_json["message_uuid"].as_str().unwrap_or("unknown");
        tracing::info!(to, message_uuid, "Vonage accepted SMS");

        Ok(())
    }
}
