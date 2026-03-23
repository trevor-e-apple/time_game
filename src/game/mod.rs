mod rect;

use cgmath::Vector2;
use winit::event::WindowEvent;

use crate::{game::rect::Rect, graphics::GraphicsState};

const BOARD_ROWS: usize = 9;
const BOARD_COLUMNS: usize = 9;
const BOARD_SQUARE_DIM: Vector2<f32> = Vector2 { x: 100.0, y: 100.0 };
#[derive(Default, Clone, Copy)]
struct BoardSquare {
    rect: Rect,
    color: (f32, f32, f32),
}

const UNSELECTED_COLOR: (f32, f32, f32) = (0.0, 0.0, 1.0);
const SELECTED_COLOR: (f32, f32, f32) = (0.0, 1.0, 1.0);

#[derive(Default, Clone, Copy)]
struct BoardPiece {
    rect: Rect,
    color: (f32, f32, f32),
    selected: bool,
}

impl BoardPiece {
    fn toggle_select(&mut self) {
        self.selected = !self.selected;
        if self.selected {
            self.color = SELECTED_COLOR;
        } else {
            self.color = UNSELECTED_COLOR;
        }
    }
}

pub struct GameState {
    mouse_position: Vector2<f32>,
    board_squares: [[BoardSquare; BOARD_COLUMNS]; BOARD_ROWS],
    board_pieces: [[Option<BoardPiece>; BOARD_COLUMNS]; BOARD_ROWS],
    selected_piece: Option<(usize, usize)>,
}

impl GameState {
    pub fn new(graphics_state: &GraphicsState<'_>) -> Self {
        let logical_size = graphics_state.get_logical_size();

        // Chess board
        let board_squares = {
            let mut board_squares = [[BoardSquare::default(); BOARD_ROWS]; BOARD_COLUMNS];
            let x_offset =
                (logical_size.width / 2.0) - ((BOARD_ROWS / 2) as f32 * BOARD_SQUARE_DIM.x);
            let y_offset =
                (logical_size.height / 2.0) - ((BOARD_COLUMNS / 2) as f32 * BOARD_SQUARE_DIM.y);
            for row in 0..BOARD_ROWS {
                for column in 0..BOARD_COLUMNS {
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
                        x: (column as f32) * BOARD_SQUARE_DIM.x + x_offset,
                        y: (row as f32) * BOARD_SQUARE_DIM.y + y_offset,
                    };
                    // entities.push(Entity::BoardSquare());
                    board_squares[row][column] = BoardSquare {
                        rect: Rect::with_center(position, BOARD_SQUARE_DIM),
                        color,
                    };
                }
            }

            board_squares
        };

        let board_pieces = [[None; BOARD_COLUMNS]; BOARD_ROWS];

        Self {
            board_squares,
            board_pieces,
            mouse_position: Vector2 { x: 0.0, y: 0.0 },
            selected_piece: None,
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
                    if state.is_pressed() {
                        // Handle mouse up events only currently
                        return Err(());
                    }

                    // Let pieces handle event
                    for row_index in 0..BOARD_ROWS {
                        for column_index in 0..BOARD_COLUMNS {
                            let piece = &mut self.board_pieces[row_index][column_index];
                            match piece {
                                Some(piece) => {
                                    if piece.rect.point_in(mouse_position) {
                                        piece.toggle_select();
                                        if piece.selected {
                                            self.selected_piece = Some((row_index, column_index));
                                        } else {
                                            self.selected_piece = None;
                                        }
                                        return Ok(());
                                    }
                                }
                                None => {}
                            }
                        }
                    }

                    // Let board squares handle event
                    for row_index in 0..BOARD_ROWS {
                        for column_index in 0..BOARD_COLUMNS {
                            let board_square = self.board_squares[row_index][column_index];
                            if board_square.rect.point_in(mouse_position) {
                                match self.board_pieces[row_index][column_index] {
                                    Some(_) => todo!("Attack"),
                                    None => match self.selected_piece {
                                        Some(selected_piece_coordinates) => {
                                            // Move piece
                                            let selected_piece = self.board_pieces
                                                [selected_piece_coordinates.0]
                                                [selected_piece_coordinates.1]
                                                .unwrap();
                                            self.board_pieces[selected_piece_coordinates.0]
                                                [selected_piece_coordinates.1] = None;
                                            self.board_pieces[row_index][column_index] =
                                                Some(selected_piece);
                                        }
                                        None => {
                                            // Create new piece
                                            self.board_pieces[row_index][column_index] =
                                                Some(BoardPiece {
                                                    rect: Rect::with_center(
                                                        board_square.rect.get_center(),
                                                        0.5 * BOARD_SQUARE_DIM,
                                                    ),
                                                    color: UNSELECTED_COLOR,
                                                    selected: false,
                                                });
                                        }
                                    },
                                }
                                return Ok(());
                            }
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
        for square_row in self.board_squares {
            for board_square in square_row {
                graphics_state.push_debug_square(
                    board_square.rect.get_center(),
                    board_square.rect.dim,
                    0.0,
                    board_square.color,
                );
            }
        }

        for piece_row in self.board_pieces {
            for piece in piece_row {
                match piece {
                    Some(piece) => {
                        graphics_state.push_debug_square(
                            piece.rect.get_center(),
                            piece.rect.dim,
                            0.0,
                            piece.color,
                        );
                    }
                    None => {}
                }
            }
        }
    }
}
