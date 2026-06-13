mod config;
mod metrics;
mod ua;

use std::sync::Arc;
use std::sync::LazyLock;

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
    mobile_prefix_dot: String,
    desktop_prefix_dot: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //
    // version
    //
    if matches!(std::env::args().nth(1).as_deref(), Some("--version" | "-V")) {
        println!("Name: {}", env!("CARGO_PKG_NAME"));
        println!("Version: {}", version());
        println!("Git SHA: {}", *GIT_SHA);
        println!("Rust: {}", rust_version());
        println!("Build Time: {}", *BUILD_TIME);

        return Ok(());
    }
    //
    // JSON structured logging
    //
    tracing_subscriber::fmt()
        .json()
        .flatten_event(true)
        .with_current_span(false)
        .with_span_list(false)
        .with_target(false)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config.toml".to_string());

    let config = Arc::new(Config::load(&config_path)?);

    let listen_addr = config.listen.clone();

    let state = AppState {
        mobile_prefix_dot: format!("{}.", config.mobile_prefix),
        desktop_prefix_dot: format!("{}.", config.desktop_prefix),
        config,
    };

    let app = Router::new()
        .route("/", any(handler))
        .route("/{*path}", any(handler))
        .route("/healthz", axum::routing::get(health_handler))
        .route("/metrics", axum::routing::get(metrics_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;

    tracing::info!(
        listen_addr = listen_addr.as_str(),
        version = version(),
        git_sha = *GIT_SHA,
        event = "startup"
    );

    axum::serve(listener, app).await?;

    Ok(())
}

fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

static GIT_SHA: LazyLock<String> = LazyLock::new(|| {
    option_env!("VERGEN_GIT_SHA")
        .unwrap_or("unknown")
        .chars()
        .take(8)
        .collect()
});

fn rust_version() -> &'static str {
    option_env!("VERGEN_RUSTC_SEMVER").unwrap_or("unknown")
}

static BUILD_TIME: LazyLock<String> = LazyLock::new(|| {
    let ts = option_env!("VERGEN_BUILD_TIMESTAMP").unwrap_or("unknown");

    ts.split('.')
        .next()
        .map(|s| format!("{s}Z"))
        .unwrap_or_else(|| ts.to_string())
});

async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "ok")
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

    //
    // prevent loop redirect
    //
    if host.starts_with(&state.mobile_prefix_dot) || host.starts_with(&state.desktop_prefix_dot) {
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
