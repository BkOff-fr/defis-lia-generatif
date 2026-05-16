//! Sert le dashboard Svelte embarqué (`web-team/build/`) via `rust-embed`.
//!
//! Stratégie de routing :
//!
//! 1. La route exacte demandée correspond à un fichier embed → on le renvoie
//!    avec son MIME guess.
//! 2. Sinon, si le path ressemble à un asset (contient un `.`), 404.
//! 3. Sinon, SPA fallback : on renvoie `index.html` (le routeur client
//!    SvelteKit prendra la suite).
//!
//! Si le bundle n'a pas été buildé (`web-team/build/index.html` absent),
//! le handler renvoie un message explicatif au lieu d'un 404 silencieux.

use axum::{
    body::Body,
    extract::Path,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../web-team/build"]
#[include = "*.html"]
#[include = "*.js"]
#[include = "*.css"]
#[include = "*.json"]
#[include = "*.svg"]
#[include = "*.png"]
#[include = "*.ico"]
#[include = "*.webp"]
#[include = "*.woff2"]
struct WebAssets;

/// Handler de la racine `/`.
pub async fn index() -> Response {
    serve_or_fallback("index.html")
}

/// Handler générique pour tout path `/*rest`.
pub async fn handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    if path.is_empty() {
        return serve_or_fallback("index.html");
    }
    serve_or_fallback(path)
}

/// Catch-all SPA — utilisé pour les routes Svelte client-side comme
/// `/admin/dashboard`. Si le path matche un fichier statique on le renvoie ;
/// sinon on retombe sur `index.html`.
pub async fn fallback(Path(path): Path<String>) -> Response {
    serve_or_fallback(&path)
}

fn serve_or_fallback(path: &str) -> Response {
    match WebAssets::get(path) {
        Some(file) => respond_with_file(path, file.data.into_owned()),
        None => {
            if path.contains('.') {
                // Path qui ressemble à un asset → on ne masque pas en 200.
                StatusCode::NOT_FOUND.into_response()
            } else {
                // SPA fallback.
                match WebAssets::get("index.html") {
                    Some(idx) => respond_with_file("index.html", idx.data.into_owned()),
                    None => bundle_missing_response(),
                }
            }
        },
    }
}

fn respond_with_file(path: &str, bytes: Vec<u8>) -> Response {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let cache = if path.contains("_app/immutable/") {
        // SvelteKit produit des fichiers content-hashed sous _app/immutable/,
        // donc cache 1 an.
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime.as_ref())
        .header(header::CACHE_CONTROL, cache)
        .body(Body::from(bytes))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

fn bundle_missing_response() -> Response {
    let html = r#"<!doctype html>
<html lang="fr"><head><meta charset="utf-8"><title>Sobr.ia — Mode Équipe</title>
<style>body{background:#0a0d0b;color:#f0ece3;font-family:system-ui,sans-serif;padding:40px;line-height:1.5}
code{background:#131815;padding:2px 6px;border-radius:4px;color:#c5f04a}</style></head>
<body>
<h1 style="color:#c5f04a">Sobr.ia — Dashboard équipe</h1>
<p>Le bundle frontend n'a pas été buildé. Lancez :</p>
<pre><code>cd web-team &amp;&amp; npm ci &amp;&amp; npm run build</code></pre>
<p>puis rebuildez le binaire <code>cargo build -p sobria-team-aggregator</code>.</p>
<p>L'API REST <code>/api/v1/*</code> reste fonctionnelle.</p>
</body></html>"#;
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(html))
        .expect("response build")
}
