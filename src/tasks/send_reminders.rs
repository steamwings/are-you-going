use loco_rs::{
    app::AppContext,
    bgworker::BackgroundWorker,
    task::{Task, TaskInfo},
    Result,
};

use crate::workers::send_reminders::{SendRemindersWorker, SendRemindersWorkerArgs};

pub struct SendRemindersTask;

#[async_trait::async_trait]
impl Task for SendRemindersTask {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "SendRemindersWorker".to_string(),
            detail: "Send scheduled event reminders".to_string(),
        }
    }

    async fn run(&self, app_context: &AppContext, _vars: &loco_rs::task::Vars) -> Result<()> {
        SendRemindersWorker::build(app_context)
            .perform(SendRemindersWorkerArgs {})
            .await
    }
}
