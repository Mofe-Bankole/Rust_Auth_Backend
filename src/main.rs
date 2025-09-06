use axum::{
    Router,
    routing::{get, post},
};
use configs::db;
use handlers::auth::sign_up_user;
use std::error::Error;
pub mod configs;
pub mod handlers;
pub mod models;
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pool = db::init_db().await?;
    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Auth Backend Server" }))
        .route("/signup", post(sign_up_user(&pool)));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:2049").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
