mod app_state;
mod camera;
mod graphics;
mod texture;

use crate::app_state::AppState;

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

struct App {
    state: Option<AppState>, // We use option at the top level so that all of app state can be initialized together
}

impl ApplicationHandler for App {
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(state) => state,
            None => return,
        };

        // TODO: move this to app state?
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                // TODO: should update be called somewhere else? Is redrawrequested guaranteed to be called regularly?
                state.update();
                // TODO: handle render errors
                state.render().unwrap();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => match (code, key_state.is_pressed()) {
                (KeyCode::Escape, true) => {
                    event_loop.exit();
                }
                _ => {
                    state
                        .camera_controller
                        .handle_key(code, key_state.is_pressed());
                }
            },
            _ => (),
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        // Use pollster for lightweight blocking on async function
        self.state = Some(pollster::block_on(AppState::resumed(window)).unwrap());
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    // We use ControlFlow::Poll since we have regular updates without user input
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App { state: None };
    event_loop.run_app(&mut app).unwrap();
}
