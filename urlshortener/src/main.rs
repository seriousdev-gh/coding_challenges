use std::{env};

use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use dotenv::dotenv;

mod services;
mod api;
mod entities;
use entities::{prelude::*, *};

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{host}:{port}");

    let mut opt = ConnectOptions::new(db_url.to_owned());
    opt.sqlx_logging_level(tracing::log::LevelFilter::Debug);
    let conn = Database::connect(opt).await.expect("Database connection failed");
    let state = AppState { conn };
    let app = api::create_router(state);
    let listener = tokio::net::TcpListener::bind(server_url.clone()).await.expect(&format!("Failed to create listener on {}", server_url));
    tracing::info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.expect("Failed to start server");
}

#[derive(Clone)]
struct AppState {
    conn: DatabaseConnection,
}
