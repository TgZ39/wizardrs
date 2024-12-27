use crate::config::Config;
use crate::gui::app_page::host_page::HostPage;
use crate::gui::app_page::join_page::JoinPage;
use crate::gui::app_page::settings_page::SettingsPage;
use crate::gui::app_page::AppPage;
use crate::image_cache::ImageCache;
use crate::interaction::{GuiMessage, StateUpdate};
use eframe::Frame;
use egui::Context;
use std::sync::mpsc;
use strum::IntoEnumIterator;

pub(crate) mod app_page;
pub(crate) mod handle_message;
pub(crate) mod handle_state_update;

pub struct App {
    current_page: AppPage,
    host_page: HostPage,
    join_page: JoinPage,
    settings_page: SettingsPage,
    state_rx: mpsc::Receiver<StateUpdate>, // receive state updates from backend
    state_tx: mpsc::Sender<StateUpdate>,   // used to pass to tasks to send state updates to self
    config: Config,
    image_cache: Option<ImageCache>,
}

impl App {
    pub fn new(config: Config) -> Self {
        let (state_tx, state_rx) = mpsc::channel();

        let app = Self {
            current_page: AppPage::Host,
            host_page: HostPage::new(),
            join_page: JoinPage::new(),
            settings_page: SettingsPage::new(),
            state_rx,
            state_tx,
            config,
            image_cache: None,
        };

        // async load image cache
        if let Some(path) = &app.config.card_deck {
            let message = GuiMessage::RequestImageCache {
                path: path.to_path_buf(),
            };
            app.handle_message(message);
        }

        app
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        self.update_state();

        let bg_frame = egui::containers::Frame::default()
            .inner_margin(4.0)
            .fill(ctx.style().visuals.extreme_bg_color);
        egui::TopBottomPanel::top("top_panel")
            .frame(bg_frame)
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.visuals_mut().button_frame = false;

                    for page in AppPage::iter() {
                        if ui
                            .selectable_label(page == self.current_page, page.to_string())
                            .clicked()
                        {
                            self.current_page = page;
                        }
                    }
                });
            });

        // show AppPage
        match self.current_page {
            AppPage::Host => self.render_host_page(ctx, frame),
            AppPage::Join => self.render_join_page(ctx, frame),
            AppPage::Settings => self.render_settings_page(ctx, frame),
        }

        ctx.request_repaint();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.config.save().unwrap();
    }
}
