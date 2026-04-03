use async_trait::async_trait;
use axum::Router as AxumRouter;
use loco_rs::{
    app::{AppContext, Hooks, Initializer},
    bgworker::{BackgroundWorker, Queue},
    boot::{create_app, BootResult, StartMode},
    config::Config,
    controller::AppRoutes,
    db::truncate_table,
    environment::Environment,
    task::Tasks,
    Result,
};
use migration::Migrator;
use std::path::Path;
use std::sync::Arc;

use crate::{
    controllers,
    initializers,
    models::_entities::prelude::*,
    sms::{mock::MockProvider, vonage::VonageProvider, SmsProvider},
    tasks::send_reminders::SendRemindersTask,
    workers::{
        send_reminders::SendRemindersWorker,
        send_sms::SendSmsWorker,
    },
};

pub struct App;

#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    fn app_version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        )
    }

    async fn boot(
        mode: StartMode,
        environment: &Environment,
        config: Config,
    ) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment, config).await
    }

    async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
        Ok(vec![Box::new(
            initializers::view_engine::ViewEngineInitializer,
        )])
    }

    async fn after_context(ctx: AppContext) -> Result<AppContext> {
        // Register SMS provider based on environment
        let provider: Arc<dyn SmsProvider> =
            if std::env::var("SMS_PROVIDER").unwrap_or_default() == "mock" {
                Arc::new(MockProvider::new())
            } else {
                let base_url = std::env::var("SMS_API_BASE_URL")
                    .unwrap_or_else(|_| "https://api.nexmo.com/v1".to_string());
                let api_key = std::env::var("SMS_API_KEY").unwrap_or_default();
                let api_secret = std::env::var("SMS_API_SECRET").unwrap_or_default();
                let from_number = std::env::var("SMS_API_FROM_NUMBER").unwrap_or_default();

                if api_key.is_empty() || api_secret.is_empty() {
                    tracing::warn!("SMS_API_KEY or SMS_API_SECRET not set, using mock SMS provider");
                    Arc::new(MockProvider::new())
                } else {
                    Arc::new(VonageProvider::new(&base_url, &api_key, &api_secret, &from_number))
                }
            };

        ctx.shared_store.insert(provider);
        Ok(ctx)
    }

    async fn after_routes(router: AxumRouter, _ctx: &AppContext) -> Result<AxumRouter> {
        use axum::{
            http::{StatusCode, Uri},
            response::{IntoResponse, Redirect},
        };
        Ok(router.fallback(|uri: Uri| async move {
            let path = uri.path();
            if path != "/" && path.ends_with('/') {
                let loc = match uri.query() {
                    Some(q) => format!("{}?{}", path.trim_end_matches('/'), q),
                    None => path.trim_end_matches('/').to_string(),
                };
                return Redirect::permanent(&loc).into_response();
            }
            StatusCode::NOT_FOUND.into_response()
        }))
    }

    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes()
            .add_route(controllers::dashboard::routes())
            .add_route(controllers::rsvp::routes())
            .add_route(controllers::webhooks::routes())
    }

    async fn connect_workers(ctx: &AppContext, queue: &Queue) -> Result<()> {
        queue.register(SendSmsWorker::build(ctx)).await?;
        queue.register(SendRemindersWorker::build(ctx)).await?;
        Ok(())
    }

    fn register_tasks(tasks: &mut Tasks) {
        tasks.register(SendRemindersTask);
        // tasks-inject (do not remove)
    }

    async fn truncate(ctx: &AppContext) -> Result<()> {
        truncate_table(&ctx.db, Events).await?;
        truncate_table(&ctx.db, EventReminders).await?;
        truncate_table(&ctx.db, Rsvps).await?;
        truncate_table(&ctx.db, MagicLinks).await?;
        truncate_table(&ctx.db, SmsOptOuts).await?;
        truncate_table(&ctx.db, ReminderSends).await?;
        Ok(())
    }

    async fn seed(_ctx: &AppContext, _base: &Path) -> Result<()> {
        Ok(())
    }
}
