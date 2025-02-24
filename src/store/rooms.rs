use crate::error::Error;
use surrealdb::engine::any::Any;
use surrealdb::Surreal;

pub async fn create_room(
    name: &String,
    db: &Surreal<Any>
) -> Result<(), Error> {
    Ok(())
}
