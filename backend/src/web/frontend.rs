use axum::Router;
#[cfg(feature = "embed-frontend")]
use static_serve::embed_assets;
#[cfg(feature = "embed-frontend")]
embed_assets!("../frontend/dist", compress = true);
#[cfg(feature = "embed-frontend")]
use axum::response::Html;

use crate::web::AppState;

pub fn attach_frontend_routes(app: Router<AppState>) -> Router<AppState> {
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

#[cfg(feature = "embed-frontend")]
async fn serve_embedded_index() -> Result<Html<String>, WebError> {
    const INDEX_HTML: &str = include_str!("../../frontend/dist/index.html");
    Ok(Html(INDEX_HTML.to_string()))
}
