use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240724_000001_create_short_urls_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ShortUrls::Table)
                    .col(ColumnDef::new(ShortUrls::Id).big_integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(ShortUrls::Key).string().not_null().unique_key())
                    .col(ColumnDef::new(ShortUrls::LongUrl).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ShortUrls::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum ShortUrls {
    Table,
    Id,
    Key,
    LongUrl
}