use std::sync::Arc;

use crate::{camera::CameraController, graphics::GraphicsState};

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
