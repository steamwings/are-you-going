#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;

mod m20250329_000001_create_events;
mod m20250329_000002_create_event_reminders;
mod m20250329_000003_create_rsvps;
mod m20250329_000004_create_magic_links;
mod m20250329_000005_create_sms_opt_outs;
mod m20250329_000006_create_reminder_sends;
mod m20250329_000007_add_allergies_text_to_rsvps;
mod m20250402_000001_add_sent_at_to_event_reminders;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250329_000001_create_events::Migration),
            Box::new(m20250329_000002_create_event_reminders::Migration),
            Box::new(m20250329_000003_create_rsvps::Migration),
            Box::new(m20250329_000004_create_magic_links::Migration),
            Box::new(m20250329_000005_create_sms_opt_outs::Migration),
            Box::new(m20250329_000006_create_reminder_sends::Migration),
            Box::new(m20250329_000007_add_allergies_text_to_rsvps::Migration),
            Box::new(m20250402_000001_add_sent_at_to_event_reminders::Migration),
        ]
    }
}
