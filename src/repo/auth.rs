use argon2::{Argon2, PasswordHash, PasswordVerifier};
use sqlx::PgPool;
use crate::error::Error;

#[derive(Debug, sqlx::FromRow)]
struct ValidationRow {
    id: i64,
    encrypted_password: String,
}

/// Looks up user by `username` and validates `password`; returns Ok(`user.id`)
/// if user is found and password is valid; else returns Ok(None)
pub async fn validate_user_and_password(username: &String, password: &String, pool: &PgPool) -> Result<Option<i64>, Error> {
    let mut row_option = sqlx::query_as::<_, ValidationRow>("SELECT id, encrypted_password FROM users WHERE name = $1")
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
        Err(err) => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use argon2::password_hash::rand_core::OsRng;
    use argon2::password_hash::{Salt, SaltString};
    use argon2::PasswordHasher;
    use sqlx::Executor;
    use super::*;

    #[sqlx::test]
    async fn test_validate_user_and_password(pool: PgPool) {
        let username = String::from("foobar");
        let email = String::from("foobar@example.com");
        let password = String::from("bazbar");
        let salt_string = SaltString::generate(&mut OsRng);
        let salt: Salt = salt_string.as_ref().try_into().unwrap();
        let argon2 = Argon2::default();
        let hash = argon2.hash_password(password.as_bytes(), salt).unwrap();

        let row = sqlx::query_as::<_, ValidationRow>("INSERT INTO users (name, email, encrypted_password) VALUES ($1, $2, $3) RETURNING id, encrypted_password")
            .bind(&username)
            .bind(&email)
            .bind(hash.to_string())
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(validate_user_and_password(&username, &password, &pool).await.unwrap(), Some(row.id));
    }

    #[sqlx::test]
    async fn test_validate_user_and_password_with_missing(pool: PgPool) {
        let username = String::from("foobar");
        let password = String::from("bazbar");

        assert!(validate_user_and_password(&username, &password, &pool).await.unwrap().is_none());
    }

    #[sqlx::test]
    async fn test_validate_user_and_password_with_mismatch(pool: PgPool) {
        let username = String::from("foobar");
        let email = String::from("foobar@example.com");
        let password = String::from("bazbar");
        let actual_password = String::from("fizzbuzz");
        let salt_string = SaltString::generate(&mut OsRng);
        let salt: Salt = salt_string.as_ref().try_into().unwrap();
        let argon2 = Argon2::default();
        let hash = argon2.hash_password(actual_password.as_bytes(), salt).unwrap();

        sqlx::query("INSERT INTO users (name, email, encrypted_password) VALUES ($1, $2, $3)")
            .bind(&username)
            .bind(&email)
            .bind(hash.to_string())
            .execute(&pool)
            .await
            .unwrap();

        assert!(validate_user_and_password(&username, &password, &pool).await.unwrap().is_none());
    }
}
