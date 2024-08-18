use std::env;

use sea_orm::{ConnectOptions, Database, DatabaseConnection};

#[derive(Clone)]
pub struct AppState {
    pub conn: DatabaseConnection,
    pub base_url: String,
    pub bind_url: String,
}

pub fn load_envs(env_name: &str) {
    dotenvy::from_filename(format!("env.{}", env_name)).expect(format!("env.{} not found", env_name).as_str());
}

pub async fn call() -> AppState {
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

    AppState {
        conn,
        base_url,
        bind_url,
    }
}
