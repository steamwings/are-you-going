use are_you_going::app::App;
use are_you_going::models::{
    _entities::{events, rsvps},
    events::FieldConfig,
    sms_opt_outs,
};
use loco_rs::testing::request::request;
use sea_orm::ActiveModelTrait;
use serde_json::json;

#[tokio::test]
#[serial_test::serial]
async fn test_inbound_sms_opt_out_stop() {
    request::<App, _, _>(|server, ctx| async move {
        let field_config = FieldConfig::default();
        let event = events::ActiveModel::new("Test", "Test", "wh-test", &field_config);
        let event = event.insert(&ctx.db).await.unwrap();

        let rsvp = rsvps::ActiveModel::new(
            event.id, "Test User", "+14434631334", 1, 0, true, None, None,
        );
        rsvp.insert(&ctx.db).await.unwrap();

        let payload = json!({
            "msisdn": "+14434631334",
            "text": "STOP"
        });

        let response = server
            .post("/webhooks/inbound-sms")
            .json(&payload)
            .await;

        response.assert_status_ok();

        let opted_out = sms_opt_outs::is_opted_out(&ctx.db, "+14434631334")
            .await
            .unwrap();
        assert!(opted_out);
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_inbound_sms_peace_be_still() {
    request::<App, _, _>(|server, ctx| async move {
        let payload = json!({
            "msisdn": "+15551234567",
            "text": "peace be still"
        });

        let response = server
            .post("/webhooks/inbound-sms")
            .json(&payload)
            .await;

        response.assert_status_ok();

        let opted_out = sms_opt_outs::is_opted_out(&ctx.db, "+15551234567")
            .await
            .unwrap();
        assert!(opted_out);
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_inbound_sms_no_action() {
    request::<App, _, _>(|server, ctx| async move {
        let payload = json!({
            "msisdn": "+15559999999",
            "text": "hello there"
        });

        let response = server
            .post("/webhooks/inbound-sms")
            .json(&payload)
            .await;

        response.assert_status_ok();

        let opted_out = sms_opt_outs::is_opted_out(&ctx.db, "+15559999999")
            .await
            .unwrap();
        assert!(!opted_out);
    })
    .await;
}
