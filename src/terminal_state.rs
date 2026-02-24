use winit::{
    event::{KeyEvent, WindowEvent},
    keyboard::Key,
};

pub struct TerminalState {
    text: String,
    active: bool,
}

impl TerminalState {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            active: false,
        }
    }

    pub fn keyboard_input(event: KeyEvent) {
        let logical_key = event.logical_key;
        let state = event.state;
        match logical_key {
            Key::Named(named_key) => todo!(),
            Key::Character(_) => todo!(),
            Key::Unidentified(native_key) => todo!(),
            Key::Dead(_) => todo!(),
        }
    }
}
