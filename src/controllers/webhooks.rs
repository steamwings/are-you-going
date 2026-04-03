use axum::Json;
use loco_rs::prelude::*;
use serde::Deserialize;

use crate::models::{rsvps, sms_opt_outs};

#[derive(Deserialize)]
pub struct InboundSms {
    pub msisdn: Option<String>,
    pub text: Option<String>,
    // Vonage may also send "from" field
    pub from: Option<String>,
}

async fn inbound_sms(
    State(ctx): State<AppContext>,
    Json(payload): Json<InboundSms>,
) -> Result<Response> {
    let phone = payload.msisdn.or(payload.from).unwrap_or_default();
    let text = payload.text.unwrap_or_default();
    let text_trimmed = text.trim().to_uppercase();

    if text_trimmed == "PEACE BE STILL" || text_trimmed == "STOP" {
        tracing::info!(phone = %phone, "Processing SMS opt-out");

        // Normalize phone if possible, otherwise use raw
        let normalized = crate::models::rsvps::normalize_phone(&phone).unwrap_or(phone.clone());

        sms_opt_outs::upsert_opt_out(&ctx.db, &normalized)
            .await
            .map_err(|e| Error::wrap(e))?;

        rsvps::opt_out_by_phone(&ctx.db, &normalized)
            .await
            .map_err(|e| Error::wrap(e))?;

        tracing::info!(phone = %normalized, "Opt-out recorded");
    } else {
        tracing::info!(phone = %phone, text = %text, "Received inbound SMS (no action)");
    }

    format::empty_json()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/webhooks")
        .add("/inbound-sms", post(inbound_sms))
}
