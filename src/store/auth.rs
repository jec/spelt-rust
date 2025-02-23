use crate::error::Error;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

pub async fn users_stream(db: &Surreal<Client>) {

}

pub async fn create_user(name: &String, email: &String, password: &String, db: &Surreal<Client>) -> Result<(), Error> {
    let result = db.query("CREATE user").await?;

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

        (User {}, String::from("foo"))
    }

    pub async fn create_test_session(user_id: i64, now_offset: i64, db: &Surreal<Any>) -> (Session, String)  {
        (Session {}, String::from("foo"))
    }
}
