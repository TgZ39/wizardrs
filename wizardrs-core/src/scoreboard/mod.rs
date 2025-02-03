use round_entry::RoundEntry;
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

pub mod round_entry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBoard {
    pub players: Vec<(String, Uuid)>, // (Username, UUID)
    pub rounds: Vec<Vec<RoundEntry>>, // Vector of Rounds containing RoundEntries
    pub current_round: u8,
}

impl ScoreBoard {
    pub fn new(players: Vec<(String, Uuid)>) -> Self {
        let num_rounds = (60 / players.len().max(1)) as u8;
        let num_players = players.len();

        Self {
            players,
            rounds: vec![vec![RoundEntry::default(); num_players]; num_rounds as usize],
            current_round: 1,
        }
    }

    /// Set the round subsequent modifications will happen to.
    /// The round parameter equals the number of cards in a round.
    /// So round 3 means each player has 3 cards.
    pub fn set_current_round(&mut self, round: u8) {
        self.current_round = round;
    }

    /// Sets the bid of a player in the current round.
    pub fn set_bid(&mut self, uuid: Uuid, bid: u8) {
        // get index corresponding to uuid
        let player_index = self.players.iter().position(|(_, id)| *id == uuid);
        if player_index.is_none() {
            return;
        }

        let round_index = self.current_round as usize - 1;

        self.rounds[round_index][player_index.unwrap()].bid = Some(bid);
    }

    /// Increase the won tricks of a player in the current round by 1.
    pub fn increment_won_tricks(&mut self, uuid: Uuid) {
        // get index corresponding to uuid
        let player_index = self.players.iter().position(|(_, id)| *id == uuid);
        if player_index.is_none() {
            return;
        }
        let player_index = player_index.unwrap();

        let round_index = self.current_round as usize - 1;

        let won_tricks = self.rounds[round_index][player_index].won_tricks;
        self.rounds[round_index][player_index].won_tricks = won_tricks.saturating_add(1)
    }

    /// Calculates the scores for the current round and applies them.
    pub fn apply_scores(&mut self) {
        let round_index = self.current_round as usize - 1;

        let self_clone = self.clone();
        for (player_index, entry) in self
            .rounds
            .get_mut(round_index)
            .expect("index should always be valid")
            .iter_mut()
            .enumerate()
        {
            if let Some(bid) = entry.bid {
                let add_score = if bid == entry.won_tricks {
                    // player guessed correctly
                    20 + (bid * 10) as i32
                } else {
                    // play didn't guess correctly
                    let diff = ((bid as i32) - (entry.won_tricks as i32)).abs();
                    diff * -10
                };

                if round_index == 0 {
                    // first round
                    entry.score = Some(add_score);
                } else {
                    // not first round
                    if let Some(prev_round) = self_clone.get_round(round_index as u8) {
                        // there is a previous round so get score from there so add calculated score and set that to the score of the current round
                        if let Some(prev_score) = prev_round[player_index].score {
                            entry.score = Some(prev_score + add_score);
                        } else {
                            error!("previous score does not exist: {self_clone:?}");
                            entry.score = Some(add_score);
                        }
                    } else {
                        error!("previous rounds does not exist: {self_clone:?}");
                    }
                }
            }
        }
    }

    /// Get (Score, Bid, Won Tricks) by round
    pub fn get_round(&self, round: u8) -> Option<Vec<RoundEntry>> {
        self.rounds.get(round as usize - 1).cloned()
    }

    /// Get the current round as (Score, Bid, Won Tricks)
    pub fn get_current_round(&self) -> Option<Vec<RoundEntry>> {
        self.get_round(self.current_round)
    }

    /// Returns the sum of all bids in the current round.
    pub fn sum_bids(&self) -> u32 {
        self.get_current_round()
            .expect("current round should always be valid")
            .iter()
            .fold(0, |acc, entry| acc + entry.bid.unwrap_or(0) as u32)
    }

    /// Get the entry for a player in the current round
    pub fn get_entry(&self, uuid: Uuid) -> Option<RoundEntry> {
        if let Some(current_round) = self.get_current_round() {
            if let Some(index) = self.get_index(uuid) {
                return Some(current_round[index]);
            }
        }

        None
    }

    /// Returns the index of a player given the UUID.
    pub fn get_index(&self, uuid: Uuid) -> Option<usize> {
        self.players.iter().position(|(_, id)| *id == uuid)
    }
}
