use crate::gui::App;
use crate::interaction::Message;
use directories::ProjectDirs;
use eframe::Frame;
use egui::Context;
use std::fs;
use std::path::PathBuf;

pub struct SettingsPage {
    deck_paths: Vec<PathBuf>,
    pub downloading_adrian_kennard: bool,
}

impl SettingsPage {
    pub fn new() -> Self {
        let mut page = Self {
            deck_paths: vec![],
            downloading_adrian_kennard: false,
        };
        page.update_card_decks();

        page
    }

    pub fn update_card_decks(&mut self) {
        if let Some(proj_dirs) = ProjectDirs::from("de", "TgZ39", "Wizardrs") {
            let mut deck_dir = proj_dirs.data_dir().to_path_buf();
            deck_dir.push("decks");

            let mut out = Vec::new();
            for entry in fs::read_dir(deck_dir).unwrap().flatten() {
                if entry.path().is_dir() {
                    out.push(entry.path().to_path_buf());
                }
            }

            self.deck_paths = out;
        }
    }
}

impl App {
    pub fn render_settings_page(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Grid::new("settings").show(ui, |ui| {
                // theme preference
                ui.label("Theme: ");
                let mut theme_preference = ui.ctx().options(|opt| opt.theme_preference);
                theme_preference.radio_buttons(ui);

                ui.ctx().set_theme(theme_preference);
                self.config.theme = theme_preference;
                ui.end_row();

                // deck selection
                ui.label("Deck: ");
                let selected_name = match &self.config.card_deck {
                    None => "None".to_string(),
                    Some(path) => path.file_name().unwrap().to_string_lossy().to_string(),
                };
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt("deck_selection")
                        .selected_text(selected_name)
                        .show_ui(ui, |ui| {
                            for path in &self.settings_page.deck_paths {
                                let name = path.file_name().unwrap().to_string_lossy().to_string();

                                if ui
                                    .selectable_value(
                                        &mut self.config.card_deck,
                                        Some(path.to_path_buf()),
                                        name,
                                    )
                                    .clicked()
                                {
                                    let message = Message::RequestImageCache {
                                        path: path.to_path_buf(),
                                    };
                                    self.handle_message(message);
                                };
                            }

                            // none option
                            if ui
                                .selectable_value(&mut self.config.card_deck, None, "None")
                                .clicked()
                            {
                                self.image_cache = None;
                            }
                        });
                    // refresh button
                    if ui.button("Refresh").clicked() {
                        // update installed decks
                        self.settings_page.update_card_decks();

                        // refresh active deck
                        if let Some(path) = &self.config.card_deck {
                            let message = Message::RequestImageCache {
                                path: path.to_path_buf(),
                            };
                            self.handle_message(message);
                        }
                    }

                    // download Adrian Kennard Deck
                    ui.add_enabled_ui(!self.settings_page.downloading_adrian_kennard, |ui| {
                        if ui.button("Download adrian-kennard deck").clicked() {
                            self.settings_page.downloading_adrian_kennard = true;

                            let message = Message::DownloadAndrianKennardDeck;
                            self.handle_message(message);
                        }
                    });
                });
            });
        });
    }
}
