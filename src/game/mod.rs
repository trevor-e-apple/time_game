mod rect;

use cgmath::Vector2;
use winit::{dpi::LogicalSize, event::WindowEvent};

use crate::{game::rect::Rect, graphics::GraphicsState};

struct Entity {
    rect: Rect,
}

pub struct GameState {
    mouse_position: Vector2<f32>,
    entities: Vec<Entity>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            entities: vec![],
            mouse_position: Vector2 { x: 0.0, y: 0.0 },
        }
    }

    pub fn handle_event(
        &mut self,
        event: &WindowEvent,
        graphics_state: &GraphicsState<'_>,
    ) -> Result<(), ()> {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let logical_pos = graphics_state.to_logical(position);
                self.mouse_position = Vector2 {
                    x: logical_pos.x as f32,
                    y: logical_pos.y as f32,
                };
                println!("{:?}", self.mouse_position);
                Ok(())
            }
            WindowEvent::MouseInput { state, button, .. } => match button {
                winit::event::MouseButton::Left => {
                    let mouse_position = &self.mouse_position;
                    for entity in &self.entities {
                        if entity.rect.point_in(mouse_position) {
                            return Ok(());
                        }
                    }
                    return Err(());
                }
                _ => {
                    return Err(());
                }
            },
            _ => Err(()),
        }
    }

    pub fn update(&self, graphics_state: &mut GraphicsState<'_>, logical_size: &LogicalSize<f32>) {
        // Chess board
        {
            let scale = Vector2 { x: 100.0, y: 100.0 };
            let rows = 9;
            let columns = 9;
            let x_offset = (logical_size.width / 2.0) - ((rows / 2) as f32 * scale.x);
            let y_offset = (logical_size.height / 2.0) - ((columns / 2) as f32 * scale.y);
            for row in 0..rows {
                for column in 0..columns {
                    let color = if row % 2 == 0 {
                        if column % 2 == 0 {
                            (1.0, 1.0, 1.0)
                        } else {
                            (0.0, 0.0, 0.0)
                        }
                    } else {
                        if column % 2 == 0 {
                            (0.0, 0.0, 0.0)
                        } else {
                            (1.0, 1.0, 1.0)
                        }
                    };
                    let position = Vector2 {
                        x: (column as f32) * scale.x + x_offset,
                        y: (row as f32) * scale.y + y_offset,
                    };
                    graphics_state.push_debug_square(position, scale, 0.0, color);
                }
            }
        }
    }
}
