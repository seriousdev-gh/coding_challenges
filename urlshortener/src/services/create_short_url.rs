use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait};
use std::hash::{DefaultHasher, Hash, Hasher};
use crate::{short_urls, ShortUrls};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

// returns generated key
pub async fn call(long_url: String, conn: &DatabaseConnection) -> String {

    let key = generate_key(&long_url);

    let url_record = short_urls::ActiveModel {
        key: ActiveValue::Set(key.clone()),
        long_url: ActiveValue::Set(long_url),
        ..Default::default()
    };

    ShortUrls::insert(url_record).exec(conn).await.unwrap();

    key
}

fn generate_key(url: &str) -> String {
    let mut s = DefaultHasher::new();
    url.hash(&mut s);
    // trim to 32 bits to get shorter key
    let hash = (s.finish() >> 32) as u32;

    URL_SAFE_NO_PAD.encode(hash.to_be_bytes())
}