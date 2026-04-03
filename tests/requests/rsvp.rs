use are_you_going::app::App;
use are_you_going::models::{
    _entities::events,
    events::FieldConfig,
};
use loco_rs::testing::request::request;
use sea_orm::ActiveModelTrait;
use serde::Serialize;

async fn create_test_event(db: &sea_orm::DatabaseConnection) -> events::Model {
    let field_config = FieldConfig::default();
    let event = events::ActiveModel::new("Test Event", "A test event", "test-ev", &field_config);
    event.insert(db).await.unwrap()
}

#[tokio::test]
#[serial_test::serial]
async fn test_rsvp_form_loads() {
    request::<App, _, _>(|server, ctx| async move {
        let _event = create_test_event(&ctx.db).await;

        let response = server.get("/e/test-ev").await;
        response.assert_status_ok();
        let body = response.text();
        assert!(body.contains("Test Event"));
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_rsvp_form_404_for_missing_slug() {
    request::<App, _, _>(|server, _ctx| async move {
        let response = server.get("/e/nonexistent").await;
        response.assert_status_not_found();
    })
    .await;
}

// --- phone format endpoint ---

#[tokio::test]
#[serial_test::serial]
async fn test_phone_format_endpoint_normalizes() {
    request::<App, _, _>(|server, _ctx| async move {
        let response = server.get("/e/phone/format?phone_number=2025550100").await;
        response.assert_status_ok();
        let body = response.text();
        assert!(body.contains("+1 (202) 555-0100"), "got: {body}");
        // Response must be an input element with htmx attributes so it can swap itself
        assert!(body.contains("hx-get"));
        assert!(body.contains("hx-trigger=\"blur\""));
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_phone_format_endpoint_passthrough_on_invalid() {
    request::<App, _, _>(|server, _ctx| async move {
        let response = server.get("/e/phone/format?phone_number=notaphone").await;
        response.assert_status_ok();
        // Falls back to the raw value — no crash, no data loss
        let body = response.text();
        assert!(body.contains("notaphone"), "got: {body}");
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_phone_format_endpoint_accepts_dashes() {
    request::<App, _, _>(|server, _ctx| async move {
        let response = server.get("/e/phone/format?phone_number=202-555-0100").await;
        response.assert_status_ok();
        assert!(response.text().contains("+1 (202) 555-0100"));
    })
    .await;
}

// --- RSVP form submission ---

#[derive(Serialize)]
struct RsvpPayload {
    phone_number: String,
    party_size: String,
    kids_count: String,
    sms_opt_in: Option<String>,
    allergies: Option<String>,
    custom_response: Option<String>,
}

impl RsvpPayload {
    fn minimal(phone: &str) -> Self {
        Self {
            phone_number: phone.to_string(),
            party_size: "2".to_string(),
            kids_count: "0".to_string(),
            sms_opt_in: None,
            allergies: None,
            custom_response: None,
        }
    }
}

#[tokio::test]
#[serial_test::serial]
async fn test_rsvp_submit_valid() {
    request::<App, _, _>(|server, ctx| async move {
        let _event = create_test_event(&ctx.db).await;
        let response = server
            .post("/e/test-ev")
            .form(&RsvpPayload::minimal("2025550100"))
            .await;
        // Successful submit redirects to thanks with phone and opt-in params
        assert_eq!(response.status_code(), 303);
        assert!(response.header("location").to_str().unwrap().starts_with("/e/test-ev/thanks"));
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_rsvp_submit_invalid_phone_shows_error() {
    request::<App, _, _>(|server, ctx| async move {
        let _event = create_test_event(&ctx.db).await;
        let response = server
            .post("/e/test-ev")
            .form(&RsvpPayload::minimal("notaphone"))
            .await;
        response.assert_status_ok();
        assert!(response.text().contains("Invalid phone number"));
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_rsvp_submit_party_size_overflow_shows_error() {
    request::<App, _, _>(|server, ctx| async move {
        let _event = create_test_event(&ctx.db).await;
        let payload = RsvpPayload {
            party_size: "99999999999".to_string(),
            ..RsvpPayload::minimal("2025550100")
        };
        let response = server.post("/e/test-ev").form(&payload).await;
        response.assert_status_ok();
        assert!(response.text().contains("Party size"));
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_rsvp_submit_party_size_zero_shows_error() {
    request::<App, _, _>(|server, ctx| async move {
        let _event = create_test_event(&ctx.db).await;
        let payload = RsvpPayload {
            party_size: "0".to_string(),
            ..RsvpPayload::minimal("2025550100")
        };
        let response = server.post("/e/test-ev").form(&payload).await;
        response.assert_status_ok();
        assert!(response.text().contains("Party size"));
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_rsvp_submit_party_size_non_numeric_shows_error() {
    request::<App, _, _>(|server, ctx| async move {
        let _event = create_test_event(&ctx.db).await;
        let payload = RsvpPayload {
            party_size: "many".to_string(),
            ..RsvpPayload::minimal("2025550100")
        };
        let response = server.post("/e/test-ev").form(&payload).await;
        response.assert_status_ok();
        assert!(response.text().contains("Party size"));
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_rsvp_submit_duplicate_phone_shows_error() {
    request::<App, _, _>(|server, ctx| async move {
        let _event = create_test_event(&ctx.db).await;
        // First submission succeeds
        server
            .post("/e/test-ev")
            .form(&RsvpPayload::minimal("2025550100"))
            .await;
        // Second submission with same number is rejected
        let response = server
            .post("/e/test-ev")
            .form(&RsvpPayload::minimal("2025550100"))
            .await;
        response.assert_status_ok();
        assert!(response.text().contains("already exists"));
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_rsvp_submit_formatted_phone_normalizes() {
    request::<App, _, _>(|server, ctx| async move {
        let _event = create_test_event(&ctx.db).await;
        // User typed dashes — should normalize and succeed
        let response = server
            .post("/e/test-ev")
            .form(&RsvpPayload::minimal("202-555-0100"))
            .await;
        assert_eq!(response.status_code(), 303);
    })
    .await;
}
