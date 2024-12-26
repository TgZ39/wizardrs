use crate::gui::App;
use eframe::Frame;
use egui::{global_theme_preference_buttons, Context};

pub struct SettingsPage;

impl App {
    pub fn render_settings_page(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Dark/Light mode: ");
                global_theme_preference_buttons(ui);
            });
        });
    }
}
