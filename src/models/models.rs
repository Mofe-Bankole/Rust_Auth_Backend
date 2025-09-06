use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct CreateUser {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

impl User {
    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_email(&self) -> &str {
        &self.email
    }

    pub fn get_created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

impl CreateUser {
    pub fn new(name: String, email: String, password: String) -> Self {
        Self {
            name,
            email,
            password,
        }
    }
}

impl LoginUser {
    pub fn new(email: String, password: String) -> Self {
        Self { email, password }
    }
}
