use crate::client::WizardClient;
use crate::state::player::Player;
use std::sync::Arc;
use tracing::instrument;
use wizardrs_core::card::color::CardColor;
use wizardrs_core::card::value::CardValue;
use wizardrs_core::server_event::ServerEvent;

impl WizardClient {
    /// Handle events being sent from the server to the client
    #[instrument(skip(self))]
    pub(crate) async fn handle_server_event(self: &Arc<Self>, event: ServerEvent) {
        match event {
            ServerEvent::UpdatePlayerList { players } => {
                let players = players
                    .iter()
                    .map(|(username, uuid)| Player {
                        username: username.clone(),
                        uuid: *uuid,
                        is_ready: false,
                    })
                    .collect();

                self.game_state.write().await.set_players(players);
                self.update_game_state().await;
            }
            ServerEvent::SetUUID { .. } => {}
            ServerEvent::PlayerChatMessage { .. } => {
                self.game_state.write().await.push_event_log(event);
                self.update_game_state().await;
            }
            ServerEvent::SetGamePhase { phase } => {
                self.game_state.write().await.set_game_phase(phase);
                self.update_game_state().await;
            }
            ServerEvent::SetHand { mut hand } => {
                // sort hand
                hand.sort_by_key(|card| {
                    let color_score = match card.color {
                        CardColor::Blue => 0,
                        CardColor::Red => 100,
                        CardColor::Green => 200,
                        CardColor::Yellow => 300,
                    };
                    let value_score = match card.value {
                        CardValue::Fool => 14,
                        CardValue::Simple(value) => 14 - value as u32,
                        CardValue::Wizard => 0,
                    };

                    color_score + value_score
                });

                self.game_state.write().await.set_hand(hand);
                self.update_game_state().await;
            }
            ServerEvent::SetTrumpSuit { trump_suit } => {
                self.game_state.write().await.set_trump_suit(trump_suit);
                self.update_game_state().await;
            }
            ServerEvent::RequestSelectTrumpColor => {
                self.game_state.write().await.set_select_trump_color(true);
                self.update_game_state().await;
            }
            ServerEvent::UpdateScoreBoard { scoreboard } => {
                self.game_state.write().await.set_scoreboard(scoreboard);
                self.update_game_state().await;
            }
            ServerEvent::SetPlayerOnTurn { index } => {
                self.game_state.write().await.set_player_on_turn(index);
                self.update_game_state().await;
            }
            ServerEvent::PlayerPlayCard { uuid, card } => {
                // check if self played the card
                if uuid == self.uuid {
                    self.game_state
                        .write()
                        .await
                        .hand
                        .retain(|hand_card| *hand_card != card);
                }

                self.game_state.write().await.player_play_card(uuid, card);
                self.update_game_state().await;
            }
            ServerEvent::ClearPlayedCards => {
                self.game_state.write().await.played_cards.clear();
                self.update_game_state().await;
            }
            ServerEvent::WaitingForReady { waiting } => {
                self.game_state.write().await.waiting_for_ready = waiting;
                self.update_game_state().await;
            }
            ServerEvent::PlayerReady { uuid, ready } => {
                self.game_state
                    .write()
                    .await
                    .players
                    .iter_mut()
                    .for_each(|player| {
                        if player.uuid == uuid {
                            player.is_ready = ready;
                        }
                    });
                self.update_game_state().await;
            }
        }
    }
}
