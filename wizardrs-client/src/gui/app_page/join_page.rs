use crate::client::WizardClient;
use crate::gui::App;
use crate::state::GameState;
use eframe::Frame;
use egui::Context;
use std::sync::{mpsc, Arc};

pub(crate) mod game_page;

pub struct JoinPage {
    game_state: Option<GameState>,
    state_rx: Option<mpsc::Receiver<GameState>>,
    url: String,
    username: String,
    is_loading: bool,
    get_client_rx: Option<mpsc::Receiver<Option<Arc<WizardClient>>>>,
    client: Option<Arc<WizardClient>>,
    chat_input: String,
}

impl JoinPage {
    pub fn new() -> Self {
        Self {
            game_state: None,
            state_rx: None,
            url: String::new(),
            username: String::new(),
            is_loading: false,
            get_client_rx: None,
            client: None,
            chat_input: String::new(),
        }
    }

    fn update_game_state(&mut self) {
        if let Some(state_rx) = &self.state_rx {
            while let Ok(state) = state_rx.try_recv() {
                self.game_state = Some(state);
            }
        }
    }

    fn get_url(&self) -> String {
        match self.url.is_empty() {
            true => String::from("ws://127.0.0.1:8144"),
            false => self.url.clone(),
        }
    }

    fn get_username(&self) -> Option<String> {
        if self.username.is_empty() {
            None
        } else {
            Some(self.username.clone())
        }
    }

    fn get_client(&mut self) {
        if self.is_loading && self.get_client_rx.is_some() {
            if let Some(recv) = &self.get_client_rx {
                while let Ok(Some(client)) = recv.try_recv() {
                    self.client = Some(client);
                    self.is_loading = false;
                }
            }
        }
    }

    fn join_game(&mut self) {
        if let Some(client) = &self.client {
            client.disconnect();
        }

        let url = self.get_url();
        let username = if let Some(username) = self.get_username() {
            username
        } else {
            return;
        };

        let (send, recv) = mpsc::channel::<Option<Arc<WizardClient>>>();
        let (state_tx, state_rx) = mpsc::channel();

        self.is_loading = true;
        self.get_client_rx = Some(recv);
        self.state_rx = Some(state_rx);

        // create client
        tokio::spawn(async move {
            let client = WizardClient::new(url, username, state_tx).await.ok();
            send.send(client).unwrap();
        });
    }

    fn leave_game(&mut self) {
        if let Some(client) = &self.client {
            let client = client.clone();
            tokio::spawn(async move {
                client.disconnect();
            });
        }

        self.game_state = None;
        self.get_client_rx = None;
        self.state_rx = None;
        self.client = None;
    }
}

impl App {
    pub fn render_join_page(&mut self, ctx: &Context, frame: &mut Frame) {
        self.join_page.get_client();
        self.join_page.update_game_state();

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
                    let enabled = self.join_page.get_username().is_some();
                    ui.add_enabled_ui(enabled, |ui| {
                        if ui.button("Join Game").clicked() {
                            self.join_page.join_game();
                        }
                    });
                });
            }
            true => {
                self.join_page.render_game_page(ctx, frame);
            }
        }
    }
}
