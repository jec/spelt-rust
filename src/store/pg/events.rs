use crate::error::Error;
use serde::Serialize;
use sqlx::PgPool;

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
