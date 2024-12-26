use crate::{card::Card, game_phase::GamePhase, scoreboard::ScoreBoard, trump_suit::TrumpSuit};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerEvent {
    UpdatePlayerList {
        players: Vec<(String, Uuid)>,
    },
    SetUUID {
        // only used once to tell remote client its own UUID
        uuid: Uuid,
    },
    PlayerChatMessage {
        username: String,
        uuid: Uuid,
        content: String,
    },
    SetGamePhase {
        phase: GamePhase,
    },
    SetHand {
        hand: Vec<Card>,
    },
    SetTrumpSuit {
        trump_suit: TrumpSuit,
    },
    RequestSelectTrumpColor, // if trump suit is wizard ask dealer for trump color
    UpdateScoreBoard {
        scoreboard: ScoreBoard,
    },
    SetPlayerOnTurn {
        index: u8,
    },
    PlayerPlayCard {
        uuid: Uuid,
        card: Card,
    },
    ClearPlayedCards,
}
