use crate::gui::App;
use crate::interaction::GuiMessage;
use eframe::Frame;
use egui::{Context, Image, RichText, Ui, Vec2};
use egui_extras::Column;
use std::ops::Deref;
use tracing::error;
use wizardrs_core::card::value::CardValue;
use wizardrs_core::card::Card;
use wizardrs_core::client_event::ClientEvent;
use wizardrs_core::game_phase::GamePhase;
use wizardrs_core::server_event::ServerEvent;
use wizardrs_core::trump_suit::TrumpSuit;

pub(crate) mod bidding_page;
pub(crate) mod finished_page;
pub(crate) mod lobby_page;
pub(crate) mod playing_page;

impl App {
    pub fn render_game_page(&mut self, ctx: &Context, frame: &mut Frame) {
        self.side_panel(ctx, frame);

        if let Some(state) = &self.join_page.game_state {
            match state.game_phase {
                GamePhase::Lobby => self.render_lobby_page(ctx, frame),
                GamePhase::Bidding => self.render_bidding_page(ctx, frame),
                GamePhase::Playing => self.render_playing_page(ctx, frame),
                GamePhase::Finished => self.render_playing_page(ctx, frame),
            }
        }
    }

    /// Renders the player list and chat
    pub fn side_panel(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::SidePanel::left("players")
            .resizable(true)
            .default_width(250.0)
            .width_range(150.0..=300.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Players");
                });
                ui.separator();

                if let Some(state) = &self.join_page.game_state {
                    for player in &state.players {
                        let label = {
                            let mut out = String::new();
                            // username
                            out.push_str(&player.username);

                            // won tricks vs bid tricks
                            if let Some((_score, bid, won_tricks)) =
                                state.scoreboard.get_entry(player.uuid)
                            {
                                if let Some(bid) = bid {
                                    out.push_str(&format!(" [{}/{}]", won_tricks, bid));
                                }
                            }

                            // ready or not ready
                            if state.waiting_for_ready {
                                match player.is_ready {
                                    true => out.push_str(" [Ready]"),
                                    false => out.push_str(" [Not Ready]"),
                                }
                            }

                            out
                        };
                        if state.get_player_on_turn().uuid == player.uuid // check if player is on turn
                            && !state.waiting_for_ready // check if we are waiting for ready
                            && state.game_phase != GamePhase::Lobby
                            && state.game_phase != GamePhase::Finished
                        {
                            let label = egui::Label::new(RichText::new(label).strong().underline());
                            ui.add(label);
                        } else {
                            let label = egui::Label::new(RichText::new(label));
                            ui.add(label);
                        }
                    }
                }

                let sep = egui::Separator::default().grow(10.0).spacing(0.0);
                ui.add(sep);

