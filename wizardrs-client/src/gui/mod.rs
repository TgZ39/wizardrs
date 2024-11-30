use crate::gui::app_page::host_page::HostPage;
use crate::gui::app_page::join_page::JoinPage;
use crate::gui::app_page::AppPage;
use eframe::Frame;
use egui::Context;
use strum::IntoEnumIterator;

pub(crate) mod app_page;

pub struct App {
    current_page: AppPage,
    host_page: HostPage,
    join_page: JoinPage,
    first_frame: bool, // set ppp on startup
}

impl App {
    pub fn new() -> Self {
        Self {
            current_page: AppPage::Host,
            host_page: HostPage::new(),
            join_page: JoinPage::new(),
            first_frame: true,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        if self.first_frame {
            ctx.set_zoom_factor(1.5);
            self.first_frame = false;
        }

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
        }

        ctx.request_repaint();
    }
}
