mod routes;
mod util;

use crate::routes::{route_servers_csv, route_whereis_csv};
use axum::routing::get;
use axum::Router;

pub static API_BASE_URL: &'static str = "https://api.serverseeker.net";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(route_index))
        .route("/whereis/csv", get(route_whereis_csv))
        .route("/servers/csv", get(route_servers_csv));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn route_index() -> String {
    include_str!("index.txt")
        .replace("$VERSION", env!("CARGO_PKG_VERSION"))
        .to_string()
}
