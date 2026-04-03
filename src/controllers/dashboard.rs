use axum::extract::Path;
use axum::http::header;
use axum_extra::extract::Form;
use chrono_tz::Tz;
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde::Deserialize;

use crate::{
    middleware::dashboard_auth::{require_auth, COOKIE_NAME, COOKIE_VALUE},
    models::{
        _entities::{
            event_reminders, events,
            rsvps,
        },
        events::{generate_slug, validate_custom_slug, FieldConfig},
    },
};

#[derive(Deserialize)]
pub struct LoginForm {
    pub password: String,
}

#[derive(Deserialize)]
pub struct EventForm {
    pub name: String,
    pub description: String,
    pub slug: Option<String>,
    pub show_name: Option<String>,
    pub show_party_size: Option<String>,
    pub show_kids_count: Option<String>,
    pub show_allergies: Option<String>,
    pub custom_prompt: Option<String>,
    #[serde(default)]
    pub reminder_datetime: Vec<String>,
    #[serde(default)]
    pub reminder_message: Vec<String>,
}

fn get_timezone() -> Tz {
    std::env::var("TIMEZONE")
        .unwrap_or_else(|_| "America/New_York".to_string())
        .parse::<Tz>()
        .unwrap_or(chrono_tz::America::New_York)
}

fn utc_to_local(utc: &chrono::DateTime<chrono::FixedOffset>, tz: Tz) -> chrono::NaiveDateTime {
    utc.with_timezone(&tz).naive_local()
}

async fn login_page(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    format::render().view(&v, "dashboard/login.html", serde_json::json!({}))
}

async fn login_submit(
    ViewEngine(v): ViewEngine<TeraView>,
    Form(form): Form<LoginForm>,
) -> Result<Response> {
    let expected = std::env::var("DASHBOARD_PASSWORD").unwrap_or_default();
    if form.password == expected {
        let cookie = format!(
            "{COOKIE_NAME}={COOKIE_VALUE}; Path=/; HttpOnly; SameSite=Lax"
        );
        format::RenderBuilder::new()
            .header(header::SET_COOKIE, &cookie)
            .redirect("/dashboard")
    } else {
        let body = serde_json::json!({"error": "Invalid password"});
        format::render().view(&v, "dashboard/login.html", body)
    }
}

async fn index(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let events_list = events::Entity::find()
        .order_by_desc(events::Column::CreatedAt)
        .all(&ctx.db)
        .await
        .map_err(|e| Error::wrap(e))?;

    let mut events_with_counts = Vec::new();
    for event in &events_list {
        let count = rsvps::Entity::find()
            .filter(rsvps::Column::EventId.eq(event.id))
            .count(&ctx.db)
            .await
            .map_err(|e| Error::wrap(e))?;
        events_with_counts.push(serde_json::json!({
            "id": event.id,
            "name": event.name,
            "slug": event.slug,
            "rsvp_count": count,
        }));
    }

    format::render().view(
        &v,
        "dashboard/index.html",
        serde_json::json!({"events": events_with_counts}),
    )
}

async fn new_event(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    format::render().view(
        &v,
        "dashboard/event_form.html",
        serde_json::json!({"event": null, "error": null}),
    )
}

async fn create_event(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    Form(form): Form<EventForm>,
) -> Result<Response> {
    let slug = if let Some(ref custom) = form.slug {
        let s = custom.trim();
        if s.is_empty() {
            generate_slug()
        } else {
            if !validate_custom_slug(s) {
                return format::render().view(
                    &v,
                    "dashboard/event_form.html",
                    serde_json::json!({
                        "event": null,
                        "error": "Slug must be 3-32 chars, alphanumeric and hyphens only"
                    }),
                );
            }
            s.to_string()
        }
    } else {
        generate_slug()
    };

    // Check uniqueness, retry with random if collision
    let final_slug = if crate::models::events::slug_exists(&ctx.db, &slug)
        .await
        .map_err(|e| Error::wrap(e))?
    {
        if form.slug.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false) {
            return format::render().view(
                &v,
                "dashboard/event_form.html",
                serde_json::json!({
                    "event": null,
                    "error": "That slug is already taken"
                }),
            );
        }
        generate_slug()
    } else {
        slug
    };

    let field_config = FieldConfig {
        show_name: form.show_name.is_some(),
        show_party_size: form.show_party_size.is_some(),
        show_kids_count: form.show_kids_count.is_some(),
        show_allergies: form.show_allergies.is_some(),
        custom_prompt: form.custom_prompt.filter(|s| !s.trim().is_empty()),
    };

    let event = events::ActiveModel::new(&form.name, &form.description, &final_slug, &field_config);
    let event = event.insert(&ctx.db).await.map_err(|e| Error::wrap(e))?;

    save_reminders_from_form(&ctx.db, event.id, &form.reminder_datetime, &form.reminder_message).await?;

    format::redirect(&format!("/dashboard/events/{}", event.id))
}

