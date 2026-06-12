mod config;
mod metrics;
mod ua;

use std::sync::Arc;

use axum::{
    Router,
    extract::{OriginalUri, State},
    http::{HeaderMap, StatusCode, header::LOCATION},
    response::{IntoResponse, Response},
    routing::any,
};

use config::Config;
use metrics::REDIRECT_TOTAL;

use tracing_subscriber::EnvFilter;

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //
    // JSON structured logging
    //
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config.toml".to_string());

    let config = Arc::new(Config::load(&config_path)?);

    let listen_addr = config.listen.clone();

    let state = AppState { config };

    let app = Router::new()
        .route("/", any(handler))
        .route("/{*path}", any(handler))
        .route("/metrics", axum::routing::get(metrics_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;

    println!("listen on {}", listen_addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn metrics_handler() -> impl IntoResponse {
    metrics::encode_metrics()
}

async fn handler(
    State(state): State<AppState>,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
) -> Response {
    //
    // Host
    //
    let host = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get("host"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("")
        .trim();

    if host.is_empty() {
        return (StatusCode::BAD_REQUEST, "missing host").into_response();
    }

    let cfg = &state.config;

    let mobile_prefix_dot = format!("{}.", cfg.mobile_prefix);
    let desktop_prefix_dot = format!("{}.", cfg.desktop_prefix);

    //
    // prevent loop redirect
    //
    if host.starts_with(&mobile_prefix_dot) || host.starts_with(&desktop_prefix_dot) {
        return StatusCode::NO_CONTENT.into_response();
    }

    //
    // Query > Cookie > UA
    //
    let mut force_mobile: Option<bool> = None;

    //
    // Query
    //
    if let Some(query) = uri.query() {
        for pair in query.split('&') {
            if let Some((k, v)) = pair.split_once('=')
                && k == "view"
            {
                force_mobile = match v {
                    "mobile" => Some(true),
                    "desktop" => Some(false),
                    _ => None,
                };
                break;
            }
        }
    }

    //
    // Cookie
    //
    if force_mobile.is_none()
        && let Some(cookie_header) = headers.get("cookie")
        && let Ok(cookie_str) = cookie_header.to_str()
    {
        for item in cookie_str.split(';') {
            let item = item.trim();

            if let Some((name, value)) = item.split_once('=')
                && name == cfg.cookie_name
            {
                force_mobile = match value {
                    v if v == cfg.mobile_cookie_value => Some(true),
                    v if v == cfg.desktop_cookie_value => Some(false),
                    _ => None,
                };
                break;
            }
        }
    }

    //
    // UA fallback
    //
    let ua = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let is_mobile = match force_mobile {
        Some(v) => v,
        None => ua::is_mobile(ua),
    };

    let device = if is_mobile { "mobile" } else { "desktop" };

    //
    // build host
    //
    let prefix = if is_mobile {
        &cfg.mobile_prefix
    } else {
        &cfg.desktop_prefix
    };

    let mut target_host = String::with_capacity(prefix.len() + 1 + host.len());

    target_host.push_str(prefix);
    target_host.push('.');
    target_host.push_str(host);

    //
    // path + query (no uri.to_string allocation)
    //
    let path_and_query = uri.path_and_query().map(|v| v.as_str()).unwrap_or("/");

    let mut target_url = String::with_capacity(8 + target_host.len() + path_and_query.len());

    target_url.push_str("https://");
    target_url.push_str(&target_host);
    target_url.push_str(path_and_query);

    //
    // metrics
    //
    REDIRECT_TOTAL.with_label_values(&[device, host]).inc();

    //
    // structured logs (JSON)
    //
    tracing::info!(
        host = host,
        device = device,
        ua = ua,
        target_url = target_url,
        event = "redirect"
    );

    //
    // response code
    //
    let status = match cfg.redirect_code {
        301 => StatusCode::MOVED_PERMANENTLY,
        _ => StatusCode::FOUND,
    };

    (status, [(LOCATION, target_url)]).into_response()
}
