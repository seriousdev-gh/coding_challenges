use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use std::hash::{DefaultHasher, Hash, Hasher};
use crate::{short_urls, ShortUrls};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

// returns generated key
pub async fn call(long_url: String, conn: &DatabaseConnection) -> Result<String, DbErr> {

    let key = generate_key(&long_url);

    let existing_record = ShortUrls::find()
        .filter(short_urls::Column::Key.eq(&key))
        .one(conn)
        .await?;

    if existing_record.is_some() {
        // TODO: add check for hash collision, in that case try to generate different hash
        return Ok(key);
    }

    let url_record = short_urls::ActiveModel {
        key: ActiveValue::Set(key.clone()),
        long_url: ActiveValue::Set(long_url),
        ..Default::default()
    };

    ShortUrls::insert(url_record).exec(conn).await?;

    Ok(key)
}

fn generate_key(url: &str) -> String {
    let mut s = DefaultHasher::new();
    url.hash(&mut s);
    // trim to 32 bits to get shorter key
    let hash = (s.finish() >> 32) as u32;

    URL_SAFE_NO_PAD.encode(hash.to_be_bytes())
}