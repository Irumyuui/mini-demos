use std::net::SocketAddr;

use axum::{Router, routing::get};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = Router::new().route("/", get(root));

    let addr = SocketAddr::from(([127, 0, 0, 1], 11451));
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> &'static str {
    "Hello, world!"
}
