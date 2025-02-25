use crate::error::Error;
use crate::models::auth::{Session, User};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{Salt, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use serde::{Deserialize, Serialize};
use surrealdb::engine::any::Any;
use surrealdb::{RecordId, Surreal};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct NewUser {
    pub name: String,
    pub email: String,
    pub encrypted_password: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct NewSession {
    pub uuid: Uuid,
    pub device_identifier: String,
    pub device_name: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Query result used by [`validate_user_and_password()`]
#[derive(Debug, Deserialize)]
struct ValidationRow {
    id: RecordId,
    encrypted_password: String,
}

pub async fn users_stream(db: &Surreal<Any>) -> Result<Vec<User>, Error> {
    Ok(db.select("user").await?)
}

pub async fn create_user(
    name: &String,
    email: &String,
    password: &String,
    db: &Surreal<Any>
) -> Result<User, Error> {
    // Why is this necessary?
    let name = name.clone();
    let email = email.clone();

    let salt_string = SaltString::generate(&mut OsRng);
    let salt: Salt = salt_string.as_ref().try_into().unwrap();
    let argon2 = Argon2::default();
    let encrypted_password = argon2.hash_password(password.as_bytes(), salt).unwrap().to_string();
    let now = chrono::Utc::now();

    let user: Option<User> = db.create("user")
        .content(NewUser {
            name,
            email,
            encrypted_password,
            created_at: now,
            updated_at: now,
        })
        .await?;

    user.ok_or(Error::Db("Failed to create user".to_string()))
}

pub async fn get_user_by_record_id(
    user_id: &RecordId,
    db: &Surreal<Any>,
) -> Result<Option<User>, Error> {
    Ok(db.select(user_id).await?)
}

/// Looks up user by `username` and validates `password`; returns `Ok(Some(user.id))`
/// if user is found and password is valid; else returns `Ok(None)`
pub async fn validate_user_and_password(
    username: &String,
    password: &String,
    db: &Surreal<Any>
) -> Result<Option<RecordId>, Error> {
    let row_option: Option<ValidationRow> = db.query("SELECT id, encrypted_password FROM user WHERE name = $name")
        .bind(("name", username.clone()))
        .await?
        .take(0)?;

    if row_option.is_none() {
        return Ok(None)
    }

    let row = row_option.unwrap();
    let hash = PasswordHash::try_from(row.encrypted_password.as_str()).unwrap();

    match Argon2::default().verify_password(password.as_bytes(), &hash) {
        Ok(()) => Ok(Some(row.id)),
        Err(e) => Ok(None),
    }
}

/// Creates a new `Session` related to the user with record ID `user_id` and on
/// success returns the session
pub async fn create_session(
    user_id: &RecordId,
    device_id: &String,
    device_name: &Option<String>,
    db: &Surreal<Any>
) -> Result<Session, Error> {
    let row_option: Option<Session> = db.query("
        LET $sess = (
            CREATE session SET
                uuid = rand::uuid::v4(),
                device_identifier = $device_id,
                device_name = $device_name,
                created_at = time::now(),
                updated_at = time::now()
        );
        RELATE $user -> authed_as -> $sess;
        RETURN $sess;"
    )
        .bind(("device_id", device_id.clone()))
        .bind(("device_name", device_name.clone()))
        .bind(("user", user_id.clone()))
        .await?
        .take(2)?;

    row_option.ok_or(Error::Db("Failed to create session".to_string()))
}

pub async fn validate_session(
    session_uuid: &String,
    db: &Surreal<Any>
) -> Result<(User, Session), Error> {
    let query = r#"
    LET $sess = (SELECT * FROM session WHERE uuid = type::uuid($uuid));
    SELECT * FROM user WHERE -> authed_as -> (session WHERE $sess);
    RETURN $sess;
    "#;

    let mut result = db.query(query)
        .bind(("uuid", session_uuid.clone()))
        .await?;

    let user: Option<User> = result.take(1)?;
    let session: Option<Session> = result.take(2)?;

    if user.is_none() || session.is_none() {
        return Err(Error::Auth("Session logged out".to_string()));
    }

    Ok((user.unwrap(), session.unwrap()))
}

pub async fn invalidate_existing_sessions(
    user_id: &RecordId,
    device_id: &String,
    db: &Surreal<Any>
) -> Result<(), Error> {
    let query = r#"
    DELETE FROM session
    WHERE device_identifier = $device_id
    AND $user_id -> authed_as -> session
    "#;

    let _result = db.query(query)
        .bind(("user_id", user_id.clone()))
        .bind(("device_id", device_id.clone()))
        .await?;

    Ok(())
}

pub async fn log_out(
    session_id: &RecordId,
    db: &Surreal<Any>
) -> Result<(), Error> {
    let _: Option<Session> = db.delete(session_id).await?;
    Ok(())
}

pub async fn log_out_all(
    user_id: &RecordId,
    db: &Surreal<Any>
) -> Result<(), Error> {
    let _ = db.query("DELETE FROM session WHERE $user_id -> authed_as -> session")
        .bind(("user_id", user_id.clone()))
        .await?;
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::models::auth::{Session, User};
    use crate::services;
    use faker_rand::en_us::internet::Domain;
    use faker_rand::en_us::names::FirstName;
    use rand::Rng;
    use surrealdb::engine::any::Any;

    #[tokio::test]
    async fn test_create_user() {
        crate::test::run_with_db(test_create_user_impl).await;
    }

    async fn test_create_user_impl(db: Surreal<Any>) {
        let name = "foo".to_string();
        let email = "bar@example.com".to_string();
        let password = "foobarbaz".to_string();
        let result = create_user(&name, &email, &password, &db).await.unwrap();

        assert_eq!(&result.name, &name);
        assert_eq!(&result.email, &email);
    }

    #[tokio::test]
    async fn test_validate_user_and_password() {
        crate::test::run_with_db(test_validate_user_and_password_impl).await;
    }

    async fn test_validate_user_and_password_impl(db: Surreal<Any>) {
        let (user, password) = create_test_user(&db).await;
        let result = validate_user_and_password(&user.name, &password, &db).await.unwrap();
        assert_eq!(result, Some(user.id));
    }

    #[tokio::test]
    async fn test_validate_user_and_password_with_mismatch() {
        crate::test::run_with_db(test_validate_user_and_password_impl_with_mismatch).await;
    }

    async fn test_validate_user_and_password_impl_with_mismatch(db: Surreal<Any>) {
        let (user, password) = create_test_user(&db).await;
        let not_the_password = "foobar".to_string();
        let result = validate_user_and_password(&user.name, &not_the_password, &db).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_validate_user_and_password_with_missing() {
        crate::test::run_with_db(test_validate_user_and_password_impl_with_missing).await;
    }

    async fn test_validate_user_and_password_impl_with_missing(db: Surreal<Any>) {
        let username = String::from("foobar");
        let password = String::from("bazbar");
        let result = validate_user_and_password(&username, &password, &db).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_user_by_record_id() {
        crate::test::run_with_db(test_get_user_by_record_id_impl).await;
    }

    async fn test_get_user_by_record_id_impl(db: Surreal<Any>) {
        let (user, password) = create_test_user(&db).await;
        let result = get_user_by_record_id(&user.id, &db).await.unwrap();
        assert_eq!(result.unwrap().id, user.id);
    }

    #[tokio::test]
    async fn test_get_user_by_record_id_when_missing() {
        crate::test::run_with_db(test_get_user_by_record_id_when_missing_impl).await;
    }

    async fn test_get_user_by_record_id_when_missing_impl(db: Surreal<Any>) {
        let record_id = RecordId::from_table_key("user", "foo");
        let result = get_user_by_record_id(&record_id, &db).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_create_session() {
        crate::test::run_with_db(test_create_session_impl).await;
    }

    async fn test_create_session_impl(db: Surreal<Any>) {
        let (user, password) = create_test_user(&db).await;
        let device_id = "foobar".to_string();

        let session = create_session(
            &user.id,
            &device_id,
            &None,
            &db
        ).await.unwrap();

        assert_eq!(&session.device_identifier, &device_id);
    }

    #[tokio::test]
    async fn test_validate_session() {
        crate::test::run_with_db(test_validate_session_impl).await;
    }

    async fn test_validate_session_impl(db: Surreal<Any>) {
        let (user, password) = create_test_user(&db).await;
        let (session, _jwt) = create_test_session(&user.id, 0, &db).await;
        let (user_result, session_result) = validate_session(&session.uuid.to_string(), &db).await.unwrap();
        assert_eq!(user_result.id, user.id);
        assert_eq!(session_result.id, session.id);
    }

    #[tokio::test]
    async fn test_validate_session_with_missing() {
        crate::test::run_with_db(test_validate_session_with_missing_impl).await;
    }

    async fn test_validate_session_with_missing_impl(db: Surreal<Any>) {
        let result = validate_session(&uuid::Uuid::new_v4().to_string(), &db).await;
        assert!(result.is_err_and(|err| {
            match err {
                Error::Auth(_) => true,
                _ => false
            }
        }));
    }

    #[tokio::test]
    async fn test_invalidate_existing_sessions() {
        crate::test::run_with_db(test_invalidate_existing_sessions_impl).await;
    }

    async fn test_invalidate_existing_sessions_impl(db: Surreal<Any>) {
        let (user, password) = create_test_user(&db).await;
        let (session_1, _jwt) = create_test_session(&user.id, 0, &db).await;
        let (session_2, _jwt) = create_test_session(&user.id, 0, &db).await;

        // Check that the call succeeded.
        let result = invalidate_existing_sessions(&user.id, &session_1.device_identifier, &db).await;
        assert!(&result.is_ok());

        // Check that session_1 was deleted.
        let session: Option<Session> = db.select(session_1.id).await.unwrap();
        assert!(&session.is_none());

        // Check that session_2 wasn't deleted.
        let session: Option<Session> = db.select(session_2.id).await.unwrap();
        assert!(&session.is_some());
    }

    #[tokio::test]
    async fn test_log_out() {
        crate::test::run_with_db(test_log_out_impl).await;
    }

    async fn test_log_out_impl(db: Surreal<Any>) {
        let (user, password) = create_test_user(&db).await;
        let (session_1, _jwt) = create_test_session(&user.id, 0, &db).await;
        let (session_2, _jwt) = create_test_session(&user.id, 0, &db).await;

        // Check that the call succeeded.
        let result = log_out(&session_1.id, &db).await;
        assert!(&result.is_ok());

        // Check that session_1 was deleted.
        let session: Option<Session> = db.select(session_1.id).await.unwrap();
        assert!(&session.is_none());

        // Check that session_2 wasn't deleted.
        let session: Option<Session> = db.select(session_2.id).await.unwrap();
        assert!(&session.is_some());
    }

    #[tokio::test]
    async fn test_log_out_all() {
        crate::test::run_with_db(test_log_out_all_impl).await;
    }

    async fn test_log_out_all_impl(db: Surreal<Any>) {
        let (user, password) = create_test_user(&db).await;
        let (session_1, _jwt) = create_test_session(&user.id, 0, &db).await;
        let (session_2, _jwt) = create_test_session(&user.id, 0, &db).await;

        // Check that the call succeeded.
        let result = log_out_all(&user.id, &db).await;
        assert!(&result.is_ok());

        // Check that session_1 was deleted.
        let session: Option<Session> = db.select(session_1.id).await.unwrap();
        assert!(&session.is_none());

        // Check that session_2 was deleted.
        let session: Option<Session> = db.select(session_2.id).await.unwrap();
        assert!(&session.is_none());
    }

    /// Helper function that creates a User for testing; returns a tuple of
    /// `User` and the `String` password
    pub async fn create_test_user(db: &Surreal<Any>) -> (User, String) {
        let mut rng = rand::thread_rng();
        let username = rng.gen::<FirstName>().to_string().to_lowercase();
        let email = format!("{}@{}", rng.gen::<FirstName>().to_string(), rng.gen::<Domain>().to_string()).to_lowercase();
        let password = uuid::Uuid::new_v4().to_string();
        let user = create_user(&username, &email, &password, db).await.unwrap();

        (user, password)
    }

    /// Helper function that creates a Session for testing; returns a tuple of
    /// `Session` and the `String` JWT
    pub async fn create_test_session(user_id: &RecordId, jwt_now_offset: i64, db: &Surreal<Any>) -> (Session, String)  {
        let device_identifier = Uuid::new_v4().to_string();
        let session = create_session(&user_id, &device_identifier, &None, db).await.unwrap();
        let jwt = services::jwt::create_jwt(&session.uuid.to_string(), jwt_now_offset).unwrap();
        (session, jwt)
    }
}
