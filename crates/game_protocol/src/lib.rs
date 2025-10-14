use serde::{Deserialize, Serialize};
use game_core::objects::*;
use std::collections::HashMap;

/// Protocol messages exchanged between client and server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    // Client to Server messages
    Connect { player_name: String },
    Disconnect,
    PlayerAction { action: PlayerAction },
    
    // Server to Client messages
    Welcome { player_id: u32, world_state: WorldSnapshot },
    WorldSnapshot { snapshot: WorldSnapshot },
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerAction {
    UpdatePosition { position: Position },
    SpawnShip,
}

/// Complete world state snapshot (sent on connect)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub players: HashMap<u32, PlayerState>,
    pub ships: HashMap<u32, ShipState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub name: String,
    pub position: Position,
    pub health: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipState {
    pub name: String,
    pub position: Position,
    pub blocks: Vec<BlockState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockState {
    pub id: u32,
    pub name: String,
    pub block_type: String,
    pub position: RelPosition,
    pub integrity: f32,
}

/// Network connection management using WebSockets
pub mod connection {
    use super::*;
    use anyhow::Result;
    use std::net::SocketAddr;
    use tokio::sync::mpsc;
    use tokio_tungstenite::{accept_async, connect_async, tungstenite::Message as WsMessage};
    use futures_util::{StreamExt, SinkExt};
    use std::collections::HashMap;

    pub struct GameServer {
        pub listener: tokio::net::TcpListener,
        pub connections: HashMap<u32, mpsc::UnboundedSender<Message>>,
        pub message_tx: mpsc::UnboundedSender<(u32, Message)>,
        pub message_rx: mpsc::UnboundedReceiver<(u32, Message)>,
        pub next_client_id: u32,
    }

    impl GameServer {
        pub async fn new(addr: SocketAddr) -> Result<Self> {
            let listener = tokio::net::TcpListener::bind(addr).await?;
            let (message_tx, message_rx) = mpsc::unbounded_channel();

            Ok(Self {
                listener,
                connections: HashMap::new(),
                message_tx,
                message_rx,
                next_client_id: 1,
            })
        }

        pub async fn accept_connections(&mut self) -> Result<()> {
            while let Ok((stream, addr)) = self.listener.accept().await {
                tracing::info!("New connection from: {}", addr);
                
                let client_id = self.next_client_id;
                self.next_client_id += 1;
                
                let tx = self.message_tx.clone();
                let (conn_tx, conn_rx) = mpsc::unbounded_channel();
                self.connections.insert(client_id, conn_tx);
                
                // Spawn task to handle this WebSocket connection
                tokio::spawn(async move {
                    if let Err(e) = handle_websocket_connection(stream, client_id, tx, conn_rx).await {
                        tracing::error!("Connection {} error: {}", client_id, e);
                    }
                });
            }
            Ok(())
        }

        pub fn send_to_client(&self, client_id: u32, message: Message) -> Result<()> {
            if let Some(conn_tx) = self.connections.get(&client_id) {
                conn_tx.send(message)?;
            }
            Ok(())
        }

        pub fn broadcast(&self, message: Message) -> Result<()> {
            for conn_tx in self.connections.values() {
                let _ = conn_tx.send(message.clone());
            }
            Ok(())
        }

        pub fn disconnect_client(&mut self, client_id: u32) {
            self.connections.remove(&client_id);
        }
    }

    pub struct GameClient {
        pub ws_tx: Option<mpsc::UnboundedSender<Message>>,
        pub message_rx: mpsc::UnboundedReceiver<Message>,
    }

    impl GameClient {
        pub fn new() -> Self {
            let (_, message_rx) = mpsc::unbounded_channel();
            Self {
                ws_tx: None,
                message_rx,
            }
        }

        pub async fn connect(&mut self, server_addr: &str) -> Result<()> {
            let url = format!("ws://{}", server_addr);
            let (ws_stream, _) = connect_async(&url).await?;
            let (mut ws_tx_sink, mut ws_rx_stream) = ws_stream.split();
            
            let (tx, mut rx) = mpsc::unbounded_channel();
            self.ws_tx = Some(tx);
            
            let (msg_tx, msg_rx) = mpsc::unbounded_channel();
            self.message_rx = msg_rx;

            // Spawn task to send messages to server
            tokio::spawn(async move {
                while let Some(message) = rx.recv().await {
                    if let Ok(data) = bincode::serialize(&message) {
                        let ws_msg = WsMessage::Binary(data);
                        if ws_tx_sink.send(ws_msg).await.is_err() {
                            break;
                        }
                    }
                }
            });

            // Spawn task to receive messages from server
            tokio::spawn(async move {
                while let Some(msg) = ws_rx_stream.next().await {
                    if let Ok(WsMessage::Binary(data)) = msg {
                        if let Ok(message) = bincode::deserialize::<Message>(&data) {
                            let _ = msg_tx.send(message);
                        }
                    }
                }
            });

            Ok(())
        }

        pub fn send_message(&self, message: Message) -> Result<()> {
            if let Some(tx) = &self.ws_tx {
                tx.send(message)?;
            }
            Ok(())
        }
    }

    pub async fn handle_websocket_connection(
        stream: tokio::net::TcpStream,
        client_id: u32,
        server_tx: mpsc::UnboundedSender<(u32, Message)>,
        mut conn_rx: mpsc::UnboundedReceiver<Message>,
    ) -> Result<()> {
        let ws_stream = accept_async(stream).await?;
        let (mut ws_tx, mut ws_rx) = ws_stream.split();
        
        // Spawn task to send messages to client
        tokio::spawn(async move {
            while let Some(message) = conn_rx.recv().await {
                if let Ok(data) = bincode::serialize(&message) {
                    let ws_msg = WsMessage::Binary(data);
                    if ws_tx.send(ws_msg).await.is_err() {
                        break;
                    }
                }
            }
        });
        
        // Receive messages from client
        while let Some(msg) = ws_rx.next().await {
            match msg {
                Ok(WsMessage::Binary(data)) => {
                    if let Ok(message) = bincode::deserialize::<Message>(&data) {
                        server_tx.send((client_id, message))?;
                    }
                },
                Ok(WsMessage::Close(_)) => break,
                Err(_) => break,
                _ => {}
            }
        }
        
        Ok(())
    }
}

/// Utilities for converting between game_core types and protocol types
pub mod conversion {
    use super::*;
    use game_core::world::GameWorld;

    impl From<&Player> for PlayerState {
        fn from(player: &Player) -> Self {
            Self {
                name: player.name.clone(),
                position: player.physical.placed.position.clone(),
                health: player.health,
            }
        }
    }

    impl From<&LargeGrid> for ShipState {
        fn from(ship: &LargeGrid) -> Self {
            Self {
                name: ship.grid.name.clone(),
                position: ship.grid.physical.placed.position.clone(),
                blocks: ship.large_blocks.iter().map(|b| BlockState {
                    id: b.id,
                    name: b.name.clone(),
                    block_type: b.block_type.clone(),
                    position: b.block.rel_object.position.clone(),
                    integrity: b.block.integrity,
                }).collect(),
            }
        }
    }

    pub fn world_to_snapshot(world: &GameWorld) -> WorldSnapshot {
        WorldSnapshot {
            players: world.players.iter()
                .map(|(&id, player)| (id, PlayerState::from(player)))
                .collect(),
            ships: world.ships.iter()
                .map(|(&id, ship)| (id, ShipState::from(ship)))
                .collect(),
        }
    }
}
