use crate::gui::App;
use eframe::Frame;
use egui::Context;
use wizardrs_core::client_event::ClientEvent;

impl App {
    pub fn render_lobby_page(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // check if there are enough players to start the game
            let enabled = if let Some(state) = &self.join_page.game_state {
                !(state.players.len() < 3 || state.players.len() > 6)
            } else {
                false
            };
            // start game button
            ui.add_enabled_ui(enabled, |ui| {
                ui.centered_and_justified(|ui| {
                    if ui.button("Start Game").clicked() {
                        self.start_game();
                    }
                });
            });
        });
    }

    /// Send StartGame event to the server
    fn start_game(&self) {
        // TODO migrate to GuiMessage

        if let Some(client) = &self.join_page.client {
            client.send_event(ClientEvent::StartGame);
        }
    }
}
