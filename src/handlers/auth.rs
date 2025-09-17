use crate::models::models::ErrorResponse;
use crate::models::models::*;
use axum::{
    extract::{Extension, Json},
    http::StatusCode,
};
use bcrypt::{hash, verify};
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use sqlx::{Error, PgPool, Row, postgres::PgRow};

// Helper function to convert PgRow to User
fn row_to_user(row: PgRow) -> User {
    User {
        id: row.get("id"),
        name: row.get("name"),
        email: row.get("email"),
        password_hash: row.get("password_hash"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
fn error_re
pub async fn sign_up_user(
    pool: &PgPool,
    user: &CreateUser,
) -> Result<User, Box<dyn std::error::Error>> {
    // Check if user already exists
    let existing_user = sqlx::query("SELECT id FROM users WHERE email = $1")
        .bind(&user.email)
        .fetch_optional(pool)
        .await?;

    if existing_user.is_some() {
        return Err("User already exists".into());
    }

    // Hash password
    let password_hash = hash(&user.password, 10)?;

    // Create user
    let new_user_row = sqlx::query(
        "INSERT INTO users (name, email, password_hash, created_at, updated_at) VALUES ($1, $2, $3, NOW(), NOW()) RETURNING id, name, email, password_hash, created_at, updated_at",
    )
    .bind(&user.name)
    .bind(&user.email)
    .bind(password_hash)
    .fetch_one(pool)
    .await?;

    Ok(row_to_user(new_user_row))
}

pub async fn register_user(
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user = sign_up_user(&pool, &payload).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let token = generate_token(&user).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to generate token".to_string(),
            }),
        )
    })?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

// //////////////////////// Logging In User
pub async fn login_user(pool: &PgPool, user: &LoginUser) -> Result<User, sqlx::Error> {
    let user_row = sqlx::query(
        "SELECT id, name, email, password_hash, created_at, updated_at FROM users WHERE email = $1",
    )
    .bind(&user.email)
    .fetch_optional(pool)
    .await?;

    match user_row {
        Some(row) => Ok(row_to_user(row)),
        None => Err(sqlx::Error::RowNotFound),
    }
}

pub async fn sign_in_user(
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<LoginUser>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user = login_user(&pool, &payload).await.map_err(|e| {
        let status = match e {
            Error::RowNotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        return (
            status,
            Json(ErrorResponse {
                error: match e {
                    Error::RowNotFound => "User not found".to_string(),
                    _ => "Internal server error".to_string(),
                },
            }),
        );
    })?;

    // Verify password
    let is_valid = verify(&payload.password, &user.password_hash).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Password verification failed".to_string(),
            }),
        )
    })?;

    if !is_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid credentials".to_string(),
            }),
        ));
    }

    // Generate token
    let token = generate_token(&user).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to generate token".to_string(),
            }),
        )
    })?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

pub fn generate_token(user: &User) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());

    let now = Utc::now();
    let expires_in = Duration::hours(48);

    let claims = Claims {
        sub: user.id.to_string(),
        exp: (now + expires_in).timestamp() as usize,
        iat: now.timestamp() as usize,
        email: user.email.clone(),
    };

    let header = Header::new(Algorithm::HS256);
    let encoding_key = EncodingKey::from_secret(secret.as_ref());

    encode(&header, &claims, &encoding_key)
}
