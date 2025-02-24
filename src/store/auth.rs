use crate::error::Error;
use crate::models::auth::{Session, User};
use serde::{Deserialize, Serialize};
use surrealdb::engine::any::Any;
use surrealdb::{RecordId, Surreal};

#[derive(Debug, Serialize)]
pub struct NewUser<'a> {
    pub name: &'a String,
    pub email: &'a String,
    pub encrypted_password: &'a String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn users_stream(db: &Surreal<Any>) -> Result<Vec<User>, Error> {
    Ok(db.select("user").await?)
}

pub async fn validate_user_and_password(
    username: &String,
    password: &String,
    db: &Surreal<Any>
) -> Result<Option<RecordId>, Error> {
    Ok(None)
}

pub async fn create_user(
    name: &String,
    email: &String,
    password: &String,
    db: &Surreal<Any>
) -> Result<(), Error> {
    // let _ = db.create("user")
    //     .content(NewUser {
    //         name,
    //         email,
    //         encrypted_password: &"foo".to_string(),
    //         created_at: chrono::Utc::now(),
    //         updated_at: chrono::Utc::now(),
    //     })
    //     .await?;

    Ok(())
}

pub async fn get_user(
    user_id: &RecordId,
    db: &Surreal<Any>,
) -> Result<Option<User>, Error> {
    Ok(None)
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
    use surrealdb::engine::any::Any;

    #[tokio::test]
    async fn test_create_user() {

    }

    pub async fn create_test_user(db: &Surreal<Any>) -> (User, String) {
        let user = User {
            id: RecordId::from_table_key("user", "one"),
            email: "foo".to_string(),
            encrypted_password: "foo".to_string(),
            name: "foo".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        (user, String::from("foo"))
    }

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
