use std::sync::Arc;

use crate::{
    camera_controller::CameraController,
    graphics::{GraphicsState, textured_pipeline::TexturedQuad},
};

use cgmath::Vector2;
use winit::window::Window;

pub struct AppState {
    window: Arc<Window>, // We need window to be an Arc so that the surface can hold a reference to it
    graphics_state: GraphicsState,
    pub camera_controller: CameraController,
}

impl AppState {
    /// Function is async because some wgpu functions are async
    pub async fn resumed(window: Arc<Window>) -> anyhow::Result<Self> {
        let camera_controller = CameraController::new(0.01);
        let graphics_state = GraphicsState::new(window.clone()).await?;

        Ok(Self {
            window,
            graphics_state,
            camera_controller,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.graphics_state.resize(width, height)
    }

    pub fn update(&mut self) {
        // Main entities
        {
            let logical_size = self.graphics_state.get_logical_size();
            self.graphics_state.push_textured_quad(TexturedQuad {
                position: Vector2::new(logical_size.width / 2.0, logical_size.height / 2.0),
                dimensions: Vector2::new(200.0, 200.0),
                layer: 1,
            });
        }

        // Debug entities
        {
            self.graphics_state.push_debug_square(
                Vector2::new(50.0, 50.0),
                Vector2::new(30.0, 30.0),
                3.14 / 4.0,
                (1.0, 0.0, 1.0),
            );
            self.graphics_state.push_debug_triangle(
                Vector2::new(100.0, 100.0),
                Vector2::new(30.0, 30.0),
                3.14 / 4.0,
                (0.0, 1.0, 1.0),
            );
        }
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.window.request_redraw();

        self.graphics_state.render()?;

        self.graphics_state.clear_instances();

        Ok(())
    }
}
