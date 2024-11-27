use crate::client::WizardClient;
use crate::error::*;
use crate::server::builder::WizardServerBuilder;
use futures::StreamExt;
use indexmap::IndexMap;
use ngrok::prelude::*;
use ngrok::tunnel::TcpTunnel;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info, trace};
use uuid::Uuid;
use wizardrs_core::server_event::ServerEvent;

pub mod builder;

pub struct WizardServer {
    clients: Arc<RwLock<IndexMap<Uuid, Arc<WizardClient>>>>,
    broadcast_tx: broadcast::Sender<ServerEvent>,
}

impl WizardServer {
    pub fn builder() -> WizardServerBuilder {
        WizardServerBuilder::default()
    }

    pub async fn new(port: u16, ngrok_authtoken: Option<String>) -> Result<Arc<Self>> {
        // start local TcpListener
        let addr = format!("127.0.0.1:{port}");
        info!("starting TcpListener on {addr}");
        let listener = TcpListener::bind(addr).await?;

        // start ngrok tunnel
        let tunnel = match ngrok_authtoken {
            Some(token) => {
                let tunnel = ngrok::Session::builder()
                    .authtoken(token)
                    .connect()
                    .await?
                    .tcp_endpoint()
                    .listen()
                    .await;

                match tunnel {
                    Ok(tunnel) => {
                        info!("started ngrok tunnel on {}", tunnel.url());
                        Some(tunnel)
                    }
                    Err(_) => None,
                }
            }
            None => None,
        };

        let (broadcast_tx, _broadcast_rx) = broadcast::channel(128);

        let server = Arc::new(Self {
            clients: Arc::new(RwLock::new(IndexMap::new())),
            broadcast_tx,
        });

        server.clone().with_tcp_listener(listener);
        if let Some(tunnel) = tunnel {
            server.clone().with_tunnel_listener(tunnel);
        }

        Ok(server)
    }

    fn with_tcp_listener(self: Arc<Self>, listener: TcpListener) {
        let server = self.clone();

        tokio::spawn(async move {
            debug!("starting TcpListener task");

            while let Ok((stream, addr)) = listener.accept().await {
                debug!("new connection: {addr}");

                let client =
                    WizardClient::new(stream, server.clone(), self.broadcast_tx.subscribe()).await;

                match client {
                    Ok(client) => server.add_client(client).await,
                    Err(_) => {}
                }
            }
        });
    }

    fn with_tunnel_listener(self: Arc<Self>, mut tunnel: TcpTunnel) {
        let server = self.clone();

        tokio::spawn(async move {
            while let Some(Ok(conn)) = tunnel.next().await {
                let client =
                    WizardClient::new(conn, server.clone(), self.broadcast_tx.subscribe()).await;

                match client {
                    Ok(client) => server.add_client(client).await,
                    Err(_) => {}
                }
            }
        });
    }

    /// Adds WizardClient to server and broadcasts a join event
    async fn add_client(self: &Arc<Self>, client: Arc<WizardClient>) {
        debug!("successfully established connection to {}", client.uuid);

        // add client to server list
        self.clients
            .write()
            .await
            .insert(client.uuid, client.clone());

        // send join event
        let join_event = ServerEvent::PlayerJoinEvent {
            username: client.username.clone(),
            uuid: client.uuid,
        };
        self.broadcast_event(join_event);
    }

    fn broadcast_event(self: &Arc<Self>, event: ServerEvent) {
        trace!("broadcasting event: {:?}", event);

        let _ = self.broadcast_tx.send(event);
    }

    pub(crate) async fn disconnect_client(self: &Arc<Self>, client: Arc<WizardClient>) {
        if let Some(_) = self
            .clients
            .write()
            .await
            .shift_remove(&client.uuid)
        {
            debug!("disconnected client: {}", client.uuid);
            // remove client from server list
            // broadcast leave event
            let leave_event = ServerEvent::PlayerLeaveEvent {
                username: client.username.clone(),
                uuid: client.uuid,
            };
            self.broadcast_event(leave_event);
        }
    }
}
