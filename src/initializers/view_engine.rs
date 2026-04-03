use async_trait::async_trait;
use axum::{Extension, Router as AxumRouter};
use loco_rs::{
    app::{AppContext, Initializer},
    controller::views::{engines, ViewEngine},
    Result,
};

#[allow(clippy::module_name_repetitions)]
pub struct ViewEngineInitializer;

#[async_trait]
impl Initializer for ViewEngineInitializer {
    fn name(&self) -> String {
        "view-engine".to_string()
    }

    async fn after_routes(&self, router: AxumRouter, _ctx: &AppContext) -> Result<AxumRouter> {
        let tera_engine = engines::TeraView::build()?.post_process(|t| {
            t.register_function(
                "app_version",
                |_args: &std::collections::HashMap<String, tera::Value>| {
                    Ok(tera::to_value(env!("CARGO_PKG_VERSION"))
                        .map_err(|e| tera::Error::msg(e.to_string()))?)
                },
            );
            Ok(())
        })?;
        Ok(router.layer(Extension(ViewEngine::from(tera_engine))))
    }
}
