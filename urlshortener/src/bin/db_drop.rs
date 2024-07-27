use sea_orm::{Database, Statement};
use sea_orm_migration::prelude::*;
use std::env;

mod migrator;

#[tokio::main]
async fn main() {
    urlshortener::init_app_state::load_envs();

    let db_name = env::var("DATABASE_NAME").expect("DATABASE_URL is not set in .env file");
    let db_host = env::var("DATABASE_HOST").expect("DATABASE_HOST is not set in .env file");

    drop(db_host, db_name).await.unwrap();
}

async fn drop(db_url: String, db_name: String) -> Result<(), sea_orm::DbErr> {
    tracing::info!("Dropping database `{}` if exist.", db_name);
    let db = Database::connect(db_url).await.unwrap();

    db.execute(Statement::from_string(
        db.get_database_backend(),
        format!("DROP DATABASE IF EXISTS `{}`;", db_name),
    ))
    .await?;
    
    tracing::info!("Database `{}` dropped.", db_name);
    Result::Ok(())
}
