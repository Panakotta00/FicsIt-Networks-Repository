mod repository;
mod templates;
mod routes;
mod util;

use axum::middleware::from_fn;
use axum::Router;
use axum::routing::get;
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use url::Url;
use crate::repository::Repository;

pub async fn app() -> Result<Router, anyhow::Error> {
    let index_file = std::env::var("FIN_REPO_INDEX").unwrap_or(String::from("../index.zip"));
    let url = std::env::var("FIN_REPO_RAW").unwrap_or(String::from(".."));
    info!("Repository Index File: '{}'", index_file);
    info!("Repository Raw Base URL: '{}'", url);
    let repository = Repository::from_url(&index_file, url).await;

    Ok(Router::new()
        .nest_service("/script", tower_http::services::ServeDir::new("static/script"))
        .nest_service("/styles", tower_http::services::ServeDir::new("static/styles"))
        .route("/", get(routes::get_index))
        .route("/package", get(routes::package::get_package_list))
        .route("/package/:id", get(routes::package::get_package))
        .layer(from_fn(routes::middleware::get_htmx_header))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .with_state(repository))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "FicsIt-Networks-Repository-Website=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let port = std::env::var("PORT").unwrap_or(String::from("3000"));

    let addr = format!("0.0.0.0:{}", port);
    info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app().await.unwrap()).await.unwrap();
}