                // event log
                let mut resp = None;
                let margin = egui::Margin {
                    left: 2.0,
                    right: 2.0,
                    ..Default::default()
                };
                egui::CentralPanel::default()
                    .frame(egui::Frame::central_panel(ctx.style().deref()).inner_margin(margin))
                    .show_inside(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(ui.available_height() - 30.0)
                            .enable_scrolling(true)
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                if let Some(state) = &self.join_page.game_state {
                                    for event in &state.event_log {
                                        let text = match event {
                                            ServerEvent::PlayerChatMessage {
                                                username,
                                                content,
                                                ..
                                            } => format!("[{}] {}", username, content),
                                            _ => continue,
                                        };

                                        ui.label(text);
                                    }
                                }
                            });

                        ui.add_space(ui.available_height() - 25.0);

                        // chat message input
                        resp = Some(ui.text_edit_singleline(&mut self.join_page.chat_input));
                    });

                // send chat message
                if let Some(resp) = resp {
                    if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.send_chat_message();
                    }
                }
            });
    }

    fn send_chat_message(&mut self) {
        let msg = self.join_page.chat_input.clone();
        if msg.is_empty() {
            return;
        }

        if let Some(client) = &self.join_page.client {
            let event = ClientEvent::SendChatMessage { content: msg };
            client.send_event(event);
        }

        self.join_page.chat_input.clear();
    }

    /// Render Scoreboard
    pub fn render_scoreboard(&mut self, ui: &mut Ui, _ctx: &Context, _frame: &mut Frame) {
        let num_columns = if let Some(state) = &self.join_page.game_state {
            state.players.len() + 1
        } else {
            0
        };

        let table = egui_extras::TableBuilder::new(ui)
            .columns(Column::auto().resizable(false), num_columns)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center));

        table
            .header(15.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Round");
                });
                if let Some(state) = &self.join_page.game_state {
                    for player in &state.players {
                        header.col(|ui| {
                            ui.strong(&player.username);
                        });
                    }
                }
            })
            .body(|mut body| {
                if let Some(state) = &self.join_page.game_state {
                    for (index, round) in state.scoreboard.rounds.iter().enumerate() {
                        body.row(15.0, |mut row| {
                            row.col(|ui| {
                                let round = format!("Round {}", index + 1);
                                ui.strong(round);
                            });

                            for (score, bid, won_tricks) in round {
                                row.col(|ui| {
                                    // check if this round has been player or is being played
                                    if index < state.scoreboard.current_round as usize {
                                        let score = match score {
                                            Some(score) => format!("{: >3}", score),
                                            None => "   ".to_string(),
                                        };
                                        let bid = match bid {
                                            Some(bid) => format!("{}/{}", won_tricks, bid),
                                            None => "   ".to_string(),
                                        };

                                        let label = format!("[{score} {bid}]");
                                        ui.monospace(label);
                                    }
                                });
                            }
                        });
                    }
                }
            });

        // Ready button
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            if let (Some(state), Some(client)) =
                (&self.join_page.game_state, &self.join_page.client)
            {
                let button_enabled = {
                    if let Some(self_player) = state.players.iter().find(|p| p.uuid == client.uuid)
                    {
                        state.waiting_for_ready && !self_player.is_ready
                    } else {
                        false
                    }
                };

                ui.add_enabled_ui(button_enabled, |ui| {
                    let button =
                        egui::Button::new("Ready").min_size(Vec2::new(ui.available_size().x, 95.0));

                    let resp = ui.add(button);
                    if resp.clicked() {
                        // ready button clicked
                        let message = GuiMessage::Ready;
                        self.handle_message(message);
                    }
                });
            }
        });
    }

    /// Renders the trump suit and the current trick.
    pub fn render_top_bar(&self, ui: &mut Ui, _ctx: &Context, _frame: &mut Frame) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("Trump Suit");
                ui.add_sized(
                    Vec2::new(120.0, 2.0),
                    egui::Separator::default().horizontal(),
                );

                if let Some(state) = &self.join_page.game_state {
                    match &state.trump_suit {
                        TrumpSuit::Card(card) => {
                            let image = Image::new(card.image())
                                .rounding(10.0)
                                .max_size(Vec2::new(120.0, 120.0 * 1.57)) // image aspect ratio is ~ 1:1.57
                                .fit_to_exact_size(Vec2::new(120.0, 120.0 * 1.57));
                            ui.add_sized(Vec2::new(120.0, 120.0 * 1.57), image);
                            ui.label(card.color.to_string());
                        }
                        TrumpSuit::Color(card, color) => {
                            let image = Image::new(card.image())
                                .rounding(10.0)
                                .max_size(Vec2::new(120.0, 120.0 * 1.57)) // image aspect ratio is ~ 1:1.57
                                .fit_to_exact_size(Vec2::new(120.0, 120.0 * 1.57));
                            ui.add_sized(Vec2::new(120.0, 120.0 * 1.57), image);

                            let color = match (color, card.value) {
                                (Some(color), CardValue::Wizard) => color.to_string(),
                                (None, CardValue::Wizard) => "No color yet".to_string(),
                                (None, CardValue::Fool) => "No color".to_string(),
                                _ => {
                                    error!("invalid trump suit: {:?}", state.trump_suit);
                                    "invalid trump suit".to_string()
                                }
                            };
                            ui.label(color);
                        }
                        TrumpSuit::None => {
                            let label = egui::Label::new("No Trump Suit");
                            ui.add_sized(Vec2::new(120.0, 120.0 * 1.57), label);
                        }
                    }
                }
            });

            ui.separator();

            // current trick
            ui.vertical(|ui| {
                ui.heading("Current Trick");
                ui.separator();

                egui::Grid::new("current_trick").show(ui, |ui| {
                    if let Some(state) = &self.join_page.game_state {
                        for (uuid, card) in &state.played_cards {
                            let player = state
                                .players
                                .iter()
                                .find(|player| player.uuid == *uuid)
                                .unwrap();

                            ui.vertical(|ui| {
                                let image = Image::new(card.image())
                                    .rounding(10.0)
                                    .max_size(Vec2::new(120.0, 120.0 * 1.57)) // image aspect ratio is ~ 1:1.57
                                    .fit_to_exact_size(Vec2::new(120.0, 120.0 * 1.57));
                                ui.add_sized(Vec2::new(120.0, 120.0 * 1.57), image);

                                ui.label(&player.username);
                            });
                        }
                    }
                });
            });
        });

        ui.add_space(5.0);
    }
}

