use crate::{api::create_router, init_app_state};

use axum::{
    body::Body,
    http::{self, Request, Response, StatusCode},
};

use http_body_util::BodyExt; // for `collect`
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use serde_json::{json, Value};

use tower::ServiceExt; // for `call`, `oneshot`, and `ready`

async fn setup(conn: &DatabaseConnection) {
    conn.execute(Statement::from_string(
        conn.get_database_backend(), 
        "DELETE FROM short_urls;")
    ).await.unwrap();
}

async fn subject() -> Response<Body> {
    init_app_state::load_envs("test");
    let app_state = init_app_state::call().await;
    setup(&app_state.conn).await;

    let app = create_router(app_state);

    let request = json!({ "url" : "https://example.com" }).to_string();
    app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(request))
                .unwrap(),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn post_short_url() {
    let response = subject().await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        body,
        json!({ "key": "1GUJlg", "long_url": "https://example.com", "short_url": "http://localhost:5000/1GUJlg" })
    );
}
