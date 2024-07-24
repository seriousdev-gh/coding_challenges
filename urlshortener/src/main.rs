use std::{env};

use axum::{
    extract::State, http::StatusCode, response::IntoResponse, routing::{post}, Json, Router
};
use serde::{Deserialize, Serialize};
use sea_orm::{ActiveValue, ConnectOptions, Database, DatabaseConnection};
use dotenv::dotenv;

mod api;
mod entities;
use entities::{prelude::*, *};
use sea_orm::EntityTrait;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    // let host = env::var("HOST").expect("HOST is not set in .env file");
    // let port = env::var("PORT").expect("PORT is not set in .env file");
    // let server_url = format!("{host}:{port}");

    let mut opt = ConnectOptions::new(db_url.to_owned());
    opt.sqlx_logging_level(tracing::log::LevelFilter::Debug);

    let conn = Database::connect(opt).await.expect("Database connection failed");

    let state = AppState { conn };

    let app = Router::new()
        .route("/api/short", post(create_short_url))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await.expect("Start listen tcp failed");
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.expect("Failed to start server");
}

#[derive(Clone)]
struct AppState {
    conn: DatabaseConnection,
}

async fn create_short_url(
    state: State<AppState>,
    Json(payload): Json<Request>
) -> impl IntoResponse {

    assert!(payload.url.len() > 0);

    let key = generate_key();

    let url_record = short_urls::ActiveModel {
        key: ActiveValue::Set(key.clone()),
        long_url: ActiveValue::Set(payload.url.clone()),
        ..Default::default()
    };

    ShortUrls::insert(url_record).exec(&state.conn).await.unwrap();
    
    let response = Response { 
        key: key.clone(),
        long_url: payload.url,
        short_url: format!("http://localhost/{}", key)
    };

    (StatusCode::CREATED, Json(response))
}

fn generate_key() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    const LEN: usize = 5;
    let mut rng = rand::thread_rng();

    (0..LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[derive(Serialize)]
struct Response {
    key: String,
    long_url: String,
    short_url: String
}


#[derive(Deserialize)]
struct Request {
    url: String
}