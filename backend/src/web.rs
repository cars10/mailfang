use crate::db::{
    DbPool, archive_email, delete_all_emails, delete_email_by_id, get_all_emails,
    get_archived_emails, get_attachment_by_id, get_email_by_id, get_email_stats,
    get_emails_with_attachments, get_raw_data_by_id, get_unread_emails, mark_email_read,
};
use axum::{
    Router,
    extract::{Path, Query, State, ws::WebSocketUpgrade},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Json, Response},
    routing::{get, patch},
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::sync::broadcast;
use tower_http::services::ServeDir;

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
        WebError::Database(err)
    }
}

impl From<std::io::Error> for WebError {
    fn from(err: std::io::Error) -> Self {
        WebError::Io(err)
    }
}

#[derive(serde::Serialize, Clone)]
pub struct WebSocketMessage {
    pub event: String,
    pub email: Option<crate::db::EmailListRecord>,
    pub counts: Option<crate::db::EmailStats>,
}

pub type BroadcastSender = broadcast::Sender<WebSocketMessage>;

#[derive(Clone)]
struct AppState {
    pool: DbPool,
    broadcast: BroadcastSender,
}

#[derive(Deserialize)]
struct ReadRequest {
    read: bool,
}

#[derive(Deserialize)]
struct ArchiveRequest {
    archived: bool,
}

#[derive(serde::Serialize)]
struct PaginationInfo {
    page: u64,
    per_page: u64,
    total_pages: u64,
}

#[derive(serde::Serialize)]
struct EmailListResponse {
    emails: Vec<crate::db::EmailListRecord>,
    counts: crate::db::EmailStats,
    pagination: PaginationInfo,
}

#[derive(Deserialize)]
struct ListQueryParams {
    sort: Option<String>,
    order: Option<String>,
    search: Option<String>,
    page: Option<u64>,
}

