use crate::error::Error;
use crate::services::rooms::CreateRoomEvent;
use surrealdb::engine::any::Any;
use surrealdb::Surreal;

pub async fn create_event(
    kind: &String,
    event: &CreateRoomEvent,
    db: &Surreal<Any>
) -> Result<String, Error> {
    Ok("foo".to_string())
}
