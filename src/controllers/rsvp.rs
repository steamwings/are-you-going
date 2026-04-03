use axum::extract::{Path, Query};
use axum::response::IntoResponse;
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait, IntoActiveModel};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    models::{
        _entities::{magic_links, rsvps},
        events::find_by_slug,
        magic_links as magic_links_model,
        rsvps as rsvps_model,
        sms_opt_outs,
    },
    sms::{templates as sms_templates, SmsProvider},
};

#[derive(Deserialize)]
pub struct RsvpForm {
    pub name: Option<String>,
    pub phone_number: String,
    pub party_size: Option<String>,
    pub kids_count: Option<String>,
    pub sms_opt_out: Option<String>,
    pub allergies: Option<String>,
    pub custom_response: Option<String>,
}

fn parse_count(raw: &Option<String>, field: &str, min: i32, max: i32, default: i32) -> Result<i32, String> {
    match raw.as_deref().map(str::trim) {
        None | Some("") => Ok(default),
        Some(s) => s
            .parse::<i32>()
            .map_err(|_| format!("{field} must be a whole number"))
            .and_then(|n| {
                if n >= min && n <= max {
                    Ok(n)
                } else {
                    Err(format!("{field} must be between {min} and {max}"))
                }
            }),
    }
}

#[derive(Deserialize)]
pub struct EditPhoneForm {
    pub phone_number: String,
}

async fn show_form(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    Path(slug): Path<String>,
) -> Result<Response> {
    let event = find_by_slug(&ctx.db, &slug)
        .await
        .map_err(|e| Error::wrap(e))?
        .ok_or_else(|| Error::NotFound)?;

    let field_config = event.field_config();

    format::render().view(
        &v,
        "rsvp/form.html",
        serde_json::json!({
            "event": event,
            "field_config": field_config,
            "error": null,
            "rsvp": null,
        }),
    )
}

async fn submit_rsvp(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    Path(slug): Path<String>,
    Form(form): Form<RsvpForm>,
) -> Result<Response> {
    let event = find_by_slug(&ctx.db, &slug)
        .await
        .map_err(|e| Error::wrap(e))?
        .ok_or_else(|| Error::NotFound)?;

    let field_config = event.field_config();

    let phone = match rsvps_model::normalize_phone(&form.phone_number) {
        Ok(p) => p,
        Err(e) => {
            return format::render().view(
                &v,
                "rsvp/form.html",
                serde_json::json!({
                    "event": event,
                    "field_config": field_config,
                    "error": e,
                    "rsvp": null,
                }),
            );
        }
    };

    // Check for duplicate
    if let Some(_existing) = rsvps_model::find_by_event_and_phone(&ctx.db, event.id, &phone)
        .await
        .map_err(|e| Error::wrap(e))?
    {
        return format::render().view(
            &v,
            "rsvp/form.html",
            serde_json::json!({
                "event": event,
                "field_config": field_config,
                "error": "A RSVP with this phone number already exists. Try 'Edit existing RSVP'.",
                "rsvp": null,
            }),
        );
    }

    let party_size = match parse_count(&form.party_size, "Party size", 1, 999, 1) {
        Ok(n) => n,
        Err(e) => {
            return format::render().view(
                &v,
                "rsvp/form.html",
                serde_json::json!({ "event": event, "field_config": field_config, "error": e, "rsvp": null }),
            );
        }
    };
    let kids_count = match parse_count(&form.kids_count, "Kids count", 0, 999, 0) {
        Ok(n) => n,
        Err(e) => {
            return format::render().view(
                &v,
                "rsvp/form.html",
                serde_json::json!({ "event": event, "field_config": field_config, "error": e, "rsvp": null }),
            );
        }
    };

    let rsvp = rsvps::ActiveModel::new(
        event.id,
        form.name.as_deref().unwrap_or(""),
        &phone,
        party_size,
        kids_count,
        form.sms_opt_out.is_none(),
        form.allergies.filter(|s| !s.trim().is_empty()),
        form.custom_response.filter(|s| !s.trim().is_empty()),
    );
    rsvp.insert(&ctx.db).await.map_err(|e| Error::wrap(e))?;

    let sms_opt_in = form.sms_opt_out.is_none();
    let encoded_phone = phone.replace('+', "%2B");
    format::redirect(&format!(
        "/e/{}/thanks?phone={}&sms_opt_in={}",
        slug, encoded_phone, sms_opt_in
    ))
}

