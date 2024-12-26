use crate::card::{color::CardColor, Card};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientEvent {
    SetUsername { username: String },
    SendChatMessage { content: String },
    StartGame,
    MakeBid { bid: u8 },
    SetTrumpColor { color: CardColor },
    PlayCard { card: Card },
    Ready,
}
