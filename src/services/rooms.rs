use crate::error::Error;
use crate::routes::rooms::CreateRoomRequest;
use crate::{store, AppState};
use serde::Serialize;
use surrealdb::RecordId;
use twelf::reexports::log;
use uuid::Uuid;
use crate::models::auth::User;

#[derive(Debug, Serialize)]
pub struct CreateRoomEvent {
    pub r#type: String,
}

impl From<CreateRoomRequest> for CreateRoomEvent {
    fn from(request: CreateRoomRequest) -> Self {
        Self {
            r#type: "m.room.create".to_string(),
        }
    }
}

/// Creates a new Room and returns `Ok(room_id)`
pub async fn create_room(request: CreateRoomRequest, user: &User, state: &AppState) -> Result<String, Error> {
    let db = state.db.clone();
    let room_id = format!("!{}:{}", Uuid::new_v4(), state.config.server.homeserver_name);
    let event = CreateRoomEvent::from(request);

    store::rooms::create_room(&room_id, &db).await?;
    store::events::create_event(&event.r#type, &event, &db).await?;

    Ok(room_id)
}
