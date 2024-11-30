use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum ServerEvent {
    UpdatePlayerList {
        players: Vec<(String, Uuid)>
    },
    SetUUID { uuid: Uuid },
    PlayerChatMessage {
        username: String,
        uuid: Uuid,
        content: String
    }
}
