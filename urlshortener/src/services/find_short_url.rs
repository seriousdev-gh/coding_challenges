use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use crate::{short_urls, ShortUrls};

// returns generated key
pub async fn call(key: &str, conn: &DatabaseConnection) -> Result<Option<short_urls::Model>, sea_orm::DbErr> {
    ShortUrls::find()
        .filter(short_urls::Column::Key.eq(key))
        .one(conn)
        .await
}
