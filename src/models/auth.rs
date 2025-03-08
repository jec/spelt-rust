use serde::Deserialize;
use surrealdb::RecordId;
use uuid::Uuid;

/// Model for database `user` node
#[derive(Clone, Debug, Deserialize)]
pub struct User {
    pub id: RecordId,
    /// The `name` part of a Matrix user ID: `@name:domain`
    pub name: String,
    pub email: String,
    pub encrypted_password: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Model for database `session` node
#[derive(Clone, Debug, Deserialize)]
pub struct Session {
    pub id: RecordId,
    pub uuid: Uuid,
    pub device_identifier: String,
    pub device_name: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
