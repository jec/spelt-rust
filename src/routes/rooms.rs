use crate::extractors::authenticated_user::AuthenticatedUser;
use crate::{services, AppState};
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use serde::{Deserialize, Serialize};
use twelf::reexports::serde_json;

#[derive(Debug, Deserialize)]
pub struct CreateRoomRequest {
    creation_content: serde_json::Value,
    initial_state: Option<StateEvent>,
    invite: Vec<String>,
    invite_3pid: Option<Invite3Pid>,
    is_direct: Option<bool>,
    name: Option<String>,
    // power_level_content_override: Option<String>, // TODO
    preset: Option<String>,
    room_alias_name: Option<String>,
    room_version: Option<String>,
    topic: Option<String>,
    visibility: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StateEvent {
    content: String,
    r#type: String,
    state_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Invite3Pid {
    address: String,
    id_access_token: String,
    id_server: String,
    medium: String,
}

#[derive(Debug, Serialize)]
struct CreateRoomSuccess {
    room_id: String,
}

#[post("/_matrix/client/v3/createRoom")]
async fn create_room(
    auth: AuthenticatedUser,
    creation_request: web::Json<CreateRoomRequest>,
    state: web::Data<AppState>
) -> impl Responder {
    match services::rooms::create_room(creation_request.into_inner(), &auth.user_id, state.as_ref()).await {
        Ok(room_id) =>
            HttpResponse::Ok().json(CreateRoomSuccess { room_id }),
        Err(err) =>
            err.error_response(),
    }
}

#[cfg(test)]
mod tests {}
