use crate::error::Error;
use crate::models::auth::{Session, User};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{Salt, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use serde::{Deserialize, Serialize};
use surrealdb::engine::any::Any;
use surrealdb::{RecordId, Surreal};

#[derive(Debug, Serialize)]
pub struct NewUser {
    pub name: String,
    pub email: String,
    pub encrypted_password: String,
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

pub async fn create_session(
    user_id: &RecordId,
    device_id: &String,
    device_name: &Option<String>,
    db: &Surreal<Any>
) -> Result<Session, Error> {
    Err(Error::Db("Something went wrong".into()))
}

pub async fn validate_session(
    session_uuid: &String,
    db: &Surreal<Any>
) -> Result<Session, Error> {
    Err(Error::Db("Something went wrong".into()))
}

pub async fn invalidate_existing_sessions(
    user_id: &RecordId,
    device_id: &String,
    db: &Surreal<Any>
) -> Result<(), Error> {

    Ok(())
}

pub async fn log_out(
    session_id: &RecordId,
    db: &Surreal<Any>
) -> Result<(), Error> {
    Ok(())
}

pub async fn log_out_all(
    user_id: &RecordId,
    db: &Surreal<Any>
) -> Result<(), Error> {
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::models::auth::{Session, User};
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
    pub async fn create_test_session(user_id: &RecordId, now_offset: i64, db: &Surreal<Any>) -> (Session, String)  {
        let session = Session {
            id: RecordId::from_table_key("session", "one"),
            uuid: uuid::Uuid::new_v4(),
            device_identifier: "foo".to_string(),
            device_name: None,
            user_id: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        (session, String::from("foo"))
    }
}
