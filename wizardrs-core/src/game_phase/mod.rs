use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Copy, Eq, PartialEq)]
pub enum GamePhase {
    Lobby,
    Bidding,
    Playing,
    Finished,
}
