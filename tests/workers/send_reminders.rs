use are_you_going::{
    app::App,
    models::{
        _entities::{event_reminders, events, reminder_sends, rsvps},
        event_reminders as event_reminders_model,
        events::FieldConfig,
        reminder_sends as reminder_sends_model,
        sms_opt_outs,
    },
    workers::send_reminders::{SendRemindersWorker, SendRemindersWorkerArgs},
};
use chrono::Utc;
use loco_rs::{bgworker::BackgroundWorker, testing::request::request};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};

// Helper: insert an event and return it.
async fn insert_event(db: &sea_orm::DatabaseConnection, slug: &str) -> events::Model {
    events::ActiveModel::new("Test Event", "desc", slug, &FieldConfig::default())
        .insert(db)
        .await
        .unwrap()
}

// Helper: insert a reminder with remind_at offset from now in minutes.
async fn insert_reminder(
    db: &sea_orm::DatabaseConnection,
    event_id: i32,
    offset_minutes: i64,
) -> event_reminders::Model {
    let remind_at = (Utc::now() + chrono::Duration::minutes(offset_minutes)).into();
    event_reminders::ActiveModel::new(event_id, remind_at, "Don't forget!")
        .insert(db)
        .await
        .unwrap()
}

// Helper: insert an opted-in RSVP.
async fn insert_rsvp(
    db: &sea_orm::DatabaseConnection,
    event_id: i32,
    phone: &str,
    sms_opt_in: bool,
) -> rsvps::Model {
    rsvps::ActiveModel::new(event_id, "Alice", phone, 1, 0, sms_opt_in, None, None)
        .insert(db)
        .await
        .unwrap()
}

async fn reminder_send_count(db: &sea_orm::DatabaseConnection, reminder_id: i32) -> u64 {
    reminder_sends::Entity::find()
        .filter(reminder_sends::Column::EventReminderId.eq(reminder_id))
        .count(db)
        .await
        .unwrap()
}

