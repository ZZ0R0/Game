use std::collections::BTreeSet;

use render_wgpu::winit as rwinit;
use rwinit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

#[derive(Default)]
pub struct Input {
    pub keys: BTreeSet<KeyCode>,
    pub rmb_down: bool,
    pub last_cursor: Option<PhysicalPosition<f64>>,
}

impl Input {
    pub fn on_event(&mut self, e: &WindowEvent) {
        match e {
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    match event.state {
                        ElementState::Pressed => {
                            self.keys.insert(code);
                        }
                        ElementState::Released => {
                            self.keys.remove(&code);
                        }
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } if *button == MouseButton::Right => {
                self.rmb_down = *state == ElementState::Pressed;
                if !self.rmb_down {
                    self.last_cursor = None;
                }
            }
            _ => {}
        }
    }

    pub fn held(&self, k: KeyCode) -> bool {
        self.keys.contains(&k)
    }
}
