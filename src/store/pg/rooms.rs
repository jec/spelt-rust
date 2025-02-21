use crate::error::Error;
use sqlx::PgPool;

pub async fn create_room(name: &String, pool: &PgPool) -> Result<(), Error> {
    Ok(())
}
