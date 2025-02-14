use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{Salt, SaltString};
use futures_util::stream::BoxStream;
use sqlx::PgPool;
use uuid::Uuid;
use crate::error::Error;
use crate::services;

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

/// Query result used by [`validate_user_and_password()`]
#[derive(Debug, sqlx::FromRow)]
struct ValidationRow {
    id: i64,
    encrypted_password: String,
}

/// Returns a row stream of current users
///
/// The intended use for this in the `cli` module didn't work, so the query is
/// repeated in that function.
/// TODO: Figure out why calling this doesn't work.
pub async fn _users_stream(pool: &PgPool) -> BoxStream<Result<User, sqlx::Error>> {
    sqlx::query_as::<_, User>("SELECT id, name, email, encrypted_password, created_at, updated_at FROM users")
        .fetch(pool)
}

pub async fn create_user(name: &String, email: &String, password: &String, pool: &PgPool) -> Result<(), Error> {
    let salt_string = SaltString::generate(&mut OsRng);
    let salt: Salt = salt_string.as_ref().try_into().unwrap();
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), salt).unwrap().to_string();

    sqlx::query("INSERT INTO users (name, email, encrypted_password) VALUES ($1, $2, $3)")
        .bind(&name)
        .bind(&email)
        .bind(&hash)
        .execute(pool)
        .await?;

    Ok(())
}

/// Looks up user by `username` and validates `password`; returns `Ok(Some(user.id))`
/// if user is found and password is valid; else returns `Ok(None)`
pub async fn validate_user_and_password(username: &String, password: &String, pool: &PgPool) -> Result<Option<i64>, Error> {
    let row_option = sqlx::query_as::<_, ValidationRow>("SELECT id, encrypted_password FROM users WHERE name = $1")
        .bind(username)
        .fetch_optional(pool)
        .await?;

    if row_option.is_none() {
        return Ok(None)
    }

    let row = row_option.unwrap();
    let hash = PasswordHash::try_from(row.encrypted_password.as_str()).unwrap();

    match Argon2::default().verify_password(password.as_bytes(), &hash) {
        Ok(()) => Ok(Some(row.id)),
        Err(_) => Ok(None),
    }
}

