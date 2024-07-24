use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait};

use crate::{short_urls, ShortUrls};

// returns generated key
pub async fn call(long_url: String, conn: &DatabaseConnection) -> String {

    let key = generate_key();

    let url_record = short_urls::ActiveModel {
        key: ActiveValue::Set(key.clone()),
        long_url: ActiveValue::Set(long_url),
        ..Default::default()
    };

    ShortUrls::insert(url_record).exec(conn).await.unwrap();

    key
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