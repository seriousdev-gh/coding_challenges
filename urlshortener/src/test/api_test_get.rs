use crate::{api::create_router, init_app_state};

use axum::{
    body::Body,
    http::{self, Request, Response, StatusCode},
};

 // for `collect`
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};

use tower::ServiceExt; // for `call`, `oneshot`, and `ready`

async fn setup(conn: &DatabaseConnection) {
    conn.execute(Statement::from_string(
        conn.get_database_backend(), 
        "DELETE FROM short_urls;")
    ).await.unwrap();
    conn.execute(Statement::from_string(
        conn.get_database_backend(), 
        "INSERT INTO short_urls (`key`, `long_url`, `created_at`) VALUES ('TESTKEY', 'https://test.url', '2024-07-27T19:00');")
    ).await.unwrap();
}

async fn subject() -> Response<Body> {
    init_app_state::load_envs("test");
    let app_state = init_app_state::call().await;
    setup(&app_state.conn).await;
    let app = create_router(app_state);

    app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/TESTKEY")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
}


#[tokio::test]
async fn get_long_url() {
    let response = subject().await;

    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(response.headers().get("Location").unwrap(), "https://test.url");
}
