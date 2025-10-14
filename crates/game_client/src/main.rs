
use game_protocol::{Message, PlayerAction, connection::GameClient, WorldSnapshot};
use game_renderer::{Renderer, Camera, InputHandler, BlockInstance};
use winit::{
    event::{WindowEvent, DeviceEvent, KeyEvent},
    event_loop::{EventLoop, ActiveEventLoop},
    window::Window,
    keyboard::{KeyCode, PhysicalKey},
    application::ApplicationHandler,
};
use tokio::sync::mpsc;
use std::sync::Arc;
use std::thread;

struct GameApp {
    renderer: Option<Renderer>,
    camera: Camera,
    input_handler: InputHandler,
    last_frame_time: std::time::Instant,
    frame_count: u32,
    fps_timer: std::time::Instant,
    current_fps: f32,
    mouse_captured: bool,
    world_state: Option<WorldSnapshot>,
    player_id: Option<u32>,
    // Network communication channels
    action_sender: Option<mpsc::UnboundedSender<PlayerAction>>,
    message_receiver: Option<mpsc::UnboundedReceiver<Message>>,
}

impl GameApp {
    fn new() -> Self {
        let now = std::time::Instant::now();
        Self {
            renderer: None,
            camera: Camera::new(800.0, 600.0),
            input_handler: InputHandler::new(),
            last_frame_time: now,
            frame_count: 0,
            fps_timer: now,
            current_fps: 0.0,
            mouse_captured: false,
            world_state: None,
            player_id: None,
            action_sender: None,
            message_receiver: None,
        }
    }

    fn start_networking(&mut self) {
        println!("Space Engineers Clone - Starting networking thread");
        
        let (action_tx, action_rx) = mpsc::unbounded_channel::<PlayerAction>();
        let (message_tx, message_rx) = mpsc::unbounded_channel::<Message>();
        
        // Store channels
        self.action_sender = Some(action_tx);
        self.message_receiver = Some(message_rx);
        
        // Spawn networking thread with tokio runtime
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                run_network_client(action_rx, message_tx).await;
            });
        });
    }

    fn update(&mut self) {
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        // Calculate FPS
        self.frame_count += 1;
        let elapsed = now.duration_since(self.fps_timer).as_secs_f32();
        if elapsed >= 1.0 {
            self.current_fps = self.frame_count as f32 / elapsed;
            self.frame_count = 0;
            self.fps_timer = now;
        }

        // Update camera with FPS controls
        self.input_handler.update_camera(&mut self.camera, dt);

        // Send player position to server
        if let Some(ref sender) = self.action_sender {
            use game_core::objects::Position;
            let position = Position::new(
                self.camera.position.x,
                self.camera.position.y,
                self.camera.position.z,
            );
            let _ = sender.send(PlayerAction::UpdatePosition { position });
        }

        // Process network messages from networking thread
        if let Some(ref mut receiver) = self.message_receiver {
            let mut messages = Vec::new();
            while let Ok(message) = receiver.try_recv() {
                messages.push(message);
            }
            for message in messages {
                self.handle_server_message(message);
            }
        }

        // Update overlay data in renderer
        if let Some(ref mut renderer) = self.renderer {
            let player_pos = if let (Some(ref world_state), Some(player_id)) = (&self.world_state, self.player_id) {
                world_state.players.get(&player_id).map(|p| {
                    glam::Vec3::new(p.position.x, p.position.y, p.position.z)
                })
            } else {
                None
            };

            renderer.update_overlay_data(self.current_fps, player_pos);
        }
    }

    fn handle_server_message(&mut self, message: Message) {
        match message {
            Message::Welcome { player_id, world_state } => {
                println!("Welcome! Player {} in world with {} ships", 
                        player_id, world_state.ships.len());
                self.player_id = Some(player_id);
                self.world_state = Some(world_state);
                self.update_render_data();
            }
            Message::WorldSnapshot { snapshot } => {
                // Update world state with new snapshot
                self.world_state = Some(snapshot);
                self.update_render_data();
            }
            Message::Error { message } => {
                println!("Server error: {}", message);
            }
            _ => {}
        }
    }

    fn update_render_data(&mut self) {
        if let (Some(ref world_state), Some(ref mut renderer)) = (&self.world_state, &mut self.renderer) {
            let mut blocks = Vec::new();
            
            // Convert all ships' blocks to BlockInstances
            for (_ship_id, ship) in &world_state.ships {
                let ship_pos = glam::Vec3::new(
                    ship.position.x,
                    ship.position.y,
                    ship.position.z,
                );
                
                for block in &ship.blocks {
                    // Calculate world position: ship position + block relative position (2.5m per block for large grids)
                    let block_world_pos = ship_pos + glam::Vec3::new(
                        block.position.x as f32 * 2.5,
                        block.position.y as f32 * 2.5,
                        block.position.z as f32 * 2.5,
                    );
                    
                    blocks.push(BlockInstance {
                        position: block_world_pos,
                        texture_path: format!("assets/textures/large_grids/{}.png", block.block_type),
                    });
                }
            }
            
            renderer.set_blocks_to_render(blocks);
        }
    }

    fn send_action(&self, action: PlayerAction) {
        if let Some(ref sender) = self.action_sender {
            let _ = sender.send(action);
        }
    }


}

