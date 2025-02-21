use uuid::Uuid;

/// Model for database `users` table
#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub encrypted_password: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Model for database `sessions` table
#[derive(Debug, sqlx::FromRow)]
pub struct Session {
    pub id: i64,
    pub uuid: Uuid,
    pub device_identifier: String,
    pub device_name: Option<String>,
    pub user_id: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
