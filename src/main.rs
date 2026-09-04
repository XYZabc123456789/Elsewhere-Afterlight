//main.rs
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use std::sync::Arc;
use std::error::Error;

/// stands for General Result
type Gresult<T> = Result<T, Box<dyn Error>>;

mod renderer;
use renderer::*;

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    focus: bool,
    gpu: Option<Gpu>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(Arc::new(event_loop.create_window(Window::default_attributes().with_title("Elsewhere Afterlight")).unwrap()));
        self.gpu = Some(pollster::block_on(Gpu::init(self.window.as_ref().unwrap().clone())).unwrap())
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Cross clicked...");
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                let r = self.gpu.as_mut().unwrap().render(self.focus);
                match r {
                    Ok(_) => {},
                    Err(e) => panic!("{:#?}", e),
                }
                self.window.as_mut().unwrap().request_redraw();
            },
            WindowEvent::Resized(s) => {
                self.gpu.as_mut().unwrap().resize(s);
            },
            WindowEvent::Focused(f) => {
                self.focus = f;
            }
            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    let r = event_loop.run_app(&mut app);
    match r {
        Ok(_) => {println!("TERMINATED SUCCESSFULLY")},
        Err(e) => {println!("{:#?}", e)},
    }
}