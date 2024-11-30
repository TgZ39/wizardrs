use crate::state::game_phase::GamePhase;
use crate::state::player::Player;
use uuid::Uuid;
use wizardrs_core::card::Card;
use wizardrs_core::server_event::ServerEvent;

pub(crate) mod game_phase;
pub(crate) mod player;

#[derive(Debug, Clone)]
pub struct GameState {
    pub players: Vec<Player>,
    pub hand: Vec<Card>,
    pub bid: Option<u8>,
    pub phase: GamePhase,
    pub server_shutdown: bool,
    pub event_log: Vec<ServerEvent>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            players: Vec::new(),
            hand: Vec::new(),
            bid: None,
            phase: GamePhase::Lobby,
            server_shutdown: false,
            event_log: Vec::new(),
        }
    }

    pub fn set_players(&mut self, players: Vec<Player>) {
        self.players = players;
    }

    pub fn remove_player(&mut self, uuid: Uuid) {
        for (i, player) in self.players.iter().enumerate() {
            if player.uuid == uuid {
                self.players.remove(i);
                return;
            }
        }
    }

    pub fn push_event_log(&mut self, event: ServerEvent) {
        self.event_log.push(event);
    }
}
