use std::env;

use dotenv::dotenv;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
mod api;
mod entities;
mod services;
use entities::{prelude::*, *};

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let base_url = format!("{host}:{port}");
    let bind_url = format!("0.0.0.0:{port}");

    let mut opt = ConnectOptions::new(db_url.to_owned());
    opt.sqlx_logging_level(tracing::log::LevelFilter::Debug);
    let conn = Database::connect(opt)
        .await
        .expect("Database connection failed");
    let state = AppState { conn, base_url };
    let app = api::create_router(state);
    let listener = tokio::net::TcpListener::bind(bind_url.clone())
        .await
        .expect(&format!("Failed to create listener on {}", bind_url));
    tracing::info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

#[derive(Clone)]
struct AppState {
    conn: DatabaseConnection,
    base_url: String,
}
