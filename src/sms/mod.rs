pub mod mock;
pub mod templates;
pub mod vonage;

use async_trait::async_trait;
use std::fmt;

#[derive(Debug)]
pub enum SmsError {
    SendFailed(String),
}

impl fmt::Display for SmsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SmsError::SendFailed(msg) => write!(f, "SMS send failed: {msg}"),
        }
    }
}

impl std::error::Error for SmsError {}

#[async_trait]
pub trait SmsProvider: Send + Sync {
    async fn send_sms(&self, to: &str, body: &str) -> Result<(), SmsError>;
}
