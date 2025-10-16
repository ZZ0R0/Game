
use game_protocol::{Message, PlayerAction, connection::GameClient, WorldSnapshot};
use game_renderer::{Renderer, Camera, InputHandler, BlockInstance};
use winit::{
    event::{WindowEvent, DeviceEvent, KeyEvent},
    event_loop::{EventLoop, ActiveEventLoop},
    window::Window,
    keyboard::{KeyCode, PhysicalKey},
    application::ApplicationHandler,
};
use std::sync::{Arc, Mutex};
use std::thread;

/// Thread-safe game state shared between main thread and network thread
#[derive(Debug, Clone)]
struct SharedGameState {
    pub player_position: game_core::objects::Position,
    pub pending_actions: Vec<PlayerAction>,
    pub world_state: Option<WorldSnapshot>,
    pub player_id: Option<u32>,
    pub network_connected: bool,
}

impl Default for SharedGameState {
    fn default() -> Self {
        Self {
            player_position: game_core::objects::Position::new(0.0, 0.0, 0.0),
            pending_actions: Vec::new(),
            world_state: None,
            player_id: None,
            network_connected: false,
        }
    }
}

struct GameApp {
    renderer: Option<Renderer>,
    camera: Camera,
    input_handler: InputHandler,
    last_frame_time: std::time::Instant,
    frame_count: u32,
    fps_timer: std::time::Instant,
    current_fps: f32,
    mouse_captured: bool,
    is_fullscreen: bool,
    // Shared game state (thread-safe)
    shared_state: Arc<Mutex<SharedGameState>>,
    // Network throttling for local updates
    last_network_sync: std::time::Instant,
    network_sync_interval: std::time::Duration,
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
            is_fullscreen: false,
            shared_state: Arc::new(Mutex::new(SharedGameState::default())),
            last_network_sync: now,
            network_sync_interval: std::time::Duration::from_millis(16), // ~60 FPS network sync
        }
    }

    fn start_networking(&mut self) {
        println!("Space Engineers Clone - Starting networking thread");
        
        let shared_state = Arc::clone(&self.shared_state);
        
        // Spawn networking thread with tokio runtime
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                run_network_client(shared_state).await;
            });
        });
    }

    fn update(&mut self) {
        let t1 = std::time::Instant::now();
        let dt = t1.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = t1;

        // Calculate FPS
        self.frame_count += 1;
        let elapsed = t1.duration_since(self.fps_timer).as_secs_f32();
        if elapsed >= 1.0 {
            self.current_fps = self.frame_count as f32 / elapsed;
            self.frame_count = 0;
            self.fps_timer = t1;
        }

        // Update camera with FPS controls
        self.input_handler.update_camera(&mut self.camera, dt);

        // Update shared state with current player position (non-blocking)
        if t1.duration_since(self.last_network_sync) >= self.network_sync_interval {
            if let Ok(mut state) = self.shared_state.try_lock() {
                state.player_position = game_core::objects::Position::new(
                    self.camera.position.x,
                    self.camera.position.y,
                    self.camera.position.z,
                );
            }
            self.last_network_sync = t1;
        }

        let t2 = std::time::Instant::now();
        let d1 = t2.duration_since(t1).as_millis();
        // Get latest world state from shared data (non-blocking)
        let should_update = if let Ok(state) = self.shared_state.try_lock() {
            state.world_state.is_some()
        } else {
            false
        };

        let t3 = std::time::Instant::now();
        let d2 = t3.duration_since(t2).as_millis();

        if should_update {
            if let Ok(state) = self.shared_state.try_lock() {
                if let Some(ref world_state) = state.world_state {
                    if let Some(ref mut renderer) = self.renderer {
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
                                    id: block.id,
                                    version: ship.version,
                                    position: block_world_pos,
                                    texture_path: format!("assets/textures/large_grids/{}.png", block.block_type),
                                });
                            }
                        }
                        
                        renderer.set_blocks_to_render(blocks);
                    }
                }
            }
        }

        let t4 = std::time::Instant::now();
        let d3 = t4.duration_since(t3).as_millis();

        // Update overlay data in renderer
        if let Some(ref mut renderer) = self.renderer {
            let player_pos = if let Ok(state) = self.shared_state.try_lock() {
                if let (Some(ref world_state), Some(player_id)) = (&state.world_state, state.player_id) {
                    world_state.players.get(&player_id).map(|p| {
                        glam::Vec3::new(p.position.x, p.position.y, p.position.z)
                    })
                } else {
                    None
                }
            } else {
                None
            };

            renderer.update_overlay_data(self.current_fps, player_pos);
        }
        let t5 = std::time::Instant::now();
        let d4 = t5.duration_since(t4).as_millis();

       // println!("Update timings: get_state={}ms, process_state={}ms, update_overlay={}ms", d2, d3, d4);
    }



    fn send_action(&self, action: PlayerAction) {
        if let Ok(mut state) = self.shared_state.try_lock() {
            state.pending_actions.push(action);
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
                        KeyCode::F11 => {
                            // Toggle fullscreen
                            if let Some(ref renderer) = self.renderer {
                                self.is_fullscreen = !self.is_fullscreen;
                                if self.is_fullscreen {
                                    renderer.window().set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                                } else {
                                    renderer.window().set_fullscreen(None);
                                }
                                println!("Fullscreen: {}", self.is_fullscreen);
                            }
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

async fn run_network_client(shared_state: Arc<Mutex<SharedGameState>>) {
    println!("Network thread started");
    
    // Connect to server
    let mut client = GameClient::new();
    loop {
        match client.connect("127.0.0.1:8080").await {
            Ok(()) => {
                println!("Connected to server");
                if let Ok(mut state) = shared_state.lock() {
                    state.network_connected = true;
                }
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
    
    // Spawn task to handle server messages
    let shared_state_msgs = Arc::clone(&shared_state);
    tokio::spawn(async move {
        while let Some(message) = client_rx.recv().await {
            handle_network_message(&shared_state_msgs, message).await;
        }
    });
    
    // Main network loop: send pending actions and position updates
    if let Some(tx) = ws_tx {
        let mut last_position_send = std::time::Instant::now();
        let position_send_interval = std::time::Duration::from_millis(16); // ~60 FPS
        
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            
            if let Ok(mut state) = shared_state.try_lock() {
                // Send pending actions
                for action in state.pending_actions.drain(..) {
                    let msg = Message::PlayerAction { action };
                    let _ = tx.send(msg);
                }
                
                // Send position updates periodically
                let now = std::time::Instant::now();
                if now.duration_since(last_position_send) >= position_send_interval {
                    let position_msg = Message::PlayerAction { 
                        action: PlayerAction::UpdatePosition { 
                            position: state.player_position.clone() 
                        }
                    };
                    let _ = tx.send(position_msg);
                    last_position_send = now;
                }
            }
        }
    }
}

async fn handle_network_message(shared_state: &Arc<Mutex<SharedGameState>>, message: Message) {
    if let Ok(mut state) = shared_state.lock() {
        match message {
            Message::Welcome { player_id, world_state } => {
                println!("Welcome! Player {} in world with {} ships", 
                        player_id, world_state.ships.len());
                state.player_id = Some(player_id);
                state.world_state = Some(world_state);
            }
            Message::WorldSnapshot { snapshot } => {
                // Update world state with new snapshot
                state.world_state = Some(snapshot);
            }
            Message::Error { message } => {
                println!("Server error: {}", message);
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Space Engineers Clone - 3D Construction Game");
    println!("Controls: WASD=move, Shift=sprint, Space/Ctrl=up/down, Mouse=look, F=place block, G=spawn ship, ESC=toggle mouse, F11=fullscreen");
    
    let event_loop = EventLoop::new()?;
    let mut app = GameApp::new();
    
    event_loop.run_app(&mut app)?;
    
    Ok(())
}

