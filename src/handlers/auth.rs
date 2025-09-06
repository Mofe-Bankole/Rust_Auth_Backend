use crate::{
    configs::db::init_db,
    models::models::{CreateUser, LoginUser, User},
};
use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    response::Json as ResponseJson,
};
use bcrypt::{DEFAULT_COST, hash};
use chrono::Utc;
use serde::Serialize;
use sqlx::{PgPool, postgres::PgRow};

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            name: user.name,
            email: user.email,
            created_at: user.created_at,
        }
    }
}

pub async fn sign_up_user(
    pool: &PgPool,
    user: &CreateUser,
) -> Result<PgRow, Box<dyn std::error::Error>> {
    init_db();
    // Check if user already exists
    let existing_user = sqlx::query(
        "SELECT id, name, email, password_hash, created_at, updated_at FROM users WHERE email = ?",
    )
    .bind(&user.email)
    .fetch_optional(pool)
    .await?;

    if existing_user.is_some() {
        return Err("User already exists".into());
    }

    // Hash password
    let password_hash = hash(&user.password, DEFAULT_COST)?;

    // Create user
    let new_user = sqlx::query(
        "INSERT INTO users (name, email, password_hash, created_at) VALUES ($1, $2, $3, $4) RETURNING id, name, email, password_hash, created_at, updated_at",
    )
    .bind(&user.name)
    .bind(&user.email)
    .bind(&user.password)
    .fetch_one(pool)
    .await?;

    Ok(new_user)
}

pub async fn register_user(
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<CreateUser>,
) -> Result<ResponseJson<AuthResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    let user = sign_up_user(&pool, &payload).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            ResponseJson(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    // Generate JWT token
    let token = generate_token(&user).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(ErrorResponse {
                error: "Failed to generate token".to_string(),
            }),
        )
    })?;

    Ok(ResponseJson(AuthResponse {
        token,
        user: user.into(),
    }))
}

pub async fn login_user(
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<LoginUser>,
) -> Result<ResponseJson<AuthResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    // Find user by email
    let user = sqlx::query_as!(
        User,
        "SELECT id, name, email, password_hash, created_at, updated_at FROM users WHERE email = $1",
        payload.email
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(ErrorResponse {
                error: "Database error".to_string(),
            }),
        )
    })?;
}
