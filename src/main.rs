use axum::{
    Extension, Router,
    routing::{get, post},
};
use configs::db;
use std::error::Error;

use crate::handlers::auth::{register_user, sign_in_user};
pub mod configs;
pub mod handlers;
pub mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pool = db::connect_db().await?;
    let app = Router::new()
        .route("/", get(|| async { "Auth Backend Server" }))
        .route("/api/v1/signup", post(register_user))
        .route("/api/v1/login", post(sign_in_user))
        .layer(Extension(pool));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8055").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    println!("☢️ SERVER IS RUNNING ☢️");
    Ok(())
}