async fn fetch_reminder(
    db: &sea_orm::DatabaseConnection,
    id: i32,
) -> event_reminders::Model {
    event_reminders::Entity::find_by_id(id)
        .one(db)
        .await
        .unwrap()
        .unwrap()
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[tokio::test]
#[serial_test::serial]
async fn test_due_reminder_sends_sms_and_marks_sent() {
    request::<App, _, _>(|_server, ctx| async move {
        let event = insert_event(&ctx.db, "due-send-1").await;
        let reminder = insert_reminder(&ctx.db, event.id, -5).await; // 5 minutes ago
        insert_rsvp(&ctx.db, event.id, "+12025550101", true).await;

        SendRemindersWorker::build(&ctx)
            .perform(SendRemindersWorkerArgs {})
            .await
            .unwrap();

        // reminder_sends record created
        assert_eq!(reminder_send_count(&ctx.db, reminder.id).await, 1);

        // reminder marked sent
        assert!(fetch_reminder(&ctx.db, reminder.id).await.sent_at.is_some());
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_future_reminder_not_processed() {
    request::<App, _, _>(|_server, ctx| async move {
        let event = insert_event(&ctx.db, "future-rem-1").await;
        let reminder = insert_reminder(&ctx.db, event.id, 60).await; // 1 hour from now
        insert_rsvp(&ctx.db, event.id, "+12025550102", true).await;

        SendRemindersWorker::build(&ctx)
            .perform(SendRemindersWorkerArgs {})
            .await
            .unwrap();

        assert_eq!(reminder_send_count(&ctx.db, reminder.id).await, 0);
        assert!(fetch_reminder(&ctx.db, reminder.id).await.sent_at.is_none());
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_already_sent_reminder_not_reprocessed() {
    request::<App, _, _>(|_server, ctx| async move {
        let event = insert_event(&ctx.db, "already-sent-1").await;
        let reminder = insert_reminder(&ctx.db, event.id, -5).await;
        let rsvp = insert_rsvp(&ctx.db, event.id, "+12025550103", true).await;

        // Mark the reminder as already sent
        event_reminders_model::mark_sent(&ctx.db, reminder.clone())
            .await
            .unwrap();

        SendRemindersWorker::build(&ctx)
            .perform(SendRemindersWorkerArgs {})
            .await
            .unwrap();

        // No new reminder_sends should be created
        assert_eq!(reminder_send_count(&ctx.db, reminder.id).await, 0);

        // Verify the rsvp wasn't sent to (no reminder_sends for this rsvp)
        let sent = reminder_sends_model::already_sent(&ctx.db, reminder.id, rsvp.id)
            .await
            .unwrap();
        assert!(!sent);
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_opted_out_rsvp_skipped() {
    request::<App, _, _>(|_server, ctx| async move {
        let event = insert_event(&ctx.db, "opt-out-1").await;
        let reminder = insert_reminder(&ctx.db, event.id, -5).await;
        insert_rsvp(&ctx.db, event.id, "+12025550104", false).await; // opted out

        SendRemindersWorker::build(&ctx)
            .perform(SendRemindersWorkerArgs {})
            .await
            .unwrap();

        // No sends recorded (opted out)
        assert_eq!(reminder_send_count(&ctx.db, reminder.id).await, 0);

        // But the reminder itself should still be marked sent (it was processed)
        assert!(fetch_reminder(&ctx.db, reminder.id).await.sent_at.is_some());
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_sms_opt_out_table_blocks_send() {
    request::<App, _, _>(|_server, ctx| async move {
        let event = insert_event(&ctx.db, "opt-out-table-1").await;
        let reminder = insert_reminder(&ctx.db, event.id, -5).await;
        // RSVP says opted-in, but the opt-out table takes priority
        insert_rsvp(&ctx.db, event.id, "+12025550105", true).await;
        sms_opt_outs::upsert_opt_out(&ctx.db, "+12025550105")
            .await
            .unwrap();

        SendRemindersWorker::build(&ctx)
            .perform(SendRemindersWorkerArgs {})
            .await
            .unwrap();

        assert_eq!(reminder_send_count(&ctx.db, reminder.id).await, 0);
        assert!(fetch_reminder(&ctx.db, reminder.id).await.sent_at.is_some());
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_multiple_rsvps_sends_to_opted_in_only() {
    request::<App, _, _>(|_server, ctx| async move {
        let event = insert_event(&ctx.db, "multi-rsvp-1").await;
        let reminder = insert_reminder(&ctx.db, event.id, -5).await;
        let opted_in1 = insert_rsvp(&ctx.db, event.id, "+12025550106", true).await;
        let opted_in2 = insert_rsvp(&ctx.db, event.id, "+12025550107", true).await;
        let opted_out = insert_rsvp(&ctx.db, event.id, "+12025550108", false).await;

        SendRemindersWorker::build(&ctx)
            .perform(SendRemindersWorkerArgs {})
            .await
            .unwrap();

        assert_eq!(reminder_send_count(&ctx.db, reminder.id).await, 2);
        assert!(
            reminder_sends_model::already_sent(&ctx.db, reminder.id, opted_in1.id)
                .await
                .unwrap()
        );
        assert!(
            reminder_sends_model::already_sent(&ctx.db, reminder.id, opted_in2.id)
                .await
                .unwrap()
        );
        assert!(
            !reminder_sends_model::already_sent(&ctx.db, reminder.id, opted_out.id)
                .await
                .unwrap()
        );
        assert!(fetch_reminder(&ctx.db, reminder.id).await.sent_at.is_some());
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_reminder_sends_idempotency() {
    request::<App, _, _>(|_server, ctx| async move {
        let event = insert_event(&ctx.db, "idem-1").await;
        let reminder = insert_reminder(&ctx.db, event.id, -5).await;
        let rsvp = insert_rsvp(&ctx.db, event.id, "+12025550109", true).await;

        // Pre-seed a reminder_send record (simulates partial previous run)
        reminder_sends_model::record_send(&ctx.db, reminder.id, rsvp.id)
            .await
            .unwrap();

        SendRemindersWorker::build(&ctx)
            .perform(SendRemindersWorkerArgs {})
            .await
            .unwrap();

        // Still only one record despite the worker running
        assert_eq!(reminder_send_count(&ctx.db, reminder.id).await, 1);
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_no_rsvps_marks_reminder_sent() {
    request::<App, _, _>(|_server, ctx| async move {
        let event = insert_event(&ctx.db, "no-rsvps-1").await;
        let reminder = insert_reminder(&ctx.db, event.id, -5).await;
        // No RSVPs at all

        SendRemindersWorker::build(&ctx)
            .perform(SendRemindersWorkerArgs {})
            .await
            .unwrap();

        assert_eq!(reminder_send_count(&ctx.db, reminder.id).await, 0);
        assert!(fetch_reminder(&ctx.db, reminder.id).await.sent_at.is_some());
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_multiple_due_reminders_all_processed() {
    request::<App, _, _>(|_server, ctx| async move {
        let event = insert_event(&ctx.db, "multi-rem-1").await;
        let r1 = insert_reminder(&ctx.db, event.id, -10).await;
        let r2 = insert_reminder(&ctx.db, event.id, -1).await;
        insert_rsvp(&ctx.db, event.id, "+12025550110", true).await;

        SendRemindersWorker::build(&ctx)
            .perform(SendRemindersWorkerArgs {})
            .await
            .unwrap();

        assert!(fetch_reminder(&ctx.db, r1.id).await.sent_at.is_some());
        assert!(fetch_reminder(&ctx.db, r2.id).await.sent_at.is_some());
        // One send per reminder
        assert_eq!(reminder_send_count(&ctx.db, r1.id).await, 1);
        assert_eq!(reminder_send_count(&ctx.db, r2.id).await, 1);
    })
    .await;
}
