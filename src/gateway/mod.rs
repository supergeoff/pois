pub mod auth;
pub mod health;
pub mod views;

use std::net::SocketAddr;
use std::path::PathBuf;

use axum::Router;
use axum::routing::get;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::errors::AppError;

use auth::BasicAuth;

pub async fn serve(
    bind_addr: SocketAddr,
    auth: BasicAuth,
    _data_dir: PathBuf,
) -> Result<(), AppError> {
    let protected = Router::new()
        .route("/", get(views::index))
        .layer(axum::middleware::from_fn_with_state(
            auth.clone(),
            auth::middleware,
        ))
        .with_state(auth);

    let app = Router::new()
        .route("/health", get(health::health))
        .merge(protected)
        .layer(TraceLayer::new_for_http());

    let listener = TcpListener::bind(bind_addr).await.map_err(AppError::Bind)?;
    info!(%bind_addr, "pois gateway listening");
    axum::serve(listener, app).await?;
    Ok(())
}
