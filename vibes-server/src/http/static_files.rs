//! Static file serving for embedded web-ui assets

use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

/// Embedded web-ui assets (compiled into binary)
#[derive(RustEmbed)]
#[folder = "../web-ui/dist/"]
struct WebAssets;

/// Handler for serving static files from embedded assets
///
/// For SPA routing, any path that doesn't match a real file
/// will return index.html so client-side routing can handle it.
pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    // Try to serve the exact path first
    if let Some(response) = serve_file(path) {
        return response;
    }

    // For SPA: serve index.html for any non-file path
    // This enables client-side routing (TanStack Router)
    serve_file("index.html").unwrap_or_else(|| {
        (StatusCode::NOT_FOUND, "Web UI not found").into_response()
    })
}

/// Serve a file from embedded assets
fn serve_file(path: &str) -> Option<Response<Body>> {
    let file = WebAssets::get(path)?;

    // Determine content type from file extension
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime.as_ref())
        .body(Body::from(file.data.into_owned()))
        .ok()
}

