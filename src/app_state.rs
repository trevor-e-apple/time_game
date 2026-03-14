use std::{
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use crate::{game::GameState, graphics::GraphicsState, terminal_state::TerminalState};

use winit::{dpi::LogicalSize, event::WindowEvent, window::Window};

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

    game_state: GameState,
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
            game_state: GameState::new(),
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

        self.game_state
            .update(&mut self.graphics_state, &self.logical_size);

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

    pub fn handle_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::Resized(size) => self.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                // TODO: should update be called somewhere else? Is redrawrequested guaranteed to be called regularly?
                self.update();
                // TODO: handle render errors
                self.render().unwrap();
            }
            _ => match self.terminal_state.handle_event(&event) {
                Ok(_) => {}
                Err(_) => self.game_state.handle_event(&event),
            },
        }
    }

    // pub fn keyboard_input(&mut self, event: KeyEvent) {

    // }
}
