use axum::{
    extract::{FromRequest, Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Router,
};

use serde::{Deserialize, Serialize};
use tower::{ServiceBuilder};
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

use crate::{
    error_handler::{AppError, ErrorResponse},
    init_app_state::AppState,
    services,
};
use services::delete_short_url::DeleteResult;

#[cfg(test)]
#[path = "../test/mod.rs"]
mod test;

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
    AppJson(payload): AppJson<CreateShortUrlRequest>,
) -> Result<Response, AppError> {
    if payload.url.is_empty() {
        let response = (
            StatusCode::BAD_REQUEST,
            AppJson(ErrorResponse {
                message: "Provide url".to_string(),
            }),
        );
        return Ok(response.into_response());
    }

    let key = services::create_short_url::call(payload.url.clone(), &state.conn).await?;

    let response = (
        StatusCode::CREATED,
        AppJson(CreateShortUrlResponse {
            key: key.clone(),
            long_url: payload.url,
            short_url: format!("{}/{}", state.base_url, key),
        }),
    );

    Ok(response.into_response())
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

async fn redirect_to_long_url(
    state: State<AppState>,
    Path(key): Path<String>,
) -> Result<Response, AppError> {
    let result = services::find_short_url::call(&key, &state.conn).await?;
    let response = if let Some(short_url) = result {
        (StatusCode::FOUND, [(header::LOCATION, short_url.long_url)]).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            AppJson(ErrorResponse {
                message: "Key not found".to_string(),
            }),
        )
            .into_response()
    };

    Ok(response)
}

async fn delete_short_url(
    state: State<AppState>,
    Path(key): Path<String>,
) -> Result<Response, AppError> {
    let result = services::delete_short_url::call(&key, &state.conn).await?;
    let response = match result {
        DeleteResult::Deleted => (StatusCode::OK).into_response(),
        DeleteResult::NotFound => (
            StatusCode::NOT_FOUND,
            AppJson(ErrorResponse {
                message: "Key not found".to_string(),
            }),
        )
            .into_response(),
    };

    Ok(response)
}

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(AppError))]
struct AppJson<T>(T);

impl<T> IntoResponse for AppJson<T>
where
    axum::Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}
