use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};

use serde::{Deserialize, Serialize};
use tower::{Layer, ServiceBuilder};
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, OnRequest, TraceLayer};
use tracing::Level;

use crate::{services, AppState};
use services::delete_short_url::DeleteResult;

pub(crate) fn create_router(state: AppState) -> Router {
    let trace_layer = TraceLayer::new_for_http()
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO));

    Router::new()
        .route("/", post(create_short_url))
        .route("/:key", get(redirect_to_long_url))
        .route("/:key", delete(delete_short_url))
        .with_state(state)
        .layer(ServiceBuilder::new().layer(trace_layer))
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

async fn delete_short_url(state: State<AppState>, Path(key): Path<String>) -> Response {
    let result = services::delete_short_url::call(&key, &state.conn).await;

    match result {
        Ok(DeleteResult::Deleted) => (StatusCode::OK).into_response(),
        Ok(DeleteResult::NotFound) => (
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
