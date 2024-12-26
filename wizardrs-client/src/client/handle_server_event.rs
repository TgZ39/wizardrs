use crate::client::WizardClient;
use crate::state::player::Player;
use std::sync::Arc;
use wizardrs_core::server_event::ServerEvent;

impl WizardClient {
    /// Handle events being sent from the server to the client
    pub(crate) async fn handle_server_event(self: &Arc<Self>, event: ServerEvent) {
        match event {
            ServerEvent::UpdatePlayerList { players } => {
                let players = players
                    .iter()
                    .map(|(username, uuid)| Player {
                        username: username.clone(),
                        uuid: *uuid,
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
            ServerEvent::SetHand { hand } => {
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
        }
    }
}
