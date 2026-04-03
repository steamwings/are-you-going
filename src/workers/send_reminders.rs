use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};

use crate::{
    models::{
        _entities::events,
        event_reminders,
        reminder_sends,
        rsvps,
        sms_opt_outs,
    },
    sms::templates as sms_templates,
    workers::send_sms::{SendSmsWorker, SendSmsWorkerArgs},
};

pub struct SendRemindersWorker {
    pub ctx: AppContext,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct SendRemindersWorkerArgs {}

#[async_trait]
impl BackgroundWorker<SendRemindersWorkerArgs> for SendRemindersWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }

    async fn perform(&self, _args: SendRemindersWorkerArgs) -> Result<()> {
        let now = Utc::now().into();
        let due_reminders = event_reminders::find_due_reminders(&self.ctx.db, now)
            .await
            .map_err(|e| Error::wrap(e))?;

        if due_reminders.is_empty() {
            return Ok(());
        }

        tracing::info!(count = due_reminders.len(), "Due reminders found");

        for reminder in due_reminders {
            let event = events::Entity::find_by_id(reminder.event_id)
                .one(&self.ctx.db)
                .await
                .map_err(|e| Error::wrap(e))?;

            let event = match event {
                Some(e) => e,
                None => continue,
            };

            let opted_in = rsvps::find_opted_in_for_event(&self.ctx.db, event.id)
                .await
                .map_err(|e| Error::wrap(e))?;

            for rsvp in &opted_in {
                // Check idempotency
                if reminder_sends::already_sent(&self.ctx.db, reminder.id, rsvp.id)
                    .await
                    .map_err(|e| Error::wrap(e))?
                {
                    continue;
                }

                // Check opt-out
                if sms_opt_outs::is_opted_out(&self.ctx.db, &rsvp.phone_number)
                    .await
                    .map_err(|e| Error::wrap(e))?
                {
                    tracing::info!(
                        phone = %rsvp.phone_number,
                        "Skipping reminder for opted-out number"
                    );
                    continue;
                }

                let sms_body = sms_templates::reminder_sms(&event.name, &reminder.message);

                SendSmsWorker::build(&self.ctx)
                    .perform(SendSmsWorkerArgs {
                        to: rsvp.phone_number.clone(),
                        body: sms_body,
                    })
                    .await?;

                reminder_sends::record_send(&self.ctx.db, reminder.id, rsvp.id)
                    .await
                    .map_err(|e| Error::wrap(e))?;

                tracing::info!(
                    phone = %rsvp.phone_number,
                    event = %event.name,
                    "Reminder SMS sent"
                );
            }

            event_reminders::mark_sent(&self.ctx.db, reminder)
                .await
                .map_err(|e| Error::wrap(e))?;
        }

        Ok(())
    }
}
