use game_core::{world::GameWorld, objects::Position, validation::ValidationState, prelude::{EntityValidation, ValidationContext}};
use game_protocol::{Message, PlayerAction, connection::GameServer, conversion};
use std::net::SocketAddr;
use std::collections::HashMap;
use tokio::time::{interval, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    println!("🚀 Starting Space Engineers Clone Server...");
    
    let addr: SocketAddr = "127.0.0.1:8080".parse()?;
    let mut server = GameServer::new(addr).await?;
    let mut world = GameWorld::new();
    let mut client_to_player: HashMap<u32, u32> = HashMap::new();
    let mut validation_state = ValidationState::new();
    let validation_context = ValidationContext::new(0.05) // 50ms update rate
        .with_physics_limits(25.0, 50.0); // 25 m/s max speed, 50 m/s² max acceleration
    
    println!("🌐 Server listening on {}", addr);
    println!("📡 WebSocket-based protocol active (QUIC-like functionality)");
    
    // Game update tick every 50ms (20 FPS)
    let mut game_tick = interval(Duration::from_millis(50));
    
    loop {
        tokio::select! {
            // Accept new connections
            Ok((stream, addr)) = server.listener.accept() => {
                tracing::info!("New connection from: {}", addr);
                
                let client_id = server.next_client_id;
                server.next_client_id += 1;
                
                let tx = server.message_tx.clone();
                let (conn_tx, conn_rx) = tokio::sync::mpsc::unbounded_channel();
                server.connections.insert(client_id, conn_tx);
                
                // Spawn task to handle this WebSocket connection
                tokio::spawn(async move {
                    if let Err(e) = game_protocol::connection::handle_websocket_connection(stream, client_id, tx, conn_rx).await {
                        tracing::error!("Connection {} error: {}", client_id, e);
                    }
                });
            }
            
            // Handle incoming messages
            Some((client_id, message)) = server.message_rx.recv() => {
                handle_client_message(&mut server, &mut world, &mut client_to_player, &mut validation_state, &validation_context, client_id, message).await?;
            }
            
            // Game world update tick
            _ = game_tick.tick() => {
                // Valider toutes les entités avant l'update
                let validation_result = validation_state.validate_world(&world, &validation_context);
                
                if validation_result.has_errors() {
                    println!("⚠️ Validation errors detected: {}", validation_result.error_summary());
                    
                    // Optionnel: Corriger les erreurs ou sanctionner les clients
                    // Pour l'instant, on log simplement les erreurs
                }
                
                // Mettre à jour l'état de validation avec l'état actuel
                validation_state.update_from_world(&world);
                
                world.update(0.05); // 50ms delta time
                
                // Send world updates to all clients (delta updates for efficiency)
                // In a real game, you'd only send changes, not full world state
                if !world.players.is_empty() {
                    broadcast_world_updates(&server, &world).await?;
                }
            }
        }
    }
}

async fn handle_client_message(
    server: &mut GameServer,
    world: &mut GameWorld,
    client_to_player: &mut HashMap<u32, u32>,
    validation_state: &mut ValidationState,
    validation_context: &ValidationContext,
    client_id: u32,
    message: Message,
) -> Result<(), Box<dyn std::error::Error>> {
    match message {
        Message::Connect { player_name } => {
            println!("🔗 Player '{}' connecting (Client {})", player_name, client_id);
            
            let spawn_pos = Position::new(0.0, 100.0, 0.0);
            let (player_id, ship_id) = world.spawn_new_player(&player_name, spawn_pos.clone());
            
            // Map client_id to player_id et initialiser l'état de validation
            client_to_player.insert(client_id, player_id);
            validation_state.update_from_world(world); // Mettre à jour l'état initial
            
            // Send welcome message with world state
            let world_snapshot = conversion::world_to_snapshot(world);
            let welcome = Message::Welcome { 
                player_id, 
                world_state: world_snapshot 
            };
            server.send_to_client(client_id, welcome)?;
            
            println!("✅ Player '{}' spawned with ship {} at position (0, 100, 0)", 
                    player_name, ship_id);
        }
        
        Message::PlayerAction { action } => {
            match action {
                PlayerAction::UpdatePosition { position } => {
                    if let Some(&player_id) = client_to_player.get(&client_id) {
                        if let Some(player) = world.players.get_mut(&player_id) {
                            // Sauvegarder l'état actuel pour validation
                            let current_player = player.clone();
                            
                            // Appliquer temporairement le changement
                            player.physical.placed.position = position;
                            
                            // Valider le changement
                            if let Some(previous_player) = validation_state.previous_players.get(&player_id) {
                                let validation_result = player.validate_data(previous_player, validation_context);
                                
                                if !validation_result.is_valid {
                                    // Annuler le changement invalide
                                    *player = current_player;
                                    
                                    // Envoyer correction au client
                                    let correction = Message::WorldSnapshot { 
                                        snapshot: conversion::world_to_snapshot(world) 
                                    };
                                    let _ = server.send_to_client(client_id, correction);
                                    println!("⚠️ Player {} validation failed: {}", 
                                            player_id, validation_result.errors.join(", "));
                                } else {
                                    // Changement accepté
                                    println!("✅ Player {} position update accepted", player_id);
                                }
                            } else {
                                // Premier update, accepter et initialiser l'état
                                validation_state.previous_players.insert(player_id, current_player);
                            }
                        }
                    }
                }
                PlayerAction::SpawnShip => {
                    if let Some(&player_id) = client_to_player.get(&client_id) {
                        if let Some(player) = world.players.get(&player_id) {
                            let player_name = player.name.clone();
                            let ship_id = world.add_ship(&player_name);
                            println!("🚢 Player {} spawned new ship {}", player_name, ship_id);
                        }
                    }
                }
            }
        }
        
        Message::Disconnect => {
            println!("🔌 Client {} disconnected", client_id);
            server.disconnect_client(client_id);
            if let Some(player_id) = client_to_player.remove(&client_id) {
                world.players.remove(&player_id);
                validation_state.previous_players.remove(&player_id);
            }
        }
        
        _ => {
            tracing::warn!("Unhandled message from client {}: {:?}", client_id, message);
        }
    }
    
    Ok(())
}

async fn broadcast_world_updates(
    server: &GameServer,
    world: &GameWorld,
) -> Result<(), Box<dyn std::error::Error>> {
    // Send full world snapshot instead of deltas for simplicity
    let snapshot = game_protocol::conversion::world_to_snapshot(world);
    let update_msg = Message::WorldSnapshot { snapshot };
    server.broadcast(update_msg)?;
    
    Ok(())
}