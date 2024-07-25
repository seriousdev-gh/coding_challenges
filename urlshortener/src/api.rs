use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::{services, AppState};

pub(crate) fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/api/short", post(create_short_url))
        .route("/:key", get(redirect_to_long_url))
        .with_state(state)
}

async fn create_short_url(
    state: State<AppState>,
    Json(payload): Json<CreateShortUrlRequest>,
) -> Response {
    if payload.url.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Provide url".to_string(),
            }),
        )
            .into_response();
    }

    let result = services::create_short_url::call(payload.url.clone(), &state.conn).await;

    match result {
        Ok(key) => (
            StatusCode::CREATED,
            Json(CreateShortUrlResponse {
                key: key.clone(),
                long_url: payload.url,
                // TODO: use real server url
                short_url: format!("{}/{}", state.base_url, key),
            }),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: err.to_string(),
            }),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct CreateShortUrlRequest {
    url: String,
}

#[derive(Serialize)]
struct CreateShortUrlResponse {
    key: String,
    long_url: String,
    short_url: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

async fn redirect_to_long_url(state: State<AppState>, Path(key): Path<String>) -> Response {
    let result = services::find_short_url::call(&key, &state.conn).await;

    match result {
        Ok(Some(short_url)) => {
            (StatusCode::FOUND, [(header::LOCATION, short_url.long_url)]).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Key not found".to_string(),
            }),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: err.to_string(),
            }),
        )
            .into_response(),
    }
}
