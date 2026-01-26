use std::sync::Arc;

use crate::{
    camera_controller::CameraController,
    graphics::{
        GraphicsState,
        common_models::{SQUARE_INDICES, TRIANGLE_INDICES},
        textured_pipeline::{SQUARE_VERTICES, TRIANGLE_VERTICES, TexturedInstance},
    },
};

use cgmath::{Quaternion, Vector2, Vector3};
use winit::window::Window;

pub struct AppState {
    window: Arc<Window>, // We need window to be an Arc so that the surface can hold a reference to it
    graphics_state: GraphicsState,
    pub camera_controller: CameraController,
    triangle_index: usize,
    square_index: usize,
}

impl AppState {
    /// Function is async because some wgpu functions are async
    pub async fn resumed(window: Arc<Window>) -> anyhow::Result<Self> {
        let camera_controller = CameraController::new(0.01);
        let mut graphics_state = GraphicsState::new(window.clone()).await?;

        let triangle_index = graphics_state.add_model(TRIANGLE_VERTICES, TRIANGLE_INDICES, 8);
        let square_index = graphics_state.add_model(SQUARE_VERTICES, SQUARE_INDICES, 8);

        graphics_state.add_instance(
            square_index,
            TexturedInstance {
                position: Vector2::new(0.0, 0.0),
                scale: Vector2::new(0.5, 0.5),
                rotation: cgmath::Rad(0.0),
            },
        );

        Ok(Self {
            window,
            graphics_state,
            camera_controller,
            triangle_index,
            square_index,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.graphics_state.resize(width, height)
    }

    pub fn update(&mut self) {
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
