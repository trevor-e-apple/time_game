use winit::{
    event::{KeyEvent, WindowEvent},
    keyboard::{Key, NamedKey},
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

    pub fn keyboard_input(&mut self, event: KeyEvent) {
        let logical_key = event.logical_key;
        let state = event.state;
        if self.active {
            match logical_key {
                Key::Named(k) => match k {
                    NamedKey::Delete => {
                        todo!();
                    }
                    NamedKey::Backspace => {
                        self.text.pop();
                    }
                    _ => {}
                },
                Key::Character(char) => {
                    let c = char.as_str();
                    self.text.push_str(c);
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
                        self.active = true;
                    }
                }
                Key::Unidentified(_) => {}
                Key::Dead(_) => {}
            }
        }
    }
}
