use sqlx::PgPool;
use crate::error::Error;

pub async fn create_room(name: &String, pool: &PgPool) -> Result<(), Error> {
    Ok(())
}
