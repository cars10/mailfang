use crate::csp::inject_csp_meta_tag;
use crate::db::{
    DbPool, ListParams, ListQuery, delete_all_emails, delete_email_by_id, get_all_emails,
    get_attachment_by_id, get_email_by_id, get_email_stats, get_emails_by_recipient,
    get_raw_data_by_id, mark_email_read,
};
use crate::entities::emails;
use axum::{
    Router,
    extract::{Path, Query, State, ws::WebSocketUpgrade},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::get,
};
#[cfg(feature = "embed-frontend")]
use static_serve::embed_assets;
#[cfg(feature = "embed-frontend")]
embed_assets!("../frontend/dist", compress = true);
#[cfg(feature = "embed-frontend")]
use axum::response::Html;
use futures_util::{SinkExt, StreamExt};
use sea_orm::EntityTrait;
use serde::Deserialize;
use std::net::SocketAddr;
use std::time::Instant;
use tokio::sync::broadcast;
use tracing::{error, info};

#[derive(Debug)]
pub enum WebError {
    Database(sea_orm::DbErr),
    NotFound,
    Io(std::io::Error),
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            WebError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ),
            WebError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            WebError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IO error".to_string()),
        };
        (status, message).into_response()
    }
}

impl From<sea_orm::DbErr> for WebError {
    fn from(err: sea_orm::DbErr) -> Self {
        error!(component = "web", error = ?err, "Database error");
        WebError::Database(err)
    }
}

