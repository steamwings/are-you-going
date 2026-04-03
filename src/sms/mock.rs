use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use super::{SmsError, SmsProvider};

#[derive(Clone, Default)]
pub struct MockProvider {
    pub messages: Arc<Mutex<Vec<(String, String)>>>,
}

impl MockProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sent_messages(&self) -> Vec<(String, String)> {
        self.messages.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        self.messages.lock().unwrap().clear();
    }
}

#[async_trait]
impl SmsProvider for MockProvider {
    async fn send_sms(&self, to: &str, body: &str) -> Result<(), SmsError> {
        self.messages
            .lock()
            .unwrap()
            .push((to.to_string(), body.to_string()));
        Ok(())
    }
}
