mod rect;

use cgmath::Vector2;
use winit::{dpi::LogicalSize, event::WindowEvent};

use crate::{game::rect::Rect, graphics::GraphicsState};

struct ChessSquare {
    rect: Rect,
    color: (f32, f32, f32),
}
enum Entity {
    ChessSquare(ChessSquare),
}

pub struct GameState {
    mouse_position: Vector2<f32>,
    entities: Vec<Entity>,
}

impl GameState {
    pub fn new(graphics_state: &GraphicsState<'_>) -> Self {
        let mut entities = vec![];
        let logical_size = graphics_state.get_logical_size();
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
                    entities.push(Entity::ChessSquare(ChessSquare {
                        rect: Rect::with_center(position, scale),
                        color,
                    }));
                }
            }
        }

        Self {
            entities,
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

    pub fn update(&self, graphics_state: &mut GraphicsState<'_>) {
        for entity in &self.entities {
            match entity {
                Entity::ChessSquare(chess_square) => {
                    graphics_state.push_debug_square(
                        chess_square.rect.get_center(),
                        chess_square.rect.dim,
                        0.0,
                        chess_square.color,
                    );
                }
            }
        }
    }
}
