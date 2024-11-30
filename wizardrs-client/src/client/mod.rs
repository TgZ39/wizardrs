use crate::error::*;
use crate::state::player::Player;
use crate::state::GameState;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, watch, RwLock};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tracing::debug;
use uuid::Uuid;
use wizardrs_core::client_event::ClientEvent;
use wizardrs_core::server_event::ServerEvent;

pub struct WizardClient {
    pub username: String,
    pub uuid: Uuid,
    event_tx: mpsc::UnboundedSender<ClientEvent>,
    leave_tx: watch::Sender<bool>,
    state_tx: std::sync::mpsc::Sender<GameState>,
    game_state: Arc<RwLock<GameState>>,
}

impl WizardClient {
    pub async fn new(
        url: impl Into<String>,
        username: impl Into<String>,
        state_tx: std::sync::mpsc::Sender<GameState>,
    ) -> Result<Arc<Self>> {
        let (ws_stream, _) = connect_async(url.into()).await?;
        let (mut write, mut read) = ws_stream.split();

        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let username = username.into();
        let mut client = Self {
            username: username.clone(),
            uuid: Default::default(),
            event_tx,
            leave_tx: Default::default(),
            state_tx,
            game_state: Arc::new(RwLock::new(GameState::new())),
        };

        // receive uuid
        'outer: loop {
            while let Some(Ok(msg)) = read.next().await {
                if let Ok(ServerEvent::SetUUID { uuid }) =
                    serde_json::from_str::<ServerEvent>(&msg.to_string())
                {
                    client.uuid = uuid;
                    break 'outer;
                }
            }

            return Err(Error::ConnectionClosed);
        }

        // send username
        let username_event = ClientEvent::SetUsername { username };
        let json = serde_json::to_string(&username_event).unwrap();
        let msg = Message::text(json);

        if write.send(msg).await.is_err() {
            return Err(Error::ConnectionClosed);
        }

        let client = Arc::new(client);

        client.spawn_event_sender(write, event_rx);
        client.spawn_event_receiver(read);

        // send first game state
        client.update_game_state().await;

        Ok(client)
    }

    /// Spawn task to send events to the server
    fn spawn_event_sender(
        self: &Arc<Self>,
        mut write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
        mut event_rx: mpsc::UnboundedReceiver<ClientEvent>,
    ) {
        let client = self.clone();
        let mut leave_rx = self.leave_tx.subscribe();

        // send events to server
        tokio::spawn(async move {
            debug!("starting event sender task: {}", client.uuid);

            let c = client.clone();
            let send_fut = async move {
                while let Some(event) = event_rx.recv().await {
                    let json = serde_json::to_string(&event).unwrap();
                    let msg = Message::text(json);

                    if write.send(msg).await.is_err() {
                        c.disconnect();
                    }
                }
            };

            tokio::select! {
                _ = leave_rx.changed() => {}
                _ = send_fut => {}
            }

            debug!("stopping event sender task: {}", client.uuid);
            client.shutdown().await;
            client.disconnect();
        });
    }

    fn spawn_event_receiver(
        self: &Arc<Self>,
        mut read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    ) {
        let client = self.clone();
        let mut leave_rx = self.leave_tx.subscribe();

        // receive events from server
        tokio::spawn(async move {
            debug!("starting event receiver task: {}", client.uuid);

            let c = client.clone();
            let recv_fut = async move {
                while let Some(Ok(msg)) = read.next().await {
                    if let Ok(event) = serde_json::from_str(&msg.to_string()) {
                        c.handle_server_event(event).await;
                    }
                }
            };

            tokio::select! {
                _ = leave_rx.changed() => {}
                _ = recv_fut => {}
            }

            debug!("stopping event receiver task: {}", client.uuid);
            client.shutdown().await;
            client.disconnect();
        });
    }

    /// Handle events being sent from the server to the client
    async fn handle_server_event(self: &Arc<Self>, event: ServerEvent) {
        match event {
            ServerEvent::UpdatePlayerList { players } => {
                let players = players
                    .into_iter()
                    .map(|(username, uuid)| Player {
                        username,
                        uuid,
                        bid: None,
                    })
                    .collect::<Vec<_>>();

                self.game_state.write().await.set_players(players);
                self.update_game_state().await;
            }
            ServerEvent::SetUUID { .. } => {}
            ServerEvent::PlayerChatMessage { .. } => {
                self.game_state.write().await.push_event_log(event);
                self.update_game_state().await;
            }
        }
    }

    async fn shutdown(self: &Arc<Self>) {
        self.game_state.write().await.server_shutdown = true;
        self.update_game_state().await;
    }

    pub fn disconnect(self: &Arc<Self>) {
        // tell tasks to stop
        self.leave_tx.send_replace(true);
    }

    /// Sends a ClientEvent to the server
    pub fn send_event(self: &Arc<Self>, event: ClientEvent) {
        let _ = self.event_tx.send(event);
    }

    /// Send the GameState to the GUI
    async fn update_game_state(self: &Arc<Self>) {
        let state = (*self.game_state.read().await).clone();
        let _ = self.state_tx.send(state);
    }
}

impl Drop for WizardClient {
    fn drop(&mut self) {
        // tell tasks to shut down
        self.leave_tx.send_replace(true);
    }
}
