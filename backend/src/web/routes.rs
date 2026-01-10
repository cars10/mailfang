use axum::{Json, extract::State};

use crate::csp::inject_csp_meta_tag;
use crate::db::{ListParams, ListQuery};
use crate::web::error::WebError;
use crate::web::ws::{WebSocketEvent, WebSocketMessage};
use crate::web::{EmailListResponse, RenderedQueryParams};
use axum::{
    extract::{Path, Query},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};

use crate::db;
use crate::web::AppState;

pub async fn get_counts(
    State(state): State<AppState>,
) -> Result<Json<db::counts::EmailStats>, WebError> {
    let mut conn = state.pool.get()?;
    let counts = db::counts::get_email_counts(&mut conn)?;
    Ok(Json(counts))
}

pub async fn get_emails(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<EmailListResponse>, WebError> {
    let query: ListQuery = params.into();

    let mut conn = state.pool.get()?;
    let (emails, total_pages) = db::emails::get_emails(&mut conn, &query)?;

    let counts = db::counts::get_email_counts(&mut conn)?;

    Ok(Json(EmailListResponse {
        emails,
        counts,
        pagination: super::PaginationInfo::from_query(&query, total_pages),
    }))
}

pub async fn get_emails_by_recipient(
    State(state): State<AppState>,
    Path(recipient): Path<String>,
    Query(params): Query<ListParams>,
) -> Result<Json<EmailListResponse>, WebError> {
    let query: ListQuery = params.into();

    let mut conn = state.pool.get()?;
    let (emails, total_pages) = db::emails::get_emails_by_recipient(&mut conn, &recipient, &query)?;

    let counts = db::counts::get_email_counts(&mut conn)?;

    Ok(Json(EmailListResponse {
        emails,
        counts,
        pagination: super::PaginationInfo::from_query(&query, total_pages),
    }))
}

pub async fn get_email(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<crate::db::EmailRecord>, WebError> {
    let mut conn = state.pool.get()?;
    let mut email = db::email::get_email(&mut conn, &id).map_err(|e| match e {
        db::DbError::Diesel(diesel::result::Error::NotFound) => WebError::NotFound,
        _ => WebError::from(e),
    })?;

    if !email.read {
        db::email::mark_email_read(&mut conn, &id, true)?;
        email.read = true;

        let email_list_record: crate::db::EmailListRecord = email.clone().into();
        state
            .broadcast
            .send(WebSocketMessage {
                event: WebSocketEvent::EmailRead,
                email: Some(email_list_record),
                email_id: None,
                recipients: None,
            })
            .ok();
    }
    Ok(Json(email))
}

pub async fn delete_email(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, WebError> {
    let mut conn = state.pool.get()?;
    let rows_affected = db::email::delete_email(&mut conn, &id)?;

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

pub async fn delete_emails(State(state): State<AppState>) -> Result<StatusCode, WebError> {
    let mut conn = state.pool.get()?;
    db::emails::delete_all_emails(&mut conn)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_raw_email(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, WebError> {
    let mut conn = state.pool.get()?;
    let raw_data = db::email::get_raw_data(&mut conn, &id).map_err(|e| match e {
        db::DbError::Diesel(diesel::result::Error::NotFound) => WebError::NotFound,
        _ => WebError::from(e),
    })?;

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

pub async fn get_rendered_email(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<RenderedQueryParams>,
) -> Result<Response, WebError> {
    let mut conn = state.pool.get()?;
    let rendered_html = db::email::get_rendered_data(&mut conn, &id).map_err(|e| match e {
        db::DbError::Diesel(diesel::result::Error::NotFound) => WebError::NotFound,
        _ => WebError::from(e),
    })?;

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

pub async fn get_attachment(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, WebError> {
    let mut conn = state.pool.get()?;
    let attachment = db::attachment::get_attachment(&mut conn, &id).map_err(|e| match e {
        db::DbError::Diesel(diesel::result::Error::NotFound) => WebError::NotFound,
        _ => WebError::from(e),
    })?;

    let mut headers = HeaderMap::new();
    let content_type = attachment
        .content_type
        .as_deref()
        .unwrap_or("application/octet-stream");
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_str(content_type)
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