async fn show_event(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    Path(id): Path<i32>,
) -> Result<Response> {
    let event = events::Entity::find_by_id(id)
        .one(&ctx.db)
        .await
        .map_err(|e| Error::wrap(e))?
        .ok_or_else(|| Error::NotFound)?;

    let rsvps_list = crate::models::rsvps::find_by_event(&ctx.db, event.id)
        .await
        .map_err(|e| Error::wrap(e))?;

    let reminders = crate::models::event_reminders::find_by_event(&ctx.db, event.id)
        .await
        .map_err(|e| Error::wrap(e))?;

    let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:5150".to_string());
    let field_config = event.field_config();
    let tz = get_timezone();

    let reminders_local: Vec<serde_json::Value> = reminders
        .iter()
        .map(|r| {
            let local_dt = utc_to_local(&r.remind_at, tz);
            let sent_at_local = r.sent_at.as_ref().map(|s| {
                utc_to_local(s, tz).format("%Y-%m-%d %I:%M %p").to_string()
            });
            serde_json::json!({
                "remind_at_local": local_dt.format("%Y-%m-%d %I:%M %p").to_string(),
                "message": r.message,
                "sent_at_local": sent_at_local,
            })
        })
        .collect();

    let created_at_local = utc_to_local(&event.created_at, tz)
        .format("%b %-d, %Y %-I:%M %p")
        .to_string();

    format::render().view(
        &v,
        "dashboard/event_detail.html",
        serde_json::json!({
            "event": event,
            "rsvps": rsvps_list,
            "reminders": reminders_local,
            "rsvp_count": rsvps_list.len(),
            "public_url": format!("{}/e/{}", base_url, event.slug),
            "field_config": field_config,
            "timezone": tz.to_string(),
            "created_at": created_at_local,
        }),
    )
}

async fn edit_event(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    Path(id): Path<i32>,
) -> Result<Response> {
    let event = events::Entity::find_by_id(id)
        .one(&ctx.db)
        .await
        .map_err(|e| Error::wrap(e))?
        .ok_or_else(|| Error::NotFound)?;

    let reminders = crate::models::event_reminders::find_unsent_by_event(&ctx.db, event.id)
        .await
        .map_err(|e| Error::wrap(e))?;

    let field_config = event.field_config();
    let tz = get_timezone();

    let reminders_local: Vec<serde_json::Value> = reminders
        .iter()
        .map(|r| {
            let local_dt = utc_to_local(&r.remind_at, tz);
            serde_json::json!({
                "datetime": local_dt.format("%Y-%m-%dT%H:%M").to_string(),
                "message": r.message,
            })
        })
        .collect();

    format::render().view(
        &v,
        "dashboard/event_form.html",
        serde_json::json!({
            "event": event,
            "field_config": field_config,
            "reminders": reminders_local,
            "error": null,
        }),
    )
}

