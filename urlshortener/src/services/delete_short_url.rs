use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use crate::{short_urls, ShortUrls};

pub enum DeleteResult {
    NotFound,
    Deleted
}

pub async fn call(key: &str, conn: &DatabaseConnection) -> Result<DeleteResult, sea_orm::DbErr> {
    let record_option = ShortUrls::find()
        .filter(short_urls::Column::Key.eq(key))
        .one(conn)
        .await?;

    if let Some(record) = record_option {
        short_urls::ActiveModel::from(record).delete(conn).await?;
        Ok(DeleteResult::Deleted)
    } else {
        Ok(DeleteResult::NotFound)
    }
}
