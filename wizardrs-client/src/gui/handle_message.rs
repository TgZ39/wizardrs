use crate::gui::App;
use crate::{
    client::WizardClient,
    interaction::{GuiMessage, StateUpdate},
};
use std::sync::mpsc;
use wizardrs_core::client_event::ClientEvent;
use wizardrs_server::server::WizardServer;

impl App {
    pub fn handle_message(&self, message: GuiMessage) {
        let state_tx = self.state_tx.clone();

        let client = self.join_page.client.clone();
        tokio::spawn(async move {
            match message {
                GuiMessage::CreateServer { port, authtoken } => {
                    let server = WizardServer::new(port, authtoken).await.ok();
                    let update = StateUpdate::WizardServer(server);

                    state_tx
                        .send(update)
                        .expect("error sending WizardServer to GUI");
                }
                GuiMessage::JoinGame { url, username } => {
                    let (local_state_tx, local_state_rx) = mpsc::channel();

                    // forward state updates from WizardClient to GUI
                    {
                        let state_tx = state_tx.clone();

                        tokio::spawn(async move {
                            while let Ok(state) = local_state_rx.recv() {
                                let update = StateUpdate::GameState(Some(state));
                                state_tx
                                    .send(update)
                                    .expect("error sending GameState to GUI");
                            }
                        });
                    }

                    // create client
                    let client = WizardClient::new(url, username, local_state_tx).await;
                    let update = StateUpdate::WizardClient(client.ok());
                    state_tx
                        .send(update)
                        .expect("error sending WizardClient to GUI");
                }
                GuiMessage::PlayCard { card } => {
                    if let Some(client) = client {
                        let event = ClientEvent::PlayCard { card };
                        client.send_event(event);
                    }
                }
                GuiMessage::Ready => {
                    if let Some(client) = client {
                        let event = ClientEvent::Ready;
                        client.send_event(event);
                    }
                }
            }
        });
    }
}
