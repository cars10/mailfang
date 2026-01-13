pub mod error;
mod frontend;
pub mod routes;
pub mod ws;

use crate::db::{DbPool, ListQuery};
use axum::{Router, http::StatusCode, routing::get};
use serde::Deserialize;
use tower_http::compression::CompressionLayer;

use std::net::SocketAddr;
use std::time::Instant;
use tracing::info;
use ws::BroadcastSender;

#[derive(Clone)]
pub struct AppState {
    pool: DbPool,
    broadcast: BroadcastSender,
}

#[derive(Deserialize)]
pub struct RenderedQueryParams {
    allow_remote_content: Option<bool>,
}

#[derive(serde::Serialize)]
struct PaginationInfo {
    page: u64,
    per_page: u64,
    total_pages: u64,
}

impl PaginationInfo {
    fn from_query(query: &ListQuery, total_pages: u64) -> Self {
        Self {
            page: query.page,
            per_page: query.per_page,
            total_pages,
        }
    }
}

#[derive(serde::Serialize)]
pub struct EmailListResponse {
    emails: Vec<crate::db::EmailListRecord>,
    counts: crate::db::counts::EmailStats,
    pagination: PaginationInfo,
}

pub async fn run(
    addr: SocketAddr,
    pool: DbPool,
    broadcast: BroadcastSender,
) -> Result<(), Box<dyn std::error::Error>> {
    let app_state = AppState { pool, broadcast };

    let app = Router::new()
        .route("/health", get(|| async { StatusCode::OK }))
        .route("/api/counts", get(routes::get_counts))
        .route(
            "/api/emails",
            get(routes::get_emails).delete(routes::delete_emails),
        )
        .route(
            "/api/emails/inbox/{recipient}",
            get(routes::get_emails_by_recipient),
        )
        .route(
            "/api/emails/{id}",
            get(routes::get_email).delete(routes::delete_email),
        )
        .route("/api/emails/{id}/raw", get(routes::get_raw_email))
        .route("/api/emails/{id}/rendered", get(routes::get_rendered_email))
        .route("/api/attachments/{id}", get(routes::get_attachment))
        .route("/ws", get(ws::websocket_handler))
        .layer(CompressionLayer::new())
        .layer(axum::middleware::from_fn(log_http_request));

    let app = frontend::attach_frontend_routes(app).with_state(app_state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(component = "web", "Web server listening on {}", addr);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

async fn log_http_request(
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = Instant::now();

    let response = next.run(request).await;
    let status = response.status();
    let latency = start.elapsed();

    info!(
        component = "web",
        method = %method,
        uri = %uri,
        status = status.as_u16(),
        latency_ms = latency.as_millis(),
    );

    response
}
