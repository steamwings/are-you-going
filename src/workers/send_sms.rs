use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::sms::SmsProvider;

pub struct SendSmsWorker {
    pub ctx: AppContext,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct SendSmsWorkerArgs {
    pub to: String,
    pub body: String,
}

#[async_trait]
impl BackgroundWorker<SendSmsWorkerArgs> for SendSmsWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }

    async fn perform(&self, args: SendSmsWorkerArgs) -> Result<()> {
        if let Some(provider) = self.ctx.shared_store.get::<Arc<dyn SmsProvider>>() {
            provider
                .send_sms(&args.to, &args.body)
                .await
                .map_err(|e| Error::string(&e.to_string()))?;
            tracing::info!(to = %args.to, "SMS sent successfully");
        } else {
            tracing::warn!("No SMS provider configured, skipping SMS send");
        }
        Ok(())
    }
}
