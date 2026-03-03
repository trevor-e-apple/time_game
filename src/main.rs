mod app_state;
mod camera_controller;
mod graphics;
mod terminal_state;

use crate::app_state::AppState;

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

struct App<'a> {
    state: Option<AppState<'a>>, // We use option at the top level so that all of app state can be initialized together
}

impl ApplicationHandler for App<'_> {
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
                event: key_event, ..
            } => {
                let code = match key_event.physical_key {
                    PhysicalKey::Code(key_code) => key_code,
                    PhysicalKey::Unidentified(_) => todo!("We may want to ignore this entirely"),
                };
                let key_state = key_event.state;
                match (code, key_state.is_pressed()) {
                    _ => {
                        state.keyboard_input(key_event);
                    }
                }
            }
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
