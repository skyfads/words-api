use axum::Router;
use dotenv::dotenv;
use std::net::SocketAddr;

mod services {
    pub mod ai;
    pub mod db;
    pub mod word;
}

mod controllers {
    pub mod word_controller;
}

mod routes {
    pub mod word_routes;
}

mod extra;

use routes::word_routes::word_routes;
use services::db;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    db::init_db().await;
    db::run_migrations().await?;

    let app = Router::new().merge(word_routes());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