async fn thanks(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    Path(slug): Path<String>,
    Query(query): Query<ThanksQuery>,
) -> Result<Response> {
    let event = find_by_slug(&ctx.db, &slug)
        .await
        .map_err(|e| Error::wrap(e))?
        .ok_or_else(|| Error::NotFound)?;

    let mut show_reoptout_prompt = false;
    if query.sms_opt_in == Some(true) {
        if let Some(ref phone) = query.phone {
            show_reoptout_prompt = sms_opt_outs::is_opted_out(&ctx.db, phone)
                .await
                .map_err(|e| Error::wrap(e))?;
        }
    }

    format::render().view(
        &v,
        "rsvp/thanks.html",
        serde_json::json!({
            "event": event,
            "slug": slug,
            "show_reoptout_prompt": show_reoptout_prompt,
            "phone": query.phone,
        }),
    )
}

async fn reenable_sms(
    State(ctx): State<AppContext>,
    Path(slug): Path<String>,
    Form(form): Form<ReenableSmsForm>,
) -> Result<Response> {
    let normalized = rsvps_model::normalize_phone(&form.phone)
        .map_err(|e| Error::string(&e))?;
    sms_opt_outs::remove_opt_out(&ctx.db, &normalized)
        .await
        .map_err(|e| Error::wrap(e))?;
    tracing::info!(phone = %normalized, "SMS opt-out removed by user re-enable");
    format::redirect(&format!("/e/{}/thanks", slug))
}

async fn edit_phone_form(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    Path(slug): Path<String>,
) -> Result<Response> {
    let event = find_by_slug(&ctx.db, &slug)
        .await
        .map_err(|e| Error::wrap(e))?
        .ok_or_else(|| Error::NotFound)?;

    format::render().view(
        &v,
        "rsvp/edit_phone.html",
        serde_json::json!({
            "event": event,
            "error": null,
        }),
    )
}

async fn send_magic_link(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    Path(slug): Path<String>,
    Form(form): Form<EditPhoneForm>,
) -> Result<Response> {
    let event = find_by_slug(&ctx.db, &slug)
        .await
        .map_err(|e| Error::wrap(e))?
        .ok_or_else(|| Error::NotFound)?;

    let phone = match rsvps_model::normalize_phone(&form.phone_number) {
        Ok(p) => p,
        Err(e) => {
            return format::render().view(
                &v,
                "rsvp/edit_phone.html",
                serde_json::json!({
                    "event": event,
                    "error": e,
                }),
            );
        }
    };

    let rsvp = rsvps_model::find_by_event_and_phone(&ctx.db, event.id, &phone)
        .await
        .map_err(|e| Error::wrap(e))?;

    let rsvp = match rsvp {
        Some(r) => r,
        None => {
            return format::render().view(
                &v,
                "rsvp/edit_phone.html",
                serde_json::json!({
                    "event": event,
                    "error": "No RSVP found for this phone number.",
                }),
            );
        }
    };

    let expiration: i64 = std::env::var("MAGIC_LINK_EXPIRATION_SECONDS")
        .unwrap_or_else(|_| "300".to_string())
        .parse()
        .unwrap_or(300);

    let magic_link = magic_links::ActiveModel::new(rsvp.id, expiration);
    let magic_link = magic_link
        .insert(&ctx.db)
        .await
        .map_err(|e| Error::wrap(e))?;

    let base_url =
        std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:5150".to_string());
    let link = format!("{}/e/{}/edit/{}", base_url, slug, magic_link.token);

    let sms_body = sms_templates::magic_link_sms(&event.name, &link);

    // Send SMS
    if let Some(provider) = ctx.shared_store.get::<Arc<dyn SmsProvider>>() {
        if let Err(e) = provider.send_sms(&phone, &sms_body).await {
            tracing::error!("Failed to send magic link SMS: {e}");
        }
    }

    format::render().view(
        &v,
        "rsvp/magic_link_sent.html",
        serde_json::json!({
            "event": event,
        }),
    )
}

async fn edit_form(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    Path((slug, token)): Path<(String, String)>,
) -> Result<Response> {
    let event = find_by_slug(&ctx.db, &slug)
        .await
        .map_err(|e| Error::wrap(e))?
        .ok_or_else(|| Error::NotFound)?;

    let magic_link = magic_links_model::find_valid_by_token(&ctx.db, &token)
        .await
        .map_err(|e| Error::wrap(e))?
        .ok_or_else(|| Error::NotFound)?;

    let rsvp = rsvps::Entity::find_by_id(magic_link.rsvp_id)
        .one(&ctx.db)
        .await
        .map_err(|e| Error::wrap(e))?
        .ok_or_else(|| Error::NotFound)?;

    let field_config = event.field_config();

    format::render().view(
        &v,
        "rsvp/edit_form.html",
        serde_json::json!({
            "event": event,
            "field_config": field_config,
            "rsvp": rsvp,
            "token": token,
            "error": null,
        }),
    )
}

