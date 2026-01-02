use crate::web::AppState;
use axum::Router;

pub fn attach_frontend_routes(app: Router<AppState>) -> Router<AppState> {
    #[cfg(feature = "embed-frontend")]
    {
        return embed::attach_embedded_frontend(app);
    }

    #[cfg(not(feature = "embed-frontend"))]
    {
        app
    }
}

#[cfg(feature = "embed-frontend")]
mod embed {
    use crate::web::{AppState, error::WebError};
    use axum::{Router, response::Html, routing::get};
    use static_serve::embed_assets;

    embed_assets!("../frontend/dist", compress = true);

    pub fn attach_embedded_frontend(app: Router<AppState>) -> Router<AppState> {
        // Merge static router for assets, then add fallback to serve index.html for SPA routing
        app.merge(static_router())
            .fallback(get(serve_embedded_index))
    }

    async fn serve_embedded_index() -> Result<Html<String>, WebError> {
        const INDEX_HTML: &str = include_str!("../../../frontend/dist/index.html");
        Ok(Html(INDEX_HTML.to_string()))
    }
}
