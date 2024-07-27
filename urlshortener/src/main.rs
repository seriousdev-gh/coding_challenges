mod api;
mod entities;
mod error_handler;
mod init_app_state;
mod services;

use entities::{prelude::*, *};

#[tokio::main]
async fn main() {
    let app_state = init_app_state::call().await;
    
    tracing_subscriber::fmt::init();

    let bind_url = app_state.bind_url.clone();
    let app = api::create_router(app_state);
    let listener = tokio::net::TcpListener::bind(&bind_url)
        .await
        .expect(&format!("Failed to create listener on {}", bind_url));
    tracing::info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
