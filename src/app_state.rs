use std::{
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use crate::{graphics::GraphicsState, terminal_state::TerminalState};

use cgmath::Vector2;
use wgpu_text::BrushBuilder;
use winit::{dpi::LogicalSize, event::KeyEvent, window::Window};

/// For tracking the last n frame times
struct FrameTimeBuffer {
    frame_times: [Duration; 1024],
    frame_time_counter: usize,
}

impl FrameTimeBuffer {
    fn new() -> Self {
        Self {
            frame_times: [Duration::new(0, 0); 1024],
            frame_time_counter: 0,
        }
    }
    fn add_frame(&mut self, frame_time: Duration) {
        self.frame_times[self.frame_time_counter % self.frame_times.len()] = frame_time;
        self.frame_time_counter += 1;
    }
}

pub struct AppState<'a> {
    window: Arc<Window>, // We need window to be an Arc so that the surface can hold a reference to it
    graphics_state: GraphicsState<'a>,
    start_time: Instant,
    target_frame_time: Duration, // ns
    logical_size: LogicalSize<f32>,
    frame_time_buffer: FrameTimeBuffer,

    terminal_state: TerminalState,
}

impl AppState<'_> {
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
            target_frame_time: Duration::from_millis(Self::DEFAULT_FRAME_TIME_MS),
            frame_time_buffer: FrameTimeBuffer::new(),
            terminal_state: TerminalState::new(logical_size.width, logical_size.height, 0.0, 0.0),
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
            self.graphics_state.push_textured_quad(
                Vector2::new(
                    self.logical_size.width / 2.0,
                    self.logical_size.height / 2.0,
                ),
                Vector2::new(200.0, 200.0),
                1,
                "happy-tree.png",
            );
            self.graphics_state.push_textured_quad(
                Vector2::new(self.logical_size.width, self.logical_size.height),
                Vector2::new(200.0, 200.0),
                1,
                "happy-tree.png",
            );
            self.graphics_state.push_textured_quad(
                Vector2::new(
                    self.logical_size.width / 2.0,
                    self.logical_size.height / 2.0,
                ),
                Vector2::new(200.0, 200.0),
                2,
                "happy-tree-two.png",
            );

            self.graphics_state.push_textured_quad(
                Vector2::new(
                    self.logical_size.width / 4.0,
                    self.logical_size.height / 4.0,
                ),
                Vector2::new(50.0, 50.0),
                2,
                "happy-tree-two.png",
            );

            self.graphics_state.push_textured_quad(
                Vector2::new(0.0, 0.0),
                Vector2::new(200.0, 200.0),
                2,
                "happy-tree-two.png",
            );
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

        if self.terminal_state.has_focus {
            // TODO: this should probably live with the terminal code
            self.graphics_state.push_debug_square(
                Vector2 {
                    x: self.terminal_state.x + self.terminal_state.width / 2.0,
                    y: self.terminal_state.y + self.terminal_state.height / 2.0,
                },
                Vector2 {
                    x: self.terminal_state.width,
                    y: self.terminal_state.height,
                },
                0.0,
                (0.0, 0.0, 0.0),
            );
            self.graphics_state.push_text(
                &self.terminal_state.text,
                self.terminal_state.width,
                self.terminal_state.height,
                self.terminal_state.x,
                self.terminal_state.y,
                (1.0, 1.0, 1.0),
            );
        }
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.window.request_redraw();

        self.graphics_state.render()?;

        {
            let end_time = Instant::now();
            let duration = end_time - self.start_time;

            // TODO: conditional compilation?
            println!("Frame time: {} us", duration.as_micros());

            if self.target_frame_time > duration {
                let sleep_time = self.target_frame_time - duration;
                thread::sleep(sleep_time);
            }

            // TODO: conditional compilation?
            self.frame_time_buffer.add_frame(duration);
        }
        Ok(())
    }

    pub fn keyboard_input(&mut self, event: KeyEvent) {
        self.terminal_state.keyboard_input(event);
    }
}