async fn update_rsvp(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    Path((slug, token)): Path<(String, String)>,
    Form(form): Form<RsvpForm>,
) -> Result<Response> {
    let event = find_by_slug(&ctx.db, &slug)
        .await
        .map_err(|e| Error::wrap(e))?
        .ok_or_else(|| Error::NotFound)?;

    let magic_link = magic_links_model::find_valid_by_token(&ctx.db, &token)
        .await
        .map_err(|e| Error::wrap(e))?
        .ok_or_else(|| Error::NotFound)?;

    let rsvp = rsvps::Entity::find_by_id(magic_link.rsvp_id)
        .one(&ctx.db)
        .await
        .map_err(|e| Error::wrap(e))?
        .ok_or_else(|| Error::NotFound)?;

    let field_config = event.field_config();

    let phone = match rsvps_model::normalize_phone(&form.phone_number) {
        Ok(p) => p,
        Err(e) => {
            return format::render().view(
                &v,
                "rsvp/edit_form.html",
                serde_json::json!({
                    "event": event,
                    "field_config": field_config,
                    "rsvp": rsvp,
                    "token": token,
                    "error": e,
                }),
            );
        }
    };

    let party_size = match parse_count(&form.party_size, "Party size", 1, 999, 1) {
        Ok(n) => n,
        Err(e) => {
            return format::render().view(
                &v,
                "rsvp/edit_form.html",
                serde_json::json!({ "event": event, "field_config": field_config, "rsvp": rsvp, "token": token, "error": e }),
            );
        }
    };
    let kids_count = match parse_count(&form.kids_count, "Kids count", 0, 999, 0) {
        Ok(n) => n,
        Err(e) => {
            return format::render().view(
                &v,
                "rsvp/edit_form.html",
                serde_json::json!({ "event": event, "field_config": field_config, "rsvp": rsvp, "token": token, "error": e }),
            );
        }
    };
    let allergies_text = form.allergies.filter(|s| !s.trim().is_empty());

    let mut active: rsvps::ActiveModel = rsvp.into_active_model();
    active.name = ActiveValue::set(form.name.unwrap_or_default());
    active.phone_number = ActiveValue::set(phone);
    active.party_size = ActiveValue::set(party_size);
    active.kids_count = ActiveValue::set(kids_count);
    active.sms_opt_in = ActiveValue::set(form.sms_opt_out.is_none());
    active.has_allergies = ActiveValue::set(allergies_text.as_ref().map_or(false, |s| !s.is_empty()));
    active.allergies_text = ActiveValue::set(allergies_text);
    active.custom_response = ActiveValue::set(form.custom_response.filter(|s| !s.trim().is_empty()));
    let updated = active.update(&ctx.db).await.map_err(|e| Error::wrap(e))?;

    let encoded_phone = updated.phone_number.replace('+', "%2B");
    format::redirect(&format!(
        "/e/{}/thanks?phone={}&sms_opt_in={}",
        slug, encoded_phone, updated.sms_opt_in
    ))
}

#[derive(Deserialize)]
struct ThanksQuery {
    phone: Option<String>,
    sms_opt_in: Option<bool>,
}

#[derive(Deserialize)]
struct ReenableSmsForm {
    phone: String,
}

#[derive(Deserialize)]
struct PhoneQuery {
    phone_number: String,
}

async fn format_phone(Query(q): Query<PhoneQuery>) -> impl IntoResponse {
    let value = crate::models::phone::format_phone_display(&q.phone_number)
        .unwrap_or(q.phone_number);
    let html = format!(
        r#"<input type="tel" id="phone_number" name="phone_number" required
                  placeholder="+1 (555) 123-4567" value="{value}"
                  hx-get="/e/phone/format" hx-trigger="blur"
                  hx-swap="outerHTML" hx-include="this">"#
    );
    axum::response::Html(html)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/e")
        .add("/phone/format", get(format_phone))
        .add("/{slug}", get(show_form))
        .add("/{slug}", post(submit_rsvp))
        .add("/{slug}/thanks", get(thanks))
        .add("/{slug}/edit", get(edit_phone_form))
        .add("/{slug}/edit", post(send_magic_link))
        .add("/{slug}/edit/{token}", get(edit_form))
        .add("/{slug}/edit/{token}", post(update_rsvp))
        .add("/{slug}/reenable-sms", post(reenable_sms))
}
