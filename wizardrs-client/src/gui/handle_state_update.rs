use super::App;
use crate::interaction::{Message, StateUpdate};

impl App {
    pub fn update_state(&mut self) {
        // update state
        while let Ok(update) = self.state_rx.try_recv() {
            match update {
                StateUpdate::WizardClient(client) => {
                    self.join_page.is_loading = false;
                    self.join_page.client = client;
                }
                StateUpdate::WizardServer(server) => {
                    self.host_page.is_loading = false;
                    self.host_page.server = server;
                }
                StateUpdate::GameState(game_state) => {
                    self.join_page.game_state = game_state;
                }
                StateUpdate::ImageCache(cache) => {
                    self.image_cache = cache;
                }
                StateUpdate::FinishedDownloadingAdrianKennard => {
                    self.settings_page.downloading_adrian_kennard = false;

                    let message = Message::RequestUpdateDeckList;
                    self.handle_message(message);
                }
                StateUpdate::UpdateDeckList(deck) => {
                    self.settings_page.deck_paths = deck;
                }
            }
        }
    }
}
