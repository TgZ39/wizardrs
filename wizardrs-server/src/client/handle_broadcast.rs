use crate::client::WizardClient;
use std::sync::Arc;
use wizardrs_core::server_event::ServerEvent;

impl WizardClient {
    /// Handle events being broadcast by the server. This is the server side of the client.
    pub async fn handle_broadcast_event(self: &Arc<Self>, event: ServerEvent) {
        match event {
            ServerEvent::UpdatePlayerList { .. } => {
                self.send_event(event);
            }
            ServerEvent::SetUUID { .. } => {}
            ServerEvent::PlayerChatMessage { .. } => {
                self.send_event(event);
            }
            ServerEvent::SetHand { .. } => {
                self.send_event(event);
            }
            ServerEvent::SetGamePhase { .. } => {
                self.send_event(event);
            }
            ServerEvent::SetTrumpSuit { .. } => {
                self.send_event(event);
            }
            ServerEvent::RequestSelectTrumpColor => {
                self.send_event(event);
            }
            ServerEvent::UpdateScoreBoard { .. } => {
                self.send_event(event);
            }
            ServerEvent::SetPlayerOnTurn { .. } => {
                self.send_event(event);
            }
            ServerEvent::PlayerPlayCard { .. } => {
                self.send_event(event);
            }
            ServerEvent::ClearPlayedCards => {
                self.send_event(event);
            }
            ServerEvent::WaitingForReady { .. } => {
                self.send_event(event);
            }
            ServerEvent::PlayerReady { .. } => {
                self.send_event(event);
            }
        }
    }
}