pub async fn run_web_server(
    addr: SocketAddr,
    pool: DbPool,
    broadcast: BroadcastSender,
    static_dir: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let app_state = AppState { pool, broadcast };

    let mut app = Router::new()
        .route("/health", get(health_check))
        .route("/api/emails", get(list_emails).delete(delete_all))
        .route("/api/emails/unread", get(list_unread_emails))
        .route(
            "/api/emails/with-attachments",
            get(list_emails_with_attachments),
        )
        .route("/api/emails/archived", get(list_archived_emails))
        .route("/api/emails/{id}", get(get_email).delete(delete_email))
        .route("/api/emails/{id}/read", patch(mark_read))
        .route("/api/emails/{id}/archive", patch(archive_email_endpoint))
        .route("/api/emails/{id}/raw", get(get_raw_email))
        .route("/api/attachments/{id}", get(get_attachment))
        .route("/ws", get(websocket_handler));

    // Only serve static files if STATIC_DIR is provided (production mode)
    if let Some(static_dir) = static_dir {
        let static_path = PathBuf::from(static_dir);
        let assets_path = static_path.join("assets");
        let index_path = static_path.join("index.html");

        app = app
            .nest_service("/assets", ServeDir::new(assets_path))
            .fallback(get(move |state: State<AppState>| {
                let index_path = index_path.clone();
                async move { serve_index_with_path(state, index_path).await }
            }));
    }

    // Always set state at the end
    let app = app.with_state(app_state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Web server listening on {}", addr);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

async fn list_emails(
    State(state): State<AppState>,
    Query(params): Query<ListQueryParams>,
) -> Result<Json<EmailListResponse>, WebError> {
    let page = params.page.unwrap_or(1);
    let per_page = 10;
    let (emails, total_pages) = get_all_emails(
        &state.pool,
        params.sort.as_deref(),
        params.order.as_deref(),
        params.search.as_deref(),
        page,
        per_page,
    )
    .await?;
    let counts = get_email_stats(&state.pool).await?;
    Ok(Json(EmailListResponse {
        emails,
        counts,
        pagination: PaginationInfo {
            page,
            per_page,
            total_pages,
        },
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
    }

    Ok(Json(email))
}

async fn serve_index_with_path(
    State(_state): State<AppState>,
    index_path: PathBuf,
) -> Result<Html<String>, WebError> {
    let content = tokio::fs::read_to_string(&index_path).await?;
    Ok(Html(content))
}

async fn delete_email(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, WebError> {
    let rows_affected = delete_email_by_id(&state.pool, &id).await?;
    if rows_affected > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(WebError::NotFound)
    }
}

async fn delete_all(State(state): State<AppState>) -> Result<StatusCode, WebError> {
    delete_all_emails(&state.pool).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_unread_emails(
    State(state): State<AppState>,
    Query(params): Query<ListQueryParams>,
) -> Result<Json<EmailListResponse>, WebError> {
    let page = params.page.unwrap_or(1);
    let per_page = 10;
    let (emails, total_pages) = get_unread_emails(
        &state.pool,
        params.sort.as_deref(),
        params.order.as_deref(),
        params.search.as_deref(),
        page,
        per_page,
    )
    .await?;
    let counts = get_email_stats(&state.pool).await?;
    Ok(Json(EmailListResponse {
        emails,
        counts,
        pagination: PaginationInfo {
            page,
            per_page,
            total_pages,
        },
    }))
}

async fn list_emails_with_attachments(
    State(state): State<AppState>,
    Query(params): Query<ListQueryParams>,
) -> Result<Json<EmailListResponse>, WebError> {
    let page = params.page.unwrap_or(1);
    let per_page = 10;
    let (emails, total_pages) = get_emails_with_attachments(
        &state.pool,
        params.sort.as_deref(),
        params.order.as_deref(),
        params.search.as_deref(),
        page,
        per_page,
    )
    .await?;
    let counts = get_email_stats(&state.pool).await?;
    Ok(Json(EmailListResponse {
        emails,
        counts,
        pagination: PaginationInfo {
            page,
            per_page,
            total_pages,
        },
    }))
}

async fn list_archived_emails(
    State(state): State<AppState>,
    Query(params): Query<ListQueryParams>,
) -> Result<Json<EmailListResponse>, WebError> {
    let page = params.page.unwrap_or(1);
    let per_page = 10;
    let (emails, total_pages) = get_archived_emails(
        &state.pool,
        params.sort.as_deref(),
        params.order.as_deref(),
        params.search.as_deref(),
        page,
        per_page,
    )
    .await?;
    let counts = get_email_stats(&state.pool).await?;
    Ok(Json(EmailListResponse {
        emails,
        counts,
        pagination: PaginationInfo {
            page,
            per_page,
            total_pages,
        },
    }))
}

async fn mark_read(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<ReadRequest>,
) -> Result<StatusCode, WebError> {
    let rows_affected = mark_email_read(&state.pool, &id, request.read).await?;
    if rows_affected > 0 {
        // Send WebSocket notification that email was updated
        let counts = get_email_stats(&state.pool).await.ok();
        let _ = state.broadcast.send(WebSocketMessage {
            event: "new_email".to_string(),
            email: None,
            counts,
        });
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(WebError::NotFound)
    }
}

async fn archive_email_endpoint(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<ArchiveRequest>,
) -> Result<StatusCode, WebError> {
    let rows_affected = archive_email(&state.pool, &id, request.archived).await?;
    if rows_affected > 0 {
        // Send WebSocket notification that email was updated
        let counts = get_email_stats(&state.pool).await.ok();
        let _ = state.broadcast.send(WebSocketMessage {
            event: "new_email".to_string(),
            email: None,
            counts,
        });
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(WebError::NotFound)
    }
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
        let content_disposition = format!("attachment; filename=\"{}\"", filename);
        headers.insert(
            axum::http::header::CONTENT_DISPOSITION,
            HeaderValue::from_str(&content_disposition)
                .unwrap_or_else(|_| HeaderValue::from_static("attachment")),
        );
    }

    Ok((headers, attachment.data).into_response())
}

async fn health_check() -> StatusCode {
    StatusCode::OK
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
