use are_you_going::{
    app::App,
    models::_entities::events,
    models::events::FieldConfig,
};
use axum::http::header;
use loco_rs::testing::request::request;
use sea_orm::ActiveModelTrait;

const AUTH_COOKIE: &str = "dashboard_auth=authenticated";

async fn create_test_event(db: &sea_orm::DatabaseConnection, slug: &str) -> events::Model {
    let field_config = FieldConfig::default();
    events::ActiveModel::new("Test Event", "A test event", slug, &field_config)
        .insert(db)
        .await
        .unwrap()
}

#[tokio::test]
#[serial_test::serial]
async fn test_delete_event_redirects_to_dashboard() {
    request::<App, _, _>(|server, ctx| async move {
        let event = create_test_event(&ctx.db, "del-http").await;

        let response = server
            .post(&format!("/dashboard/events/{}/delete", event.id))
            .add_header(header::COOKIE, AUTH_COOKIE)
            .await;

        assert_eq!(response.status_code(), 303);
        assert_eq!(response.header("location"), "/dashboard");
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_delete_nonexistent_event_returns_not_found() {
    request::<App, _, _>(|server, _ctx| async move {
        let response = server
            .post("/dashboard/events/99999/delete")
            .add_header(header::COOKIE, AUTH_COOKIE)
            .await;

        response.assert_status_not_found();
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_delete_event_without_auth_redirects_to_login() {
    request::<App, _, _>(|server, ctx| async move {
        let event = create_test_event(&ctx.db, "del-noauth").await;

        let response = server
            .post(&format!("/dashboard/events/{}/delete", event.id))
            .await;

        assert_eq!(response.status_code(), 303);
        assert!(response.header("location").to_str().unwrap().contains("/dashboard/login"));
    })
    .await;
}
