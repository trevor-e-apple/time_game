use cgmath::Vector2;
use winit::{
    event::{KeyEvent, WindowEvent},
    keyboard::{Key, NamedKey, PhysicalKey},
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

    pub fn handle_event(&mut self, event: &WindowEvent) -> Result<(), ()> {
        match event {
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                let logical_key = &key_event.logical_key;
                let state = key_event.state;
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
                    return Ok(());
                } else {
                    match logical_key {
                        Key::Character(char) => {
                            let c = char.as_str();
                            if c == "`" {
                                self.has_focus = true;
                                return Ok(());
                            } else {
                                return Err(());
                            }
                        }
                        _ => {
                            return Err(());
                        }
                    }
                }
            }
            _ => Err(()),
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
            let text_color = (1.0, 1.0, 1.0);
            let font_size = 0.9 * self.height;
            let section_handle = graphics_state.ui.push_text(
                &self.text,
                font_size,
                self.width,
                self.height,
                self.x,
                self.y,
                text_color,
            );
            let (top_left, dim) = graphics_state.ui.get_bounding_box(section_handle);
            if dim.x > 0.0 || dim.y > 0.0 {
                graphics_state.push_ui_square(
                    top_left
                        + Vector2 {
                            x: dim.x,
                            y: 0.5 * dim.y,
                        },
                    Vector2 {
                        x: 1.0,
                        y: font_size,
                    },
                    0.0,
                    text_color,
                );
            }
        }
    }
}
