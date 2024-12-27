use std::sync::mpsc;

use crate::gui::app_page::host_page::HostPage;
use crate::gui::app_page::join_page::JoinPage;
use crate::gui::app_page::settings_page::SettingsPage;
use crate::gui::app_page::AppPage;
use crate::interaction::StateUpdate;
use eframe::Frame;
use egui::Context;
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
}

impl App {
    pub fn new() -> Self {
        let (state_tx, state_rx) = mpsc::channel();

        Self {
            current_page: AppPage::Host,
            host_page: HostPage::new(),
            join_page: JoinPage::new(),
            settings_page: SettingsPage {},
            state_rx,
            state_tx,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        self.update_state();

        // Top Panel
        egui::TopBottomPanel::top("top_panel")
            .frame(egui::Frame::none().inner_margin(4.0))
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
}
