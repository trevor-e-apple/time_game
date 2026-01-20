use std::sync::Arc;

use crate::{
    camera::CameraController,
    graphics::{
        GraphicsState, Instance, SQUARE_INDICES, SQUARE_VERTICES, TRIANGLE_INDICES,
        TRIANGLE_VERTICES,
    },
};

use cgmath::{Quaternion, Vector3};
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
            Instance {
                position: Vector3::new(0.0, 0.0, 0.0),
                scale: Vector3::new(0.5, 0.5, 1.0),
                rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            },
        );

        graphics_state.add_debug_square(Instance {
            position: Vector3::new(2.0, 0.0, 0.0),
            scale: Vector3::new(0.5, 0.5, 1.0),
            rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
        });

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
        // Update camera
        self.camera_controller
            .update_camera(&mut self.graphics_state.camera);
        self.graphics_state.update_camera_buffer();
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.window.request_redraw();

        self.graphics_state.render()?;

        Ok(())
    }
}
