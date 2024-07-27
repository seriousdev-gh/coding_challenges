#[cfg(test)]
use crate::{api::create_router, init_app_state};

use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};

use http_body_util::BodyExt; // for `collect`
use serde_json::{json, Value};

use tower::ServiceExt; // for `call`, `oneshot`, and `ready`

#[tokio::test]
async fn post_short_url() {
    let app_state = init_app_state::call().await;
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
