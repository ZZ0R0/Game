use render_wgpu::winit as rwinit;

use rwinit::event_loop::EventLoop;

use engine_app::app::App;

fn main() {
    let event_loop = EventLoop::new().expect("event_loop");
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
