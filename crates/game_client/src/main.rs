
use game_protocol::{Message, PlayerAction, connection::GameClient};
use game_renderer::{Renderer, Camera, InputHandler};
use winit::{
    event::{WindowEvent, DeviceEvent, KeyEvent},
    event_loop::{EventLoop, ActiveEventLoop},
    window::Window,
    keyboard::{KeyCode, PhysicalKey},
    application::ApplicationHandler,
};
use std::sync::{mpsc, Arc};
use std::thread;

struct GameApp {
    renderer: Option<Renderer>,
    camera: Camera,
    input_handler: InputHandler,
    last_frame_time: std::time::Instant,
    mouse_captured: bool,
    // Network communication channels
    action_sender: Option<mpsc::Sender<PlayerAction>>,
    message_receiver: Option<mpsc::Receiver<Message>>,
}

impl GameApp {
    fn new() -> Self {
        Self {
            renderer: None,
            camera: Camera::new(800.0, 600.0),
            input_handler: InputHandler::new(),
            last_frame_time: std::time::Instant::now(),
            mouse_captured: false,
            action_sender: None,
            message_receiver: None,
        }
    }

    fn start_networking(&mut self) {
        println!("Space Engineers Clone - Starting networking thread");
        
        let (action_tx, action_rx) = mpsc::channel::<PlayerAction>();
        let (message_tx, message_rx) = mpsc::channel::<Message>();
        
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

        // Update camera with FPS controls
        self.input_handler.update_camera(&mut self.camera, dt);

        // Process network messages from networking thread
        if let Some(ref receiver) = self.message_receiver {
            while let Ok(message) = receiver.try_recv() {
                self.handle_server_message(message);
            }
        }
    }

    fn handle_server_message(&self, message: Message) {
        match message {
            Message::Welcome { player_id, world_state } => {
                println!("Welcome! Player {} in world with {} ships", 
                        player_id, world_state.ships.len());
            }
            Message::WorldUpdate { delta: _ } => {
                // Process world updates for rendering
            }
            Message::PlayerJoined { player_id, name, position: _ } => {
                println!("Player '{}' joined (ID: {})", name, player_id);
            }
            Message::PlayerLeft { player_id } => {
                println!("Player {} left", player_id);
            }
            Message::Error { message } => {
                println!("Server error: {}", message);
            }
            _ => {}
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
                            // Remove block action
                            self.send_action(PlayerAction::UseEnergy);
                            println!("Removing block...");
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                self.update();
                
                if let Some(ref mut renderer) = self.renderer {
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
    action_rx: mpsc::Receiver<PlayerAction>,
    message_tx: mpsc::Sender<Message>
) {
    println!("Network thread started");
    
    let mut client = GameClient::new();
    
    // Connection loop
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
    
    // Create a separate client for sending
    let mut send_client = GameClient::new();
    loop {
        match send_client.connect("127.0.0.1:8080").await {
            Ok(()) => break,
            Err(_) => {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                send_client = GameClient::new();
            }
        }
    }
    
    // Spawn message handler
    let message_tx_clone = message_tx.clone();
    tokio::spawn(async move {
        while let Some(message) = client.message_rx.recv().await {
            let _ = message_tx_clone.send(message);
        }
    });
    
    // Handle actions from main thread
    while let Ok(action) = action_rx.recv() {
        let _ = send_client.send_message(Message::PlayerAction { action });
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

