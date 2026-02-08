use std::{
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use crate::graphics::GraphicsState;

use cgmath::Vector2;
use winit::{dpi::LogicalSize, window::Window};

pub struct AppState {
    window: Arc<Window>, // We need window to be an Arc so that the surface can hold a reference to it
    graphics_state: GraphicsState,
    start_time: Instant,
    frame_time: Duration, // ns
    logical_size: LogicalSize<f32>,
}

impl AppState {
    const DEFAULT_FRAME_TIME_MS: u64 = 16;
    /// Function is async because some wgpu functions are async
    pub async fn resumed(window: Arc<Window>) -> anyhow::Result<Self> {
        let graphics_state = GraphicsState::new(window.clone()).await?;

        let logical_size = graphics_state.get_logical_size();

        Ok(Self {
            window,
            graphics_state,
            logical_size,
            start_time: Instant::now(),
            frame_time: Duration::from_millis(Self::DEFAULT_FRAME_TIME_MS),
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.graphics_state.resize(width, height);
        self.logical_size = self.graphics_state.get_logical_size();
    }

    pub fn update(&mut self) {
        self.start_time = Instant::now();

        // Main entities
        {
            self.graphics_state
                .push_textured_quad(
                    Vector2::new(
                        self.logical_size.width / 2.0,
                        self.logical_size.height / 2.0,
                    ),
                    Vector2::new(200.0, 200.0),
                    1,
                    "happy-tree.png",
                )
                .unwrap();
            self.graphics_state
                .push_textured_quad(
                    Vector2::new(0.0, 0.0),
                    Vector2::new(200.0, 200.0),
                    1,
                    "happy-tree-two.png",
                )
                .unwrap();
            self.graphics_state
                .push_textured_quad(
                    Vector2::new(
                        self.logical_size.width / 2.0 + 100.0,
                        self.logical_size.height / 2.0,
                    ),
                    Vector2::new(200.0, 200.0),
                    2,
                    "happy-tree-two.png",
                )
                .unwrap();
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

        {
            let end_time = Instant::now();
            let duration = end_time - self.start_time;

            // TODO: track / log frame time

            if self.frame_time > duration {
                let sleep_time = self.frame_time - duration;
                thread::sleep(sleep_time);
            }
        }
        Ok(())
    }
}
