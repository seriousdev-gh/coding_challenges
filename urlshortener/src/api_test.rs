use crate::{api::create_router, init_app_state};

use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};

use http_body_util::BodyExt; // for `collect`
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use serde_json::{json, Value};

use tower::ServiceExt; // for `call`, `oneshot`, and `ready`

async fn setup_post_short_url(conn: &DatabaseConnection) {
    conn.execute(Statement::from_string(
        conn.get_database_backend(), 
        "DELETE FROM short_urls;")
    ).await.unwrap();
}

#[tokio::test]
async fn post_short_url() {
    let app_state = init_app_state::call().await;
    setup_post_short_url(&app_state.conn).await;

    let app = create_router(app_state);

    let request = json!({ "url" : "https://example.com" }).to_string();
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(request))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        body,
        json!({ "key": "1GUJlg", "long_url": "https://example.com", "short_url": "http://localhost:3000/1GUJlg" })
    );
}

async fn setup_get_long_url(conn: &DatabaseConnection) {
    conn.execute(Statement::from_string(
        conn.get_database_backend(), 
        "DELETE FROM short_urls;")
    ).await.unwrap();
    conn.execute(Statement::from_string(
        conn.get_database_backend(), 
        "INSERT INTO short_urls (`key`, `long_url`, `created_at`) VALUES ('TESTKEY', 'https://test.url', '2024-07-27T19:00');")
    ).await.unwrap();
}

#[tokio::test]
async fn get_long_url() {
    let app_state = init_app_state::call().await;
    setup_get_long_url(&app_state.conn).await;
    let app = create_router(app_state);

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/TESTKEY")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(response.headers().get("Location").unwrap(), "https://test.url");
}
