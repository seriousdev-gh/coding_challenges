mod api;
mod entities;
mod error_handler;
mod init_app_state;
mod services;

use std::{env, process::exit};

use entities::{prelude::*, *};

mod migrator;
mod db_drop;
mod db_setup;

#[tokio::main]
async fn main() {

    let command = env::args().nth(1).expect("Command not specified");
    let env_name = env::args().nth(2).expect("Env name not specified");

    init_app_state::load_envs(env_name.as_str());
    tracing_subscriber::fmt::init();

    match command.as_str() {
        "db_drop" => {
            db_drop::call().await;
            exit(0);
        }
        "db_setup" => {
            db_setup::call().await;
            exit(0);
        }
        "serve" => {
            let app_state = init_app_state::call().await;
            let bind_url = app_state.bind_url.clone();
            let app = api::create_router(app_state);
            let listener = tokio::net::TcpListener::bind(&bind_url)
                .await
                .expect(&format!("Failed to create listener on {}", bind_url));
            tracing::info!("Listening on {}", listener.local_addr().unwrap());
            axum::serve(listener, app)
                .await
                .expect("Failed to start server");

            exit(0);    
        }
        _ => {
            println!("Unknown command: {}", command);
            exit(1);    
        }
    }



}

#[cfg(test)]
mod test;