use axum::{
    extract::Request,
    http::header,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};

pub const COOKIE_NAME: &str = "dashboard_auth";
pub const COOKIE_VALUE: &str = "authenticated";

pub fn is_authenticated(request: &Request) -> bool {
    request
        .headers()
        .get_all(header::COOKIE)
        .iter()
        .any(|value| {
            value
                .to_str()
                .unwrap_or("")
                .split(';')
                .any(|c| {
                    let c = c.trim();
                    c == format!("{COOKIE_NAME}={COOKIE_VALUE}").as_str()
                })
        })
}

pub async fn require_auth(request: Request, next: Next) -> Response {
    if is_authenticated(&request) {
        next.run(request).await
    } else {
        Redirect::to("/dashboard/login").into_response()
    }
}
