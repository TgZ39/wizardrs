use crate::gui::App;
use arboard::Clipboard;
use eframe::Frame;
use egui::{Color32, Context};
use std::sync::{mpsc, Arc};
use wizardrs_server::server::WizardServer;

pub struct HostPage {
    pub port: String,
    pub with_ngrok: bool,
    show_authtoken: bool,
    pub authtoken: String,
    pub server: Option<Arc<WizardServer>>,
    is_loading: bool,
    recv_server_rx: Option<mpsc::Receiver<Option<Arc<WizardServer>>>>,
}

impl HostPage {
    pub fn new() -> Self {
        Self {
            port: String::new(),
            with_ngrok: false,
            show_authtoken: false,
            authtoken: String::new(),
            server: None,
            is_loading: false,    // indicate whether a server is being started
            recv_server_rx: None, // receive the server from the task
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

        // match (
        //     self.get_port().is_some(),
        //     self.with_ngrok,
        //     self.get_authtoken().is_some(),
        // ) {
        //     (true, true, true) | (true, false, _) => true,
        //     _ => false,
        // }
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

    fn get_server(&mut self) {
        if self.is_loading && self.recv_server_rx.is_some() {
            let mut clear_rx = false;
            if let Some(recv) = &self.recv_server_rx {
                while let Ok(server) = recv.try_recv() {
                    self.server = server;
                    self.is_loading = false;

                    clear_rx = true;
                }
            }

            if clear_rx {
                self.recv_server_rx = None;
            }
        }
    }
}

impl App {
    pub fn render_host_page(&mut self, ctx: &Context, frame: &mut Frame) {
        self.host_page.get_server();

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
                            self.start_server();
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

            if let Some(server) = &self.host_page.server {
                egui::Grid::new("url_grid").num_columns(2).show(ui, |ui| {
                    let mut clipboard = Clipboard::new().unwrap();

                    // local URL
                    ui.label("local URL:");
                    if ui.link(server.local_url.to_string()).clicked() {
                        clipboard.set_text(server.local_url.to_string()).unwrap();
                    };
                    ui.end_row();

                    // ngrok url
                    if let Some(url) = &server.ngrok_url {
                        ui.label("ngrok URL:");
                        if ui.link(url.to_string()).clicked() {
                            clipboard.set_text(url.to_string()).unwrap();
                        }
                    }
                });
            }
        });
    }

    fn start_server(&mut self) {
        let port = self.host_page.get_port().unwrap();
        let authtoken = self.host_page.get_authtoken();
        let (send, recv) = mpsc::channel::<Option<Arc<WizardServer>>>();

        self.host_page.is_loading = true;
        self.host_page.recv_server_rx = Some(recv);

        // create server
        tokio::spawn(async move {
            let server = WizardServer::new(port, authtoken).await.ok();
            send.send(server).unwrap();
        });
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
