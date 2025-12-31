use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::error;

pub type DieselError = diesel::result::Error;

#[derive(Debug)]
pub enum WebError {
    Database(String),
    NotFound,
    Io(std::io::Error),
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            WebError::Database(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", err),
            ),
            WebError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            WebError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IO error".to_string()),
        };
        (status, message).into_response()
    }
}

impl From<DieselError> for WebError {
    fn from(err: DieselError) -> Self {
        error!(component = "web", error = %err, "Internal error");
        WebError::Database(err.to_string())
    }
}

impl From<std::io::Error> for WebError {
    fn from(err: std::io::Error) -> Self {
        WebError::Io(err)
    }
}

impl From<r2d2::Error> for WebError {
    fn from(err: r2d2::Error) -> Self {
        WebError::Database(err.to_string())
    }
}
