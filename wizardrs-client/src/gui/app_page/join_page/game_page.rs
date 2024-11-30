use crate::gui::app_page::join_page::JoinPage;
use crate::state::game_phase::GamePhase;
use eframe::Frame;
use egui::Context;
use std::ops::Deref;
use wizardrs_core::client_event::ClientEvent;
use wizardrs_core::server_event::ServerEvent;

pub(crate) mod bidding_page;
pub(crate) mod finished_page;
pub(crate) mod lobby_page;
pub(crate) mod playing_page;

impl JoinPage {
    pub fn render_game_page(&mut self, ctx: &Context, frame: &mut Frame) {
        self.side_panel(ctx, frame);

        if let Some(state) = &self.game_state {
            match state.phase {
                GamePhase::Lobby => self.render_lobby_page(ctx, frame),
                GamePhase::Bidding => {}
                GamePhase::Playing => {}
                GamePhase::Finished => {}
            }
        }
    }

    pub fn side_panel(&mut self, ctx: &Context, frame: &mut Frame) {
        // player list
        egui::SidePanel::left("players")
            .resizable(true)
            .default_width(200.0)
            .width_range(150.0..=250.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Players");
                });

                if let Some(state) = &self.game_state {
                    for player in &state.players {
                        ui.label(&player.username);
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
                        if let Some(state) = &self.game_state {
                            for event in &state.event_log {
                                let text = match event {
                                    ServerEvent::UpdatePlayerList { .. } => continue,
                                    ServerEvent::SetUUID { .. } => continue,
                                    ServerEvent::PlayerChatMessage {
                                        username, content, ..
                                    } => format!("[{}] {}", username, content),
                                };

                                ui.label(text);
                            }
                        }

                        ui.add_space(ui.available_height() - 25.0);

                        // chat message input
                        resp = Some(ui.text_edit_singleline(&mut self.chat_input));
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
        let msg = self.chat_input.clone();
        if msg.is_empty() {
            return;
        }

        if let Some(client) = &self.client {
            let event = ClientEvent::SendChatMessage { content: msg };
            client.send_event(event);
        }

        self.chat_input.clear();
    }
}
