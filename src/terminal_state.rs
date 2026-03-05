use cgmath::Vector2;
use winit::{
    event::KeyEvent,
    keyboard::{Key, NamedKey},
};

use crate::graphics::GraphicsState;

pub struct TerminalState {
    pub text: String,
    pub has_focus: bool,
    pub width: f32,
    pub height: f32,
    pub x: f32, // Top left corner pos x
    pub y: f32, // Top left corner pos y
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
                        if state.is_pressed() {
                            self.text.pop();
                        }
                    }
                    NamedKey::Space => {
                        if state.is_pressed() {
                            self.text.push(' ');
                        }
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

    pub fn update(&self, graphics_state: &mut GraphicsState) {
        if self.has_focus {
            graphics_state.push_ui_square(
                Vector2 {
                    x: self.x + self.width / 2.0,
                    y: self.y + self.height / 2.0,
                },
                Vector2 {
                    x: self.width,
                    y: self.height,
                },
                0.0,
                (0.0, 0.0, 0.0),
            );
            graphics_state.push_text(
                &self.text,
                0.9 * self.height,
                self.width,
                self.height,
                self.x,
                self.y,
                (1.0, 1.0, 1.0),
            );
        }
    }
}
