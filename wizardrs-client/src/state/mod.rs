use crate::state::player::Player;
use uuid::Uuid;
use wizardrs_core::card::color::CardColor;
use wizardrs_core::card::value::CardValue;
use wizardrs_core::card::Card;
use wizardrs_core::game_phase::GamePhase;
use wizardrs_core::scoreboard::ScoreBoard;
use wizardrs_core::server_event::ServerEvent;
use wizardrs_core::trump_suit::TrumpSuit;

pub(crate) mod player;

#[derive(Debug, Clone)]
pub struct GameState {
    pub players: Vec<Player>,
    pub hand: Vec<Card>,
    pub game_phase: GamePhase,
    pub server_shutdown: bool,
    pub event_log: Vec<ServerEvent>,
    pub played_cards: Vec<(Uuid, Card)>,
    pub trump_suit: TrumpSuit,
    pub scoreboard: ScoreBoard,
    pub player_on_turn: u8,
    pub self_select_trump_color: bool,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            players: Vec::new(),
            hand: Vec::new(),
            game_phase: GamePhase::Lobby,
            server_shutdown: false,
            event_log: Vec::new(),
            played_cards: Vec::new(),
            trump_suit: TrumpSuit::None,
            scoreboard: ScoreBoard::new(vec![]),
            player_on_turn: 0,
            self_select_trump_color: false,
        }
    }

    /// Update the player list. This operation resets the scoreboard.
    pub fn set_players(&mut self, players: Vec<Player>) {
        self.players = players.clone();

        // update scoreboard
        let players = players.into_iter().map(|p| (p.username, p.uuid)).collect();
        self.set_scoreboard(ScoreBoard::new(players));
    }

    /// Add ServerEvent to event log, e.g. chat messages.
    pub fn push_event_log(&mut self, event: ServerEvent) {
        self.event_log.push(event);
    }

    /// Set game phase
    pub fn set_game_phase(&mut self, game_phase: GamePhase) {
        self.game_phase = game_phase;
    }

    /// Set own hand
    pub fn set_hand(&mut self, hand: Vec<Card>) {
        self.hand = hand;
    }

    pub fn set_trump_suit(&mut self, trump_suit: TrumpSuit) {
        self.trump_suit = trump_suit;
    }

    /// Set scoreboard
    pub fn set_scoreboard(&mut self, scoreboard: ScoreBoard) {
        self.scoreboard = scoreboard
    }

    /// Check if player is last to bid in current round according to scoreboard.
    pub fn is_last_to_bid(&self, uuid: Uuid) -> Option<bool> {
        let current_round = self.scoreboard.current_round;
        let num_players = self.players.len();
        let player_index = self.players.iter().position(|p| p.uuid == uuid);

        match player_index {
            Some(index) => {
                // In round 1, player index 0 is the dealer and the last player to bid.
                // So if the player index == round - 1 then the player is the last player to bid but for larger rounds the bidder jumps from e.g. player index 3 to 0.
                // To avoid this problem we modulo the current round with the number of players.
                // if (current_round % num_players) == 0 -> num_players - 1 is the index as 0 - 1 is the last player
                let last_index = if (current_round as usize % num_players) == 0 {
                    num_players - 1
                } else {
                    (current_round as usize % num_players) - 1
                };
                Some(index == last_index)
            }
            None => None,
        }
    }

    /// Sets the index of the player on turn
    pub fn set_player_on_turn(&mut self, index: u8) {
        self.player_on_turn = index % self.players.len() as u8;
    }

    /// Gets the player whose turn it currently is
    pub fn get_player_on_turn(&self) -> Player {
        self.players[self.player_on_turn as usize].to_owned()
    }

    /// Marks whether self needs to select a trump color
    pub fn set_select_trump_color(&mut self, should_select: bool) {
        self.self_select_trump_color = should_select;
    }

    /// Returns the leading color of the current trick
    pub fn leading_color(&self) -> Option<CardColor> {
        for (_, card) in &self.played_cards {
            match card.value {
                CardValue::Fool => continue,
                CardValue::Simple(_) => return Some(card.color),
                CardValue::Wizard => return None,
            }
        }

        None
    }

    /// Adds an owned card to the played cards of the current trick
    pub fn player_play_card(&mut self, uuid: Uuid, card: Card) {
        self.played_cards.push((uuid, card));
    }
}
