use serde::Serialize;
use sqlx::PgPool;
use crate::error::Error;

#[derive(Debug, Serialize)]
pub struct CreateRoomEvent {
    pub r#type: String,
}

pub async fn create_event(
    event_type: &String,
    content: &CreateRoomEvent,
    pool: &PgPool
) -> Result<(), Error> {
    Ok(())
}
