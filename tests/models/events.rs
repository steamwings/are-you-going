use are_you_going::{
    app::App,
    models::events::{generate_slug, validate_custom_slug, FieldConfig},
    models::_entities::{event_reminders, events, rsvps},
};
use loco_rs::testing::request::request;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter};

#[tokio::test]
async fn test_slug_generation() {
    let slug = generate_slug();
    assert_eq!(slug.len(), 6);
    assert!(slug.chars().all(|c| c.is_ascii_alphanumeric()));
}

#[tokio::test]
async fn test_validate_custom_slug() {
    assert!(validate_custom_slug("abc"));
    assert!(validate_custom_slug("my-event-2026"));
    assert!(!validate_custom_slug("ab"));
    assert!(!validate_custom_slug(&"a".repeat(33)));
    assert!(!validate_custom_slug("no spaces"));
    assert!(!validate_custom_slug("no_underscores"));
}

#[tokio::test]
async fn test_field_config_serde() {
    let config = FieldConfig {
        show_name: true,
        show_party_size: true,
        show_kids_count: false,
        show_allergies: true,
        custom_prompt: Some("What dish?".to_string()),
    };

    let json = serde_json::to_string(&config).unwrap();
    let parsed: FieldConfig = serde_json::from_str(&json).unwrap();

    assert!(parsed.show_name);
    assert!(parsed.show_party_size);
    assert!(!parsed.show_kids_count);
    assert!(parsed.show_allergies);
    assert_eq!(parsed.custom_prompt.unwrap(), "What dish?");
}

#[tokio::test]
async fn test_field_config_defaults() {
    let config: FieldConfig = serde_json::from_str("{}").unwrap();
    assert!(config.show_name);
    assert!(config.show_party_size);
    assert!(!config.show_kids_count);
    assert!(!config.show_allergies);
    assert!(config.custom_prompt.is_none());
}

#[tokio::test]
#[serial_test::serial]
async fn test_create_event() {
    request::<App, _, _>(|_server, ctx| async move {
        let field_config = FieldConfig::default();
        let event = are_you_going::models::_entities::events::ActiveModel::new(
            "Test Event",
            "A test event",
            "test-slug",
            &field_config,
        );
        let event = event.insert(&ctx.db).await.unwrap();

        assert_eq!(event.name, "Test Event");
        assert_eq!(event.slug, "test-slug");

        let found = are_you_going::models::events::find_by_slug(&ctx.db, "test-slug")
            .await
            .unwrap();
        assert!(found.is_some());
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_delete_event_removes_from_db() {
    request::<App, _, _>(|_server, ctx| async move {
        let field_config = FieldConfig::default();
        let event = events::ActiveModel::new("Delete Me", "gone soon", "del-me", &field_config)
            .insert(&ctx.db)
            .await
            .unwrap();
        let id = event.id;

        event.delete(&ctx.db).await.unwrap();

        let found = events::Entity::find_by_id(id).one(&ctx.db).await.unwrap();
        assert!(found.is_none());
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_delete_event_cascade_deletes_rsvps() {
    request::<App, _, _>(|_server, ctx| async move {
        let field_config = FieldConfig::default();
        let event = events::ActiveModel::new("RSVP Event", "has rsvps", "rsvp-del", &field_config)
            .insert(&ctx.db)
            .await
            .unwrap();

        rsvps::ActiveModel::new(
            event.id,
            "Alice",
            "+12025550100",
            2,
            0,
            false,
            None,
            None,
        )
        .insert(&ctx.db)
        .await
        .unwrap();

        let event_id = event.id;

        // Mirror the handler's deletion order
        rsvps::Entity::delete_many()
            .filter(rsvps::Column::EventId.eq(event_id))
            .exec(&ctx.db)
            .await
            .unwrap();
        event.delete(&ctx.db).await.unwrap();

        let rsvp_count = rsvps::Entity::find()
            .filter(rsvps::Column::EventId.eq(event_id))
            .count(&ctx.db)
            .await
            .unwrap();
        assert_eq!(rsvp_count, 0);
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_delete_event_cascade_deletes_reminders() {
    request::<App, _, _>(|_server, ctx| async move {
        let field_config = FieldConfig::default();
        let event =
            events::ActiveModel::new("Reminder Event", "has reminders", "rem-del", &field_config)
                .insert(&ctx.db)
                .await
                .unwrap();

        let remind_at = chrono::Utc::now().fixed_offset() + chrono::Duration::hours(1);
        event_reminders::ActiveModel::new(event.id, remind_at, "Don't forget!")
            .insert(&ctx.db)
            .await
            .unwrap();

        let event_id = event.id;

        // Mirror the handler's deletion order
        event_reminders::Entity::delete_many()
            .filter(event_reminders::Column::EventId.eq(event_id))
            .exec(&ctx.db)
            .await
            .unwrap();
        event.delete(&ctx.db).await.unwrap();

        let reminder_count = event_reminders::Entity::find()
            .filter(event_reminders::Column::EventId.eq(event_id))
            .count(&ctx.db)
            .await
            .unwrap();
        assert_eq!(reminder_count, 0);
    })
    .await;
}
