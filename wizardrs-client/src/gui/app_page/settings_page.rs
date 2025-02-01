use crate::gui::App;
use crate::interaction::Message;
use eframe::emath::Align;
use eframe::Frame;
use egui::{Context, ProgressBar};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

pub struct SettingsPage {
    pub deck_paths: Vec<PathBuf>,
    pub downloading_adrian_kennard: bool,
    pub download_progress: Option<Arc<AtomicU8>>,
}

impl SettingsPage {
    pub fn new() -> Self {
        Self {
            deck_paths: vec![],
            downloading_adrian_kennard: false,
            download_progress: None,
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
                        // refresh active deck
                        if let Some(path) = &self.config.card_deck {
                            let message = Message::RequestImageCache {
                                path: path.to_path_buf(),
                            };
                            self.handle_message(message);
                        }

                        // update deck list
                        let message = Message::RequestUpdateDeckList;
                        self.handle_message(message);
                    }

                    // download Adrian Kennard deck button
                    if !self.settings_page.downloading_adrian_kennard {
                        if ui.button("Download adrian-kennard deck").clicked() {
                            self.settings_page.downloading_adrian_kennard = true;

                            let message = Message::DownloadAndrianKennardDeck;
                            self.handle_message(message);
                        }
                    } else {
                        if let Some(progress) = &self.settings_page.download_progress {
                            let percent = progress.load(Ordering::Relaxed) as f32 / 60.0;

                            ui.add(ProgressBar::new(percent).desired_width(232.0));
                        } else {
                            ui.add_enabled_ui(false, |ui| {
                                let _ = ui.button("Download adrian-kennard deck");
                            });
                        }
                    }

                    // import zip button
                    if ui.button("Import").clicked() {
                        let message = Message::ImportDeck;
                        self.handle_message(message);
                    }
                });
            });

            // github hyperlink
            ui.with_layout(egui::Layout::bottom_up(Align::Center), |ui| {
                ui.hyperlink_to(
                    "wizardrs on GitHub",
                    "https://www.github.com/TgZ39/wizardrs",
                )
            });
        });
    }
}
