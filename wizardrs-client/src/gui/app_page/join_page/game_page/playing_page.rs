use crate::gui::App;
use eframe::Frame;
use egui::Context;
use std::ops::Deref;

impl App {
    pub fn render_playing_page(&mut self, ctx: &Context, frame: &mut Frame) {
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

        // bid selection
        egui::TopBottomPanel::bottom("bid_selection_panel")
            .frame(egui::Frame::side_top_panel(ctx.style().deref()))
            .show(ctx, |ui| {
                ui.heading("Bid selection");
                ui.separator();

                self.render_bid_selection(ui, ctx, frame);
            });

        // hand
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hand");
            ui.separator();

            self.render_hand(ui, ctx, frame);
        });
    }
}
