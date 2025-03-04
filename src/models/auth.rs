use serde::Deserialize;
use surrealdb::RecordId;
use uuid::Uuid;
use crate::config::Config;
use crate::services;

const MAX_NAME_LENGTH: usize = 255;

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

impl User {
    /// Returns the fully qualified Matrix ID (MXID) as in `@name:domain`
    pub fn mxid(self: &Self, config: &Config) -> String {
        services::auth::mxid(&self.name, config)
    }

    /// Validates the `name` part of `@name:domain` according to Matrix spec
    ///
    /// See https://spec.matrix.org/v1.13/appendices/#user-identifiers
    pub fn validate_name(self: &Self, config: &Config) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("User.name cannot be empty.".to_string());
        }

        if self.mxid(config).len() > MAX_NAME_LENGTH {
            return Err("Length of fully qualified User.name cannot exceed 255.".to_string());
        }

        Ok(())
    }
}