impl ApplicationHandler for GameApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Space Engineers Clone")
            .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0));
            
        let window = event_loop.create_window(window_attributes).unwrap();
        let window = Arc::new(window);
        
        // Create renderer
        let renderer = pollster::block_on(Renderer::new(window.clone())).unwrap();
        self.renderer = Some(renderer);
        
        // Start networking thread
        self.start_networking();
        
        // Capture mouse for FPS controls
        if let Some(ref renderer) = self.renderer {
            renderer.window().set_cursor_visible(false);
            let _ = renderer.window().set_cursor_grab(winit::window::CursorGrabMode::Confined);
            self.mouse_captured = true;
        }
        
        // TODO: Connect to server when networking is implemented
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: winit::window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Space Engineers Clone shutting down...");
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(ref mut renderer) = self.renderer {
                    renderer.resize(physical_size);
                    self.camera.update_aspect(physical_size.width as f32, physical_size.height as f32);
                }
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(key_code),
                    state,
                    ..
                },
                ..
            } => {
                // Handle FPS controls
                self.input_handler.process_keyboard(key_code, state);
                
                // Handle game actions
                if state == winit::event::ElementState::Pressed {
                    match key_code {
                        KeyCode::Escape => {
                            // Toggle mouse capture
                            if let Some(ref renderer) = self.renderer {
                                self.mouse_captured = !self.mouse_captured;
                                renderer.window().set_cursor_visible(!self.mouse_captured);
                                if self.mouse_captured {
                                    let _ = renderer.window().set_cursor_grab(winit::window::CursorGrabMode::Confined);
                                } else {
                                    let _ = renderer.window().set_cursor_grab(winit::window::CursorGrabMode::None);
                                }
                            }
                        }
                        KeyCode::KeyF => {
                            // Place block action
                            self.send_action(PlayerAction::SpawnShip);
                            println!("Placing block...");
                        }
                        KeyCode::KeyG => {
                            // Remove block action / Spawn ship
                            self.send_action(PlayerAction::SpawnShip);
                            println!("Spawning ship...");
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                self.update();
                
                if let Some(ref mut renderer) = self.renderer {
                    // Update camera matrices in renderer
                    renderer.update_camera(&self.camera);
                    
                    match renderer.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.size()),
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(e) => eprintln!("Render error: {:?}", e),
                    }
                }
            }
            _ => {}
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: winit::event::DeviceId, event: DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                if self.mouse_captured {
                    self.input_handler.process_mouse(delta);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(ref renderer) = self.renderer {
            renderer.window().request_redraw();
        }
    }
}

async fn run_network_client(
    mut action_rx: mpsc::UnboundedReceiver<PlayerAction>,
    message_tx: mpsc::UnboundedSender<Message>
) {
    println!("Network thread started");
    
    // Connect to server
    let mut client = GameClient::new();
    loop {
        match client.connect("127.0.0.1:8080").await {
            Ok(()) => {
                println!("Connected to server");
                break;
            }
            Err(_) => {
                println!("Retrying connection...");
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                client = GameClient::new();
            }
        }
    }
    
    // Send connect message
    let _ = client.send_message(Message::Connect { 
        player_name: "SpaceEngineer".to_string()
    });
    
    // Get sender and receiver
    let ws_tx = client.ws_tx.clone();
    let mut client_rx = client.message_rx;
    
    // Spawn task to forward server messages to main thread
    let message_tx_clone = message_tx.clone();
    tokio::spawn(async move {
        while let Some(message) = client_rx.recv().await {
            let _ = message_tx_clone.send(message);
        }
    });
    
    // Send player actions to server
    if let Some(tx) = ws_tx {
        while let Some(action) = action_rx.recv().await {
            let msg = Message::PlayerAction { action };
            let _ = tx.send(msg);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Space Engineers Clone - 3D Construction Game");
    println!("Controls: WASD=move, Mouse=look, F=place block, G=remove block, ESC=toggle mouse");
    
    let event_loop = EventLoop::new()?;
    let mut app = GameApp::new();
    
    event_loop.run_app(&mut app)?;
    
    Ok(())
}

