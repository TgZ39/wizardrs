use crate::gui::App;
use crate::state::GameState;
use crate::{client::WizardClient, interaction::GuiMessage};
use eframe::Frame;
use egui::Context;
use std::sync::Arc;

pub(crate) mod game_page;

pub struct JoinPage {
    pub game_state: Option<GameState>,
    url: String,
    username: String,
    pub is_loading: bool,
    pub client: Option<Arc<WizardClient>>,
    chat_input: String,
}

impl JoinPage {
    pub fn new() -> Self {
        Self {
            game_state: None,
            url: String::new(),
            username: String::new(),
            is_loading: false,
            client: None,
            chat_input: String::new(),
        }
    }

    fn leave_game(&mut self) {
        if let Some(client) = &self.client {
            let client = client.clone();
            tokio::spawn(async move {
                client.disconnect();
            });
        }

        self.game_state = None;
        self.client = None;
    }
}

impl App {
    pub fn render_join_page(&mut self, ctx: &Context, frame: &mut Frame) {
        // check if server shut down
        {
            let mut shutdown = false;
            if let Some(state) = &self.join_page.game_state {
                if state.server_shutdown {
                    shutdown = true;
                }
            }
            if shutdown {
                self.join_page.leave_game();
            }
        }

        match self.join_page.client.is_some() && self.join_page.game_state.is_some() {
            false => {
                // join game phase
                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::Grid::new("join_input").num_columns(2).show(ui, |ui| {
                        // input URL
                        ui.label("URL: ");
                        let input = egui::TextEdit::singleline(&mut self.join_page.url)
                            .hint_text("ws://127.0.0.1:8144");
                        ui.add(input);
                        ui.end_row();

                        // input username
                        ui.label("Username: ");
                        ui.text_edit_singleline(&mut self.join_page.username);
                        ui.end_row();
                    });

                    ui.separator();

                    // join button
                    let enabled = !self.join_page.username.is_empty();
                    ui.add_enabled_ui(enabled, |ui| {
                        if ui.button("Join Game").clicked() {
                            self.join_game();
                        }
                    });
                });
            }
            true => {
                // client has joined a game
                self.render_game_page(ctx, frame);
            }
        }
    }

    /// Tries to join a lobby.
    fn join_game(&mut self) {
        // disconnect client if it already exists
        if let Some(client) = &self.join_page.client {
            client.disconnect();
        }

        let url = match self.join_page.url.is_empty() {
            true => "ws://127.0.0.1:8144".to_string(),
            false => self.join_page.url.to_owned(),
        };
        let username = match self.join_page.username.is_empty() {
            true => return,
            false => self.join_page.username.to_owned(),
        };

        self.join_page.is_loading = true;

        let message = GuiMessage::JoinGame { url, username };
        self.handle_message(message);
    }
}
