use migrator::Migrator;
use sea_orm::{Database, Statement};
use sea_orm_migration::prelude::*;
use std::env;

mod migrator;

#[tokio::main]
async fn main() {
    urlshortener::init_app_state::load_envs();
    tracing_subscriber::fmt::init();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let db_name = env::var("DATABASE_NAME").expect("DATABASE_URL is not set in .env file");
    let db_host = env::var("DATABASE_HOST").expect("DATABASE_HOST is not set in .env file");

    create(db_host, db_name).await.unwrap();
    migrate(db_url).await.unwrap();
}

async fn create(db_url: String, db_name: String) -> Result<(), sea_orm::DbErr> {
    tracing::info!("Creating database `{}` if not exist.", db_name);
    let db = Database::connect(db_url).await.unwrap();

    db.execute(Statement::from_string(
        db.get_database_backend(),
        format!("CREATE DATABASE IF NOT EXISTS `{}`;", db_name),
    ))
    .await?;
    
    tracing::info!("Database `{}` created or already exists.", db_name);
    Result::Ok(())
}

async fn migrate(full_url: String) -> Result<(), sea_orm::DbErr> {
    tracing::info!("Run migrations.");
    let db = Database::connect(full_url).await.unwrap();
    let schema_manager = SchemaManager::new(&db);

    Migrator::refresh(&db).await?;

    assert!(schema_manager.has_table("short_urls").await?);
    tracing::info!("Migration done.");
    Result::Ok(())
}