async fn update_event(
    State(ctx): State<AppContext>,
    Path(id): Path<i32>,
    Form(form): Form<EventForm>,
) -> Result<Response> {
    let event = events::Entity::find_by_id(id)
        .one(&ctx.db)
        .await
        .map_err(|e| Error::wrap(e))?
        .ok_or_else(|| Error::NotFound)?;

    let field_config = FieldConfig {
        show_name: form.show_name.is_some(),
        show_party_size: form.show_party_size.is_some(),
        show_kids_count: form.show_kids_count.is_some(),
        show_allergies: form.show_allergies.is_some(),
        custom_prompt: form.custom_prompt.filter(|s| !s.trim().is_empty()),
    };

    let mut active: events::ActiveModel = event.into_active_model();
    active.name = ActiveValue::set(form.name);
    active.description = ActiveValue::set(form.description);
    active.field_config =
        ActiveValue::set(serde_json::to_string(&field_config).unwrap_or_default());
    let event = active.update(&ctx.db).await.map_err(|e| Error::wrap(e))?;

    // Delete only unsent reminders and re-create from form
    event_reminders::Entity::delete_many()
        .filter(event_reminders::Column::EventId.eq(event.id))
        .filter(event_reminders::Column::SentAt.is_null())
        .exec(&ctx.db)
        .await
        .map_err(|e| Error::wrap(e))?;

    save_reminders_from_form(&ctx.db, event.id, &form.reminder_datetime, &form.reminder_message).await?;

    format::redirect(&format!("/dashboard/events/{}", event.id))
}

async fn save_reminders_from_form(
    db: &sea_orm::DatabaseConnection,
    event_id: i32,
    datetimes: &[String],
    messages: &[String],
) -> Result<()> {
    let tz = get_timezone();

    for (dt_str, msg) in datetimes.iter().zip(messages.iter()) {
        let dt_str = dt_str.trim();
        let msg = msg.trim();
        if dt_str.is_empty() || msg.is_empty() {
            continue;
        }
        match chrono::NaiveDateTime::parse_from_str(dt_str, "%Y-%m-%dT%H:%M") {
            Ok(naive) => match naive.and_local_timezone(tz).earliest() {
                Some(local_dt) => {
                    let utc_dt = local_dt.with_timezone(&chrono::Utc);
                    event_reminders::ActiveModel::new(event_id, utc_dt.into(), msg)
                        .insert(db)
                        .await
                        .map_err(|e| Error::wrap(e))?;
                }
                None => tracing::warn!(datetime = %dt_str, "Skipping reminder: ambiguous or invalid local time"),
            },
            Err(e) => tracing::warn!(datetime = %dt_str, error = %e, "Skipping reminder: failed to parse datetime"),
        }
    }
    Ok(())
}

async fn delete_event(
    State(ctx): State<AppContext>,
    Path(id): Path<i32>,
) -> Result<Response> {
    let event = events::Entity::find_by_id(id)
        .one(&ctx.db)
        .await
        .map_err(|e| Error::wrap(e))?
        .ok_or_else(|| Error::NotFound)?;

    // Cascade: delete reminders, RSVPs, then the event
    event_reminders::Entity::delete_many()
        .filter(event_reminders::Column::EventId.eq(id))
        .exec(&ctx.db)
        .await
        .map_err(|e| Error::wrap(e))?;

    rsvps::Entity::delete_many()
        .filter(rsvps::Column::EventId.eq(id))
        .exec(&ctx.db)
        .await
        .map_err(|e| Error::wrap(e))?;

    event.delete(&ctx.db).await.map_err(|e| Error::wrap(e))?;

    format::redirect("/dashboard")
}

async fn reminder_row(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    format::render().view(&v, "dashboard/reminder_row.html", serde_json::json!({}))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/dashboard")
        .add("/login", get(login_page))
        .add("/login", post(login_submit))
        .add("/", get(index))
        .add("/events/new", get(new_event))
        .add("/events/new", post(create_event))
        .add("/events/reminder-row", get(reminder_row))
        .add("/events/{id}", get(show_event))
        .add("/events/{id}/edit", get(edit_event))
        .add("/events/{id}/edit", post(update_event))
        .add("/events/{id}/delete", post(delete_event))
        .layer(axum::middleware::from_fn(require_auth_skip_login))
}

async fn require_auth_skip_login(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    if request.uri().path().starts_with("/dashboard/login") {
        next.run(request).await
    } else {
        require_auth(request, next).await
    }
}
