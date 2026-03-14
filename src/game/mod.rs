use cgmath::Vector2;
use winit::{
    dpi::LogicalSize,
    event::{KeyEvent, WindowEvent},
};

use crate::graphics::GraphicsState;

pub struct GameState {}

impl GameState {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {}

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