impl App {
    /// Render own cards
    pub fn render_hand(&mut self, ui: &mut Ui, _ctx: &Context, _frame: &mut Frame) {
        let widget_width = 120.0 + 15.0;
        let num_columns = (ui.available_width() / widget_width).floor() as usize;

        egui::ScrollArea::vertical()
            .enable_scrolling(true)
            .show(ui, |ui| {
                egui::Grid::new("hand").show(ui, |ui| {
                    // check if self can play card
                    let check_enabled = |card: &Card| -> bool {
                        // check if self is on turn
                        if let (Some(state), Some(client)) =
                            (&self.join_page.game_state, &self.join_page.client)
                        {
                            // check if all cards have already been played
                            // this can happen if we are waiting for everyone ready
                            if state.waiting_for_ready {
                                return false;
                            }

                            // check if all cards have already been played
                            // this can happen if we are waiting for everyone ready
                            if state.played_cards.len() >= state.players.len() {
                                return false;
                            }

                            // check if it is playing phase
                            if state.game_phase != GamePhase::Playing {
                                return false;
                            }

                            if state.get_player_on_turn().uuid != client.uuid {
                                return false;
                            }
                        }

                        // wizard and fool can always be played
                        if card.value == CardValue::Fool || card.value == CardValue::Wizard {
                            return true;
                        }

                        if let Some(state) = &self.join_page.game_state {
                            let leading_color = state.leading_color();
                            return if let Some(leading_color) = leading_color {
                                // there is a leading color which needs to be served

                                // check if self has leading color
                                if state.hand.iter().any(|hand_card| match hand_card.value {
                                    CardValue::Fool => false,
                                    CardValue::Simple(_) => hand_card.color == leading_color,
                                    CardValue::Wizard => false,
                                }) {
                                    // self does have leading color
                                    // only playable if it is a fool/wizard or of leading color
                                    match card.value {
                                        CardValue::Fool | CardValue::Wizard => true,
                                        CardValue::Simple(_) => card.color == leading_color,
                                    }
                                } else {
                                    // self does not have leading color so every card can be played
                                    true
                                }
                            } else {
                                // there is no color which needs to be served so every card can be played
                                true
                            };
                        }

                        // this case should not be reachable
                        error!("unreachable can play card case reached");
                        false
                    };

                    if let Some(state) = &self.join_page.game_state {
                        let mut current_column = 0;

                        for card in &state.hand {
                            current_column += 1;

                            let resp = {
                                let image = Image::new(card.image())
                                    .rounding(10.0)
                                    .max_size(Vec2::new(120.0, 120.0 * 1.57)) // image aspect ratio is ~ 1:1.57
                                    .fit_to_exact_size(Vec2::new(120.0, 120.0 * 1.57));
                                let button = egui::ImageButton::new(image);

                                ui.add_enabled(check_enabled(card), button)
                            };

                            if resp.clicked() {
                                // card clicked
                                let message = GuiMessage::PlayCard { card: *card };
                                self.handle_message(message);
                            }

                            if current_column >= num_columns {
                                ui.end_row();
                                current_column = 0;
                            }
                        }
                    }
                });
            });
    }
}
