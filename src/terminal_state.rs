use winit::{
    event::KeyEvent,
    keyboard::{Key, NamedKey},
};

pub struct TerminalState {
    pub text: String,
    has_focus: bool,
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
}

impl TerminalState {
    pub fn new(width: f32, height: f32, x: f32, y: f32) -> Self {
        Self {
            text: String::new(),
            has_focus: false,
            width,
            height,
            x,
            y,
        }
    }

    pub fn keyboard_input(&mut self, event: KeyEvent) {
        let logical_key = event.logical_key;
        let state = event.state;
        if self.has_focus {
            match logical_key {
                Key::Named(k) => match k {
                    NamedKey::Delete => {
                        todo!();
                    }
                    NamedKey::Backspace => {
                        self.text.pop();
                    }
                    NamedKey::Space => {
                        self.text.push(' ');
                    }
                    NamedKey::Escape => {
                        self.has_focus = false;
                    }
                    _ => {}
                },
                Key::Character(char) => {
                    if state.is_pressed() {
                        let c = char.as_str();
                        self.text.push_str(c);
                    }
                }
                Key::Unidentified(_) => todo!(),
                Key::Dead(_) => todo!(),
            }
        } else {
            match logical_key {
                Key::Named(_) => {}
                Key::Character(char) => {
                    let c = char.as_str();
                    if c == "`" {
                        self.has_focus = true;
                    }
                }
                Key::Unidentified(_) => {}
                Key::Dead(_) => {}
            }
        }
    }
}