/// Deletes any existing Sessions for `user_id` and `device_id`
pub async fn invalidate_existing_sessions(user_id: i64, device_id: &String, pool: &PgPool) -> Result<(), Error> {
    sqlx::query("DELETE FROM sessions WHERE user_id = $1 and device_identifier = $2")
        .bind(user_id)
        .bind(device_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Creates a session and returns `Ok(uuid)`
pub async fn create_session(user_id: i64, device_id: &String, device_name: &Option<String>, pool: &PgPool) -> Result<Session, Error> {
    Ok(
        sqlx::query_as::<_, Session>("\
                INSERT INTO sessions (device_identifier, device_name, user_id) \
                VALUES ($1, $2, $3)\
                RETURNING id, uuid, device_identifier, device_name, user_id, created_at, updated_at")
            .bind(&device_id)
            .bind(&device_name)
            .bind(user_id)
            .fetch_one(pool)
            .await?
    )
}

/// Validates the referenced Session; returns `Ok(sessions.uuid)` on success
pub async fn validate_session(session_uuid: &String, pool: &PgPool) -> Result<Session, Error> {
    if let Some(session) = sqlx::query_as::<_, Session>("SELECT id, uuid, device_identifier, device_name, user_id, created_at, updated_at FROM sessions WHERE uuid::text = $1")
            .bind(session_uuid)
            .fetch_optional(pool)
            .await? {
        Ok(session)
    } else {
        Err(Error::Auth(String::from("Session logged out")))
    }
}

/// Logs out a User by deleting the authenticated Session
pub async fn log_out(session_id: i64, pool: &PgPool) -> Result<(), Error> {
    sqlx::query("DELETE FROM sessions WHERE id = $1")
        .bind(session_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Deletes any existing Sessions for `user_id`, logging out the user from all
/// devices
pub async fn log_out_all(user_id: i64, pool: &PgPool) -> Result<(), Error> {
    sqlx::query("DELETE FROM sessions WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use argon2::password_hash::rand_core::OsRng;
    use argon2::password_hash::{Salt, SaltString};
    use argon2::PasswordHasher;
    use faker_rand::en_us::internet::Domain;
    use faker_rand::en_us::names::FirstName;
    use rand::Rng;
    use super::*;

    #[sqlx::test]
    async fn test_validate_user_and_password(pool: PgPool) {
        let (user, password) = create_test_user(&pool).await;
        assert_eq!(validate_user_and_password(&user.name, &password, &pool).await.unwrap(), Some(user.id));
    }

    #[sqlx::test]
    async fn test_validate_user_and_password_with_missing(pool: PgPool) {
        let username = String::from("foobar");
        let password = String::from("bazbar");

        assert!(validate_user_and_password(&username, &password, &pool).await.unwrap().is_none());
    }

    #[sqlx::test]
    async fn test_validate_user_and_password_with_mismatch(pool: PgPool) {
        let (user, _password) = create_test_user(&pool).await;
        let not_the_password = String::from("foobar");
        assert!(validate_user_and_password(&user.name, &not_the_password, &pool).await.unwrap().is_none());
    }

    #[sqlx::test]
    async fn test_invalidate_existing_sessions(pool: PgPool) {
        let (user, _password) = create_test_user(&pool).await;
        let (session, _jwt) = create_test_session(user.id, 0, &pool).await;

        assert_eq!(count_sessions(&pool).await, 1);

        invalidate_existing_sessions(user.id, &session.device_identifier, &pool).await.unwrap();

        assert_eq!(count_sessions(&pool).await, 0);
    }

    #[sqlx::test]
    async fn test_invalidate_existing_sessions_with_other_user(pool: PgPool) {
        let (user, _password) = create_test_user(&pool).await;
        let (session, _jwt) = create_test_session(user.id, 0, &pool).await;

        invalidate_existing_sessions(user.id + 1, &session.device_identifier, &pool).await.unwrap();

        assert_eq!(count_sessions(&pool).await, 1);
    }

    #[sqlx::test]
    async fn test_create_session (pool: PgPool) {
        let (user, _password) = create_test_user(&pool).await;
        let device_id = Uuid::new_v4().to_string();
        let session = create_session(user.id, &device_id, &None, &pool).await.unwrap();

        assert!(session.id > 0);
    }

    /// Helper function to create a User for testing
    pub async fn create_test_user(pool: &PgPool) -> (User, String) {
        let mut rng = rand::thread_rng();
        let argon2 = Argon2::default();

        let username = rng.gen::<FirstName>().to_string();
        let email = format!("{}@{}", rng.gen::<FirstName>().to_string(), rng.gen::<Domain>().to_string());
        let password = Uuid::new_v4().to_string();
        let salt_string = SaltString::generate(&mut OsRng);
        let salt: Salt = salt_string.as_ref().try_into().unwrap();
        let hash = argon2.hash_password(password.as_bytes(), salt).unwrap();

        (
            sqlx::query_as::<_, User>("\
                    INSERT INTO users (name, email, encrypted_password) \
                    VALUES ($1, $2, $3) \
                    RETURNING id, name, email, encrypted_password, created_at, updated_at")
                .bind(&username)
                .bind(&email)
                .bind(hash.to_string())
                .fetch_one(pool)
                .await
                .unwrap(),
            password
        )
    }

    /// Helper function to create a Session for testing
    pub async fn create_test_session(user_id: i64, jwt_now_offset: i64, pool: &PgPool) -> (Session, String) {
        let device_identifier = Uuid::new_v4().to_string();

        let session = sqlx::query_as::<_, Session>("\
                INSERT INTO sessions (device_identifier, user_id)
                VALUES ($1, $2)
                RETURNING id, uuid, device_identifier, device_name, user_id, created_at, updated_at")
            .bind(&device_identifier)
            .bind(user_id)
            .fetch_one(pool)
            .await
            .unwrap();

        let jwt = services::jwt::create_jwt(&session.uuid.to_string(), jwt_now_offset).unwrap();

        (session, jwt)
    }

    async fn count_sessions(pool: &PgPool) -> i64 {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions")
            .fetch_one(pool)
            .await
            .unwrap();

        row.0
    }
}
