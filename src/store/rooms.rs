use crate::error::Error;
use surrealdb::engine::any::Any;
use surrealdb::Surreal;

pub async fn create_room(
    name: &str,
    db: &Surreal<Any>
) -> Result<(), Error> {
    Ok(())
}
