#![warn(clippy::nursery, clippy::pedantic)]

mod routes;
mod util;

use crate::routes::{route_servers_csv, route_whereis_csv};
use axum::routing::get;
use axum::Router;
use tokio::signal;

pub static API_BASE_URL: &str = "https://api.serverseeker.net";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let app = Router::new()
        .route("/", get(route_index))
        .route("/whereis/csv", get(route_whereis_csv))
        .route("/servers/csv", get(route_servers_csv));

    let port: u16 = 3000;
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await.unwrap();
    println!("Listening on :{port}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Failed to start server");
}

async fn route_index() -> String {
    include_str!("index.txt").replace("$VERSION", env!("CARGO_PKG_VERSION"))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
        let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}