mod app_state;
mod camera_controller;
mod game;
mod graphics;
mod terminal_state;

use crate::app_state::AppState;

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
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
        match &mut self.state {
            Some(state) => match &event {
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                _ => state.handle_event(event),
            },
            None => return,
        };
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        match window.request_inner_size(PhysicalSize::new(1920, 1080)) {
            Some(_) => {
                // immediate return
            }
            None => {
                // Resized by display system
            }
        };

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
