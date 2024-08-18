use sea_orm_migration::prelude::*;

mod m20240724_000001_create_short_urls_table;

#[allow(dead_code)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240724_000001_create_short_urls_table::Migration)
        ]
    }
}