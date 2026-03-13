use std::{
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use crate::{
    graphics::GraphicsState,
    terminal_state::{self, TerminalState},
};

use cgmath::Vector2;
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
            terminal_state: TerminalState::new(logical_size.width, 20.0, 0.0, 0.0),
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.graphics_state.resize(width, height);
        self.logical_size = self.graphics_state.get_logical_size();

        // After resizing, disable user resizing
        self.window.set_resizable(false);
    }

    pub fn update(&mut self) {
        self.start_time = Instant::now();

        // Chess board
        {
            let scale = Vector2 { x: 100.0, y: 100.0 };
            let rows = 9;
            let columns = 9;
            let x_offset = (self.logical_size.width / 2.0) - ((rows / 2) as f32 * scale.x);
            let y_offset = (self.logical_size.height / 2.0) - ((columns / 2) as f32 * scale.y);
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
                    self.graphics_state
                        .push_debug_square(position, scale, 0.0, color);
                }
            }
        }

        self.terminal_state.update(&mut self.graphics_state);
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.window.request_redraw();

        self.graphics_state.render()?;

        {
            let end_time = Instant::now();
            let duration = end_time - self.start_time;

            // println!("Frame time: {} us", duration.as_micros());

            if self.target_frame_time > duration {
                let sleep_time = self.target_frame_time - duration;
                thread::sleep(sleep_time);
            }

            self.frame_time_buffer.add_frame(duration);
        }
        Ok(())
    }

    pub fn keyboard_input(&mut self, event: KeyEvent) {
        self.terminal_state.keyboard_input(event);
    }
}
