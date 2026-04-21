use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

use crate::VERSION;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    version: &'a str,
}

pub async fn index() -> Response {
    let template = IndexTemplate { version: VERSION };
    match template.render() {
        Ok(body) => Html(body).into_response(),
        Err(err) => {
            tracing::error!(?err, "failed to render index template");
            (StatusCode::INTERNAL_SERVER_ERROR, "template error").into_response()
        }
    }
}
