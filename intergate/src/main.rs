use askama::Template;
use axum::{Router, extract::ConnectInfo, response::Html, routing::get};
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{fmt::time::UtcTime, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct GateState;

#[tokio::main]
async fn main() {
    println!("intergate started.");
    let tracing_timer = UtcTime::new(time::macros::format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]Z"
    ));
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("INTERGATE_LOG_LEVEL")
                .unwrap_or_else(|_| "intergate=info,tower_http=info".into()),
        ))
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(true)
                .with_timer(tracing_timer)
                .with_writer(std::io::stdout),
        )
        .init();

    info!("Logger initialized.");
    let app_state = Arc::new(GateState);
    let app = Router::new()
        .route("/", get(index_handler))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(app_state);
    /*
    setcap 'cap_net_bind_service=+ep' 8inary
    */
    let addr = SocketAddr::from(([0, 0, 0, 0], 443));
    let pem_path = "/etc/letsencrypt/live/8inary.com";
    let config = RustlsConfig::from_pem_file(
        PathBuf::from(format!("{pem_path}/fullchain.pem")),
        PathBuf::from(format!("{pem_path}/privkey.pem")),
    )
    .await
    .expect("Failed to load TLS certificates");

    info!("intergate listening on: {}", addr);
    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    ip_address: &'a str,
}

async fn index_handler(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> Html<String> {
    Html(
        IndexTemplate {
            ip_address: &addr.ip().to_string(),
        }
        .render()
        .unwrap(),
    )
}
