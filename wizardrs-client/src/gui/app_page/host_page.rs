use crate::{gui::App, interaction::Message};
use arboard::Clipboard;
use eframe::Frame;
use egui::{Color32, Context};
use egui_extras::Column;
use reqwest::Url;
use std::sync::Arc;
use tracing::error;
use wizardrs_server::server::WizardServer;

pub struct HostPage {
    pub port: String,
    pub with_ngrok: bool,
    show_authtoken: bool,
    pub authtoken: String,
    pub server: Option<Arc<WizardServer>>,
    pub is_loading: bool,
    pub interfaces: Vec<(String, Url)>,
}

impl HostPage {
    pub fn new() -> Self {
        Self {
            port: String::new(),
            with_ngrok: false,
            show_authtoken: false,
            authtoken: String::new(),
            server: None,
            is_loading: false, // indicate whether a server is being started,
            interfaces: vec![],
        }
    }

    fn can_create_game(&self) -> bool {
        matches!(
            (
                self.get_port().is_some(),
                self.with_ngrok,
                self.get_authtoken().is_some(),
            ),
            (true, true, true) | (true, false, _)
        )
    }

    fn get_port(&self) -> Option<u16> {
        if self.port.is_empty() {
            return Some(8144);
        }
        match self.port.parse() {
            Ok(port) => Some(port),
            _ => None,
        }
    }

    fn get_authtoken(&self) -> Option<String> {
        if !self.with_ngrok || self.authtoken.is_empty() {
            None
        } else {
            Some(self.authtoken.clone())
        }
    }
}

impl App {
    pub fn render_host_page(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Grid::new("host_input").num_columns(2).show(ui, |ui| {
                // port input
                ui.label("Port:");
                ui.horizontal(|ui| {
                    let input =
                        egui::TextEdit::singleline(&mut self.host_page.port).hint_text("8144");
                    ui.add(input);

                    // check if port is valid
                    if self.host_page.get_port().is_none() {
                        ui.colored_label(
                            Color32::from_rgb(255, 0, 0),
                            "port must be between 1 and 65.535",
                        );
                    }
                });
                ui.end_row();

                // ngrok input
                ui.checkbox(&mut self.host_page.with_ngrok, "ngrok");

                // authtoken input
                ui.horizontal(|ui| {
                    ui.add_enabled_ui(self.host_page.with_ngrok, |ui| {
                        let input = egui::TextEdit::singleline(&mut self.host_page.authtoken)
                            .password(!self.host_page.show_authtoken);
                        ui.add(input);
                    });

                    // show password button
                    let text = if self.host_page.show_authtoken {
                        "🙈"
                    } else {
                        "👁"
                    };
                    if ui.button(text).clicked() {
                        self.host_page.show_authtoken = !self.host_page.show_authtoken;
                    }

                    // check if authtoken is valid
                    if self.host_page.with_ngrok && self.host_page.get_authtoken().is_none() {
                        ui.colored_label(Color32::from_rgb(255, 0, 0), "invalid authtoken");
                    }
                });
                ui.end_row();
            });

            ui.separator();

            // start server
            ui.horizontal_wrapped(|ui| {
                ui.add_enabled_ui(
                    self.host_page.can_create_game()
                        && !self.host_page.is_loading
                        && self.host_page.server.is_none(),
                    |ui| {
                        if ui.button("Start Server").clicked() {
                            self.create_server();
                        }
                    },
                );

                // stop server
                ui.add_enabled_ui(
                    !self.host_page.is_loading && self.host_page.server.is_some(),
                    |ui| {
                        if ui.button("Stop Server").clicked() {
                            self.stop_server();
                        }
                    },
                );
            });

            if let Some(_server) = &self.host_page.server {
                ui.separator();

                let table = egui_extras::TableBuilder::new(ui)
                    .columns(Column::auto().resizable(false), 2)
                    .striped(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center));

                table
                    .header(15.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("Interface");
                        });

                        header.col(|ui| {
                            ui.strong("URL");
                        });
                    })
                    .body(|mut body| {
                        let mut clipboard = Clipboard::new().unwrap();

                        for (interface, url) in &self.host_page.interfaces {
                            body.row(15.0, |mut row| {
                                // interface
                                row.col(|ui| {
                                    ui.monospace(interface);
                                });
                                // url
                                row.col(|ui| {
                                    if ui.link(url.to_string()).clicked() {
                                        if let Err(err) = clipboard.set_text(url.to_string()) {
                                            error!(?err, "couldn't copy URL to clipboard")
                                        }
                                    }
                                });
                            });
                        }
                    });

                // egui::Grid::new("url_grid").num_columns(2).show(ui, |ui| {
                //     let mut clipboard = Clipboard::new().unwrap();
                //
                //     // local URL
                //     ui.label("local URL:");
                //     if ui.link(server.local_url.to_string()).clicked() {
                //         clipboard.set_text(server.local_url.to_string()).unwrap();
                //     };
                //     ui.end_row();
                //
                //     // ngrok url
                //     if let Some(url) = &server.ngrok_url {
                //         ui.label("ngrok URL:");
                //         if ui.link(url.to_string()).clicked() {
                //             clipboard.set_text(url.to_string()).unwrap();
                //         }
                //     }
                // });
            }
        });
    }

    /// Tries to create a server
    fn create_server(&mut self) {
        let port = self.host_page.port.parse::<u16>().unwrap_or(8144);
        let authtoken = if self.host_page.with_ngrok && !self.host_page.authtoken.is_empty() {
            Some(self.host_page.authtoken.to_owned())
        } else {
            None
        };

        self.host_page.is_loading = true;

        let message = Message::CreateServer { port, authtoken };
        self.handle_message(message);
    }

    fn stop_server(&mut self) {
        if let Some(server) = &self.host_page.server {
            let server = server.clone();
            tokio::spawn(async move {
                server.shutdown().await;
            });

            self.host_page.server = None;
        }
    }
}