impl From<std::io::Error> for WebError {
    fn from(err: std::io::Error) -> Self {
        WebError::Io(err)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WebSocketEvent {
    NewMail,
    EmailRead,
    EmailDeleted,
}

#[derive(serde::Serialize, Clone)]
pub struct WebSocketMessage {
    pub event: WebSocketEvent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<crate::db::EmailListRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipients: Option<Vec<String>>,
}

pub type BroadcastSender = broadcast::Sender<WebSocketMessage>;

#[derive(Clone)]
struct AppState {
    pool: DbPool,
    broadcast: BroadcastSender,
}

fn attach_frontend_routes(app: Router<AppState>) -> Router<AppState> {
    #[cfg(feature = "embed-frontend")]
    {
        // Merge static router for assets, then add fallback to serve index.html for SPA routing
        return app
            .merge(static_router())
            .fallback(get(serve_embedded_index));
    }

    #[cfg(not(feature = "embed-frontend"))]
    {
        app
    }
}

#[derive(Deserialize)]
struct RenderedQueryParams {
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
struct EmailListResponse {
    emails: Vec<crate::db::EmailListRecord>,
    counts: crate::db::EmailStats,
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
        .route("/api/emails", get(list_emails).delete(delete_all))
        .route(
            "/api/emails/inbox/{recipient}",
            get(list_emails_by_recipient_endpoint),
        )
        .route("/api/emails/sidebar", get(get_email_stats_endpoint))
        .route("/api/emails/{id}", get(get_email).delete(delete_email))
        .route("/api/emails/{id}/raw", get(get_raw_email))
        .route("/api/emails/{id}/rendered", get(get_rendered_email))
        .route("/api/attachments/{id}", get(get_attachment))
        .route("/ws", get(websocket_handler))
        .layer(axum::middleware::from_fn(log_http_request));

    let app = attach_frontend_routes(app).with_state(app_state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(component = "web", "Web server listening on {}", addr);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

async fn list_emails(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<EmailListResponse>, WebError> {
    let query: ListQuery = params.into();
    let (emails, total_pages) = get_all_emails(&state.pool, &query).await?;
    let counts = get_email_stats(&state.pool).await?;
    Ok(Json(EmailListResponse {
        emails,
        counts,
        pagination: PaginationInfo::from_query(&query, total_pages),
    }))
}

async fn list_emails_by_recipient_endpoint(
    State(state): State<AppState>,
    Path(recipient): Path<String>,
    Query(params): Query<ListParams>,
) -> Result<Json<EmailListResponse>, WebError> {
    let query: ListQuery = params.into();
    let (emails, total_pages) = get_emails_by_recipient(&state.pool, &recipient, &query).await?;
    let counts = get_email_stats(&state.pool).await?;
    Ok(Json(EmailListResponse {
        emails,
        counts,
        pagination: PaginationInfo::from_query(&query, total_pages),
    }))
}

async fn get_email(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<crate::db::EmailRecord>, WebError> {
    let email = get_email_by_id(&state.pool, &id)
        .await?
        .ok_or(WebError::NotFound)?;

    if !email.read {
        mark_email_read(&state.pool, &id, true).await?;
        let updated_email = get_email_by_id(&state.pool, &id)
            .await?
            .ok_or(WebError::NotFound)?;
        let email_list_record = crate::db::EmailListRecord {
            id: updated_email.id.clone(),
            subject: updated_email.subject.clone(),
            date: updated_email.date,
            created_at: updated_email.created_at,
            from: updated_email.from.clone(),
            to: updated_email.to.clone(),
            read: updated_email.read,
            has_attachments: !updated_email.attachments.is_empty(),
        };
        state
            .broadcast
            .send(WebSocketMessage {
                event: WebSocketEvent::EmailRead,
                email: Some(email_list_record),
                email_id: None,
                recipients: None,
            })
            .ok();
        Ok(Json(updated_email))
    } else {
        Ok(Json(email))
    }
}

#[cfg(feature = "embed-frontend")]
async fn serve_embedded_index() -> Result<Html<String>, WebError> {
    const INDEX_HTML: &str = include_str!("../../frontend/dist/index.html");
    Ok(Html(INDEX_HTML.to_string()))
}

async fn delete_email(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, WebError> {
    let rows_affected = delete_email_by_id(&state.pool, &id).await?;
    if rows_affected > 0 {
        state
            .broadcast
            .send(WebSocketMessage {
                event: WebSocketEvent::EmailDeleted,
                email: None,
                email_id: Some(id),
                recipients: None,
            })
            .ok();
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(WebError::NotFound)
    }
}

async fn delete_all(State(state): State<AppState>) -> Result<StatusCode, WebError> {
    delete_all_emails(&state.pool).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_raw_email(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, WebError> {
    let raw_data = get_raw_data_by_id(&state.pool, &id)
        .await?
        .ok_or(WebError::NotFound)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("message/rfc822"),
    );

    let content_disposition = format!("attachment; filename=\"email-{}.eml\"", id);
    headers.insert(
        axum::http::header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&content_disposition)
            .unwrap_or_else(|_| HeaderValue::from_static("attachment")),
    );

    Ok((headers, raw_data).into_response())
}

async fn get_rendered_email(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<RenderedQueryParams>,
) -> Result<Response, WebError> {
    let email_model = emails::Entity::find_by_id(id.clone())
        .one(&*state.pool)
        .await?
        .ok_or(WebError::NotFound)?;

    let rendered_html = email_model.rendered_body_html.ok_or(WebError::NotFound)?;

    // Default to blocking remote content unless explicitly allowed
    let allow_remote_content = params.allow_remote_content.unwrap_or(false);
    let html = if allow_remote_content {
        rendered_html
    } else {
        inject_csp_meta_tag(rendered_html)
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("text/html"),
    );

    Ok((headers, html).into_response())
}

async fn get_attachment(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, WebError> {
    let attachment = get_attachment_by_id(&state.pool, &id)
        .await?
        .ok_or(WebError::NotFound)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_str(&attachment.mime_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );

    if let Some(filename) = &attachment.filename {
        let content_disposition = format!("inline; filename=\"{}\"", filename);
        headers.insert(
            axum::http::header::CONTENT_DISPOSITION,
            HeaderValue::from_str(&content_disposition)
                .unwrap_or_else(|_| HeaderValue::from_static("inline")),
        );
    }

    Ok((headers, attachment.data).into_response())
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

async fn get_email_stats_endpoint(
    State(state): State<AppState>,
) -> Result<Json<crate::db::EmailStats>, WebError> {
    let counts = get_email_stats(&state.pool).await?;
    Ok(Json(counts))
}

async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: axum::extract::ws::WebSocket, state: AppState) {
    let (mut sender, _receiver) = socket.split();
    let mut rx = state.broadcast.subscribe();

    // Forward broadcast messages to the WebSocket client
    while let Ok(msg) = rx.recv().await {
        let json_msg = serde_json::to_string(&msg).unwrap_or_else(|_| "{}".to_string());
        if sender
            .send(axum::extract::ws::Message::Text(json_msg.into()))
            .await
            .is_err()
        {
            break;
        }
    }
}
