use crate::error::*;
use crate::server::WizardServer;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::{broadcast, mpsc, watch};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{accept_async, WebSocketStream};
use tracing::debug;
use uuid::Uuid;
use wizardrs_core::client_event::ClientEvent;
use wizardrs_core::server_event::ServerEvent;

pub(crate) struct WizardClient {
    pub username: String,
    pub uuid: Uuid,
    server: Arc<WizardServer>,
    event_tx: mpsc::UnboundedSender<ServerEvent>, // send events to client
    leave_tx: watch::Sender<bool>,                // used to notify tasks to shut down
}

impl WizardClient {
    pub async fn new<S: AsyncWrite + AsyncRead + Unpin + Send + 'static>(
        stream: S,
        server: Arc<WizardServer>,
        broadcast_rx: broadcast::Receiver<ServerEvent>,
    ) -> Result<Arc<Self>> {
        let ws_stream = accept_async(stream).await?;
        let (write, mut read) = ws_stream.split();

        let (event_tx, event_rx) = unbounded_channel();
        let (leave_tx, _leave_rx) = watch::channel(false);

        // get username event from client
        let username = {
            let mut name = None;

            while let Some(Ok(msg)) = read.next().await {
                if let Ok(ClientEvent::SetUsername { username }) = serde_json::from_str::<ClientEvent>(&msg.to_string()) {
                    name = Some(username);
                    break;
                }
            }

            if name.is_none() {
                debug!("connection closed unexpectedly");
                return Err(Error::ConnectionClosed);
            }

            name
        };

        let client = Arc::new(Self {
            uuid: Uuid::new_v4(),
            username: username.unwrap(),
            event_tx,
            leave_tx,
            server,
        });

        client.spawn_event_receiver(read);
        client.spawn_event_sender(write, event_rx, broadcast_rx);

        Ok(client)
    }

    fn spawn_event_receiver<S: AsyncWrite + AsyncRead + Unpin + Send + 'static>(
        self: &Arc<Self>,
        mut read: SplitStream<WebSocketStream<S>>,
    ) {
        let client = self.clone();
        let mut leave_rx = self.leave_tx.subscribe();

        tokio::spawn(async move {
            debug!("starting event receiver task: {}", client.uuid);

            let c = client.clone();
            let recv_fut = async move {
                while let Some(Ok(msg)) = read.next().await {
                    if let Ok(event) = serde_json::from_str::<ClientEvent>(&msg.to_string()) {
                        c.handle_client_event(event).await;
                    }
                }
            };

            tokio::select! {
                _ = leave_rx.changed() => {}
                _ = recv_fut => {}
            }

            debug!("stopping event receiver task: {}", client.uuid);
            client.disconnect().await;
        });
    }

    fn spawn_event_sender<S: AsyncWrite + AsyncRead + Unpin + Send + 'static>(
        self: &Arc<Self>,
        mut write: SplitSink<WebSocketStream<S>, Message>,
        mut event_rx: mpsc::UnboundedReceiver<ServerEvent>,
        mut broadcast_rx: broadcast::Receiver<ServerEvent>,
    ) {
        // send events to client
        {
            let client = self.clone();
            let mut leave_rx = self.leave_tx.subscribe();

            tokio::spawn(async move {
                debug!("starting event sender task: {}", client.uuid);

                let c = client.clone();
                let send_fut = async move {
                    while let Some(event) = event_rx.recv().await {
                        let json = serde_json::to_string(&event).unwrap();
                        let msg = Message::text(json);

                        if write.send(msg).await.is_err() {
                            c.disconnect().await;
                        }
                    }
                };

                tokio::select! {
                    _ = leave_rx.changed() => {}
                    _ = send_fut => {}
                }

                debug!("stopping event sender task: {}", client.uuid);
                client.disconnect().await;
            });
        }

        // forward events from channels to event sender
        {
            let client = self.clone();
            let mut leave_rx = self.leave_tx.subscribe();

            tokio::spawn(async move {
                debug!("starting event forwarding task: {}", client.uuid);

                let c = client.clone();
                let broadcast_fut = async move {
                    while let Ok(event) = broadcast_rx.recv().await {
                        c.send_event(event);
                    }
                };

                tokio::select! {
                    _ = leave_rx.changed() => {}
                    _ = broadcast_fut => {}
                }

                debug!("stopping event forwarding task: {}", client.uuid);
                client.disconnect().await;
            });
        }
    }

    pub fn send_event(self: &Arc<Self>, event: ServerEvent) {
        let _ = self.event_tx.send(event);
    }

    pub async fn disconnect(self: &Arc<Self>) {
        // tell tasks to shut down
        self.leave_tx.send_replace(true);

        // remove self from server
        self.server.disconnect_client(self.clone()).await;
    }

    async fn handle_client_event(self: &Arc<Self>, event: ClientEvent) {

    }
}
