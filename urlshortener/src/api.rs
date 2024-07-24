use axum::{extract::State, http::StatusCode, response::{IntoResponse, Response}, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

use crate::{services, AppState};

pub(crate) fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/api/short", post(create_short_url))
        .with_state(state)
}

async fn create_short_url(
    state: State<AppState>,
    Json(payload): Json<CreateShortUrlRequest>
) -> Response  {

    if payload.url.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(CreateShortUrlError { error: "Provide url".to_string() })).into_response();
    }

    let key = services::create_short_url::call(payload.url.clone(), &state.conn).await;
    
    let response = CreateShortUrlResponse { 
        key: key.clone(),
        long_url: payload.url,
        short_url: format!("http://localhost/{}", key)
    };

    (StatusCode::CREATED, Json(response)).into_response()
}

#[derive(Deserialize)]
struct CreateShortUrlRequest {
    url: String
}


#[derive(Serialize)]
struct CreateShortUrlResponse {
    key: String,
    long_url: String,
    short_url: String
}

#[derive(Serialize)]
struct CreateShortUrlError {
    error: String
}