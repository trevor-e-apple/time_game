mod rect;

use cgmath::Vector2;
use rand::{prelude::*, rngs::ThreadRng};
use winit::{dpi::LogicalSize, event::WindowEvent};

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
    show_text: bool,

    attack: i32,
    full_life: i32,
    life: i32,
}

impl BoardPiece {
    fn select(&mut self) {
        self.selected = true;
        self.color = SELECTED_COLOR;
    }

    fn unselect(&mut self) {
        self.selected = false;
        self.color = UNSELECTED_COLOR;
    }
}

pub struct GameState {
    mouse_position: Vector2<f32>,
    board_squares: [[BoardSquare; BOARD_COLUMNS]; BOARD_ROWS],
    board_pieces: [[Option<BoardPiece>; BOARD_COLUMNS]; BOARD_ROWS],
    selected_piece: Option<(usize, usize)>,
    rng: ThreadRng,
}

fn to_ui_space(world_vector: &Vector2<f32>, window_logical_size: LogicalSize<f32>) -> Vector2<f32> {
    Vector2 {
        x: world_vector.x,
        y: window_logical_size.height - world_vector.y,
    }
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

        let rng = rand::rng();

        Self {
            board_squares,
            board_pieces,
            mouse_position: Vector2 { x: 0.0, y: 0.0 },
            selected_piece: None,
            rng,
        }
    }

    fn move_piece(&mut self, from_coordinates: (usize, usize), to_coordinates: (usize, usize)) {
        let mut piece = self.board_pieces[from_coordinates.0][from_coordinates.1].unwrap();
        let board_square = self.board_squares[to_coordinates.0][to_coordinates.1];

        // Update piece rect
        piece.rect.set_center(board_square.rect.get_center());

        // Update piece lookup table
        self.board_pieces[from_coordinates.0][from_coordinates.1] = None;
        self.board_pieces[to_coordinates.0][to_coordinates.1] = Some(piece);
    }

    fn attack_piece(
        &mut self,
        attacking_piece_coordinates: (usize, usize),
        attacked_piece_coordinates: (usize, usize),
    ) {
        let attacking_piece = &self.board_pieces[attacking_piece_coordinates.0]
            [attacked_piece_coordinates.1]
            .unwrap();
        let attacked_piece = &mut self.board_pieces[attacked_piece_coordinates.0]
            [attacked_piece_coordinates.1]
            .unwrap();

        attacked_piece.life -= attacking_piece.attack;

        if attacked_piece.life <= 0 {
            self.board_pieces[attacked_piece_coordinates.0][attacked_piece_coordinates.1] = None;
        }
    }

    fn select_piece(&mut self, piece_coordinates: (usize, usize)) {
        match &mut self.board_pieces[piece_coordinates.0][piece_coordinates.1] {
            Some(piece) => {
                piece.select();
                self.selected_piece = Some(piece_coordinates);
                println!("{:?}", piece.color);
            }
            None => todo!(),
        }
    }

    fn unselect_piece(&mut self, piece_coordinates: (usize, usize)) {
        let piece = &mut self.board_pieces[piece_coordinates.0][piece_coordinates.1].unwrap();
        piece.unselect();
        self.selected_piece = None;
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

                for row_index in 0..BOARD_ROWS {
                    for column_index in 0..BOARD_COLUMNS {
                        match &mut self.board_pieces[row_index][column_index] {
                            Some(board_piece) => {
                                if board_piece.rect.point_in(&self.mouse_position) {
                                    board_piece.show_text = true;
                                } else {
                                    board_piece.show_text = false;
                                }
                            }
                            None => {}
                        }
                    }
                }
                Ok(())
            }
            WindowEvent::MouseInput { state, button, .. } => match button {
                winit::event::MouseButton::Left => {
                    let mouse_position = self.mouse_position;
                    if state.is_pressed() {
                        // Handle mouse up events only currently
                        return Err(());
                    }

                    // Let pieces handle event
                    let mut did_something = false;
                    for row_index in 0..BOARD_ROWS {
                        for column_index in 0..BOARD_COLUMNS {
                            match self.board_pieces[row_index][column_index] {
                                Some(piece) => {
                                    if piece.rect.point_in(&mouse_position) {
                                        // match self.selected_piece {
                                        //     Some(selected_piece_coordinates) => {
                                        //         println!(
                                        //             "Unselecting piece at {:?}",
                                        //             selected_piece_coordinates
                                        //         );
                                        //         self.unselect_piece(selected_piece_coordinates);
                                        //     }
                                        //     None => {}
                                        // }
                                        // println!(
                                        //     "Selecting piece at {:?}",
                                        //     (row_index, column_index)
                                        // );
                                        self.select_piece((row_index, column_index));
                                        did_something = true;
                                    }
                                }
                                None => {}
                            }
                        }
                    }

                    if did_something {
                        for row_index in 0..BOARD_ROWS {
                            for column_index in 0..BOARD_COLUMNS {
                                match &self.board_pieces[row_index][column_index] {
                                    Some(piece) => {
                                        println!("{:?}", piece.color)
                                    }
                                    None => {}
                                }
                            }
                        }
                        return Ok(());
                    }

                    // Let board squares handle event
                    for row_index in 0..BOARD_ROWS {
                        for column_index in 0..BOARD_COLUMNS {
                            let board_square = self.board_squares[row_index][column_index];
                            if board_square.rect.point_in(&mouse_position) {
                                match self.board_pieces[row_index][column_index] {
                                    Some(_) => match self.selected_piece {
                                        Some(selected_piece_coordinates) => {
                                            // Attack other piece
                                            // self.attack_piece(
                                            //     selected_piece_coordinates,
                                            //     (row_index, column_index),
                                            // );
                                        }
                                        None => {
                                            // Don't do anything if there isn't a selected piece
                                        }
                                    },
                                    None => match self.selected_piece {
                                        Some(selected_piece_coordinates) => {
                                            // self.move_piece(
                                            //     selected_piece_coordinates,
                                            //     (row_index, column_index),
                                            // );
                                        }
                                        None => {
                                            println!("Create piece");
                                            // Create new piece
                                            let full_life = self.rng.random_range(1..11);
                                            self.board_pieces[row_index][column_index] =
                                                Some(BoardPiece {
                                                    rect: Rect::with_center(
                                                        board_square.rect.get_center(),
                                                        0.5 * BOARD_SQUARE_DIM,
                                                    ),
                                                    color: UNSELECTED_COLOR,
                                                    selected: false,
                                                    show_text: false,
                                                    attack: self.rng.random_range(0..10),
                                                    full_life,
                                                    life: full_life,
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

        for row_index in 0..BOARD_ROWS {
            for column_index in 0..BOARD_COLUMNS {
                match &self.board_pieces[row_index][column_index] {
                    Some(piece) => {
                        graphics_state.push_debug_square(
                            piece.rect.get_center(),
                            piece.rect.dim,
                            0.0,
                            piece.color,
                        );
                        if piece.show_text {
                            let pos = to_ui_space(
                                &piece.rect.bottom_left,
                                graphics_state.get_logical_size(),
                            );
                            graphics_state.ui.push_text(
                                &format!("Attack: {} Life: {}", piece.attack, piece.life),
                                piece.rect.dim.y / 2.0,
                                1000.0,
                                1000.0,
                                pos.x,
                                pos.y,
                                piece.color,
                            );
                        }
                    }
                    None => {}
                }
            }
        }
    }
}
