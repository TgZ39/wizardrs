use std::ops::Deref;

use crate::gui::App;
use eframe::Frame;
use egui::{Color32, Context, Ui, Vec2};
use wizardrs_core::card::color::CardColor;
use wizardrs_core::{client_event::ClientEvent, game_phase::GamePhase};

impl App {
    pub fn render_bidding_page(&mut self, ctx: &Context, frame: &mut Frame) {
        // scoreboard
        egui::SidePanel::right("scoreboard")
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Scoreboard");
                });
                ui.separator();

                self.render_scoreboard(ui, ctx, frame);
            });

        // current trick
        egui::TopBottomPanel::top("current_trick")
            .frame(egui::Frame::side_top_panel(ctx.style().deref()))
            .show(ctx, |ui| {
                self.render_top_bar(ui, ctx, frame);
            });

        if let Some(state) = &self.join_page.game_state {
            if state.self_select_trump_color {
                // color selection
                egui::TopBottomPanel::bottom("trump_color_selection")
                    .frame(egui::Frame::side_top_panel(ctx.style().deref()))
                    .show(ctx, |ui| {
                        ui.heading("Trump Color Selection");
                        ui.separator();

                        self.render_trump_color_selection(ui, ctx, frame);
                    });
            } else {
                // bid selection
                egui::TopBottomPanel::bottom("bid_selection_panel")
                    .frame(egui::Frame::side_top_panel(ctx.style().deref()))
                    .show(ctx, |ui| {
                        ui.heading("Bid selection");
                        ui.separator();

                        self.render_bid_selection(ui, ctx, frame);
                    });
            }
        }

        // hand
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hand");
            ui.separator();

            self.render_hand(ui, ctx, frame);
        });
    }

    pub fn render_bid_selection(&mut self, ui: &mut Ui, _ctx: &Context, _frame: &mut Frame) {
        ui.add_space(3.0);

        let widget_width = 50.0 + 5.0;
        let num_columns = (ui.available_width() / widget_width).floor() as usize;

        egui::ScrollArea::vertical()
            .enable_scrolling(true)
            .show(ui, |ui| {
                egui::Grid::new("bid_selection").show(ui, |ui| {
                    // get bids
                    if let Some(state) = &self.join_page.game_state {
                        let possible_bids: Vec<i32> =
                            (0..=state.scoreboard.current_round as i32).collect();

                        let disallowed_bid = {
                            let mut bid = i32::MAX;

                            if let Some(client) = &self.join_page.client {
                                // check if self is last player to bid
                                if state.is_last_to_bid(client.uuid).is_some_and(|b| b) {
                                    // find disallowed bid
                                    let sum_bids = state.scoreboard.bid_sum();
                                    let current_round = state.scoreboard.current_round;
                                    bid = current_round as i32 - sum_bids as i32;
                                }
                            }

                            bid
                        };

                        if let Some(client) = &self.join_page.client {
                            // checks whether a bid can be chosen
                            let check_enabled = |bid: i32| -> bool {
                                state.game_phase == GamePhase::Bidding // check if it is bidding phase
                                        && state.get_player_on_turn().uuid == client.uuid // check if self is player on turn
                                        && possible_bids.contains(&bid) // check if bid has valid range
                                        && bid != disallowed_bid // check if bid is not disallowed bid
                            };

                            let mut current_column = 0;

                            // render bids
                            for bid in &possible_bids {
                                current_column += 1;

                                let button = egui::Button::new(bid.to_string())
                                    .min_size(Vec2::new(50.0, 50.0));
                                let resp = ui.add_enabled(check_enabled(*bid), button);

                                if resp.clicked() {
                                    // TODO transfer to GuiMessage
                                    // bid clicked
                                    let event = ClientEvent::MakeBid { bid: *bid as u8 };
                                    client.send_event(event);
                                }

                                if current_column >= num_columns {
                                    ui.end_row();
                                    current_column = 0;
                                }
                            }
                        }
                    }
                });
            });

        ui.add_space(4.0);
    }

    fn render_trump_color_selection(&mut self, ui: &mut Ui, _ctx: &Context, _frame: &mut Frame) {
        ui.add_space(3.0);

        egui::Grid::new("trump_color_selection").show(ui, |ui| {
            // blue
            let button = egui::Button::new("Blue")
                .min_size(egui::Vec2::new(50.0, 50.0))
                .fill(Color32::BLUE);
            if ui.add(button).clicked() {
                // TODO transfer to GuiMessage
                let event = ClientEvent::SetTrumpColor {
                    color: CardColor::Blue,
                };
                if let Some(client) = &self.join_page.client {
                    client.send_event(event);
                }
            }

            // green
            let button = egui::Button::new("Green")
                .min_size(egui::Vec2::new(50.0, 50.0))
                .fill(Color32::GREEN);
            if ui.add(button).clicked() {
                // TODO transfer to GuiMessage
                let event = ClientEvent::SetTrumpColor {
                    color: CardColor::Green,
                };
                if let Some(client) = &self.join_page.client {
                    client.send_event(event);
                }
            }

            // red
            let button = egui::Button::new("Red")
                .min_size(egui::Vec2::new(50.0, 50.0))
                .fill(Color32::RED);
            if ui.add(button).clicked() {
                // TODO transfer to GuiMessage
                let event = ClientEvent::SetTrumpColor {
                    color: CardColor::Red,
                };
                if let Some(client) = &self.join_page.client {
                    client.send_event(event);
                }
            }

            // yellow
            let button = egui::Button::new("Yellow")
                .min_size(egui::Vec2::new(50.0, 50.0))
                .fill(Color32::YELLOW);
            if ui.add(button).clicked() {
                // TODO transfer to GuiMessage
                let event = ClientEvent::SetTrumpColor {
                    color: CardColor::Yellow,
                };
                if let Some(client) = &self.join_page.client {
                    client.send_event(event);
                }
            }
        });

        ui.add_space(4.0);
    }
}
