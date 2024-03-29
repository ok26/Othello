use piston::GenericEvent;

use crate::Board;
use crate::Move;
/// Handles events for Sudoku game.
pub struct GameboardController {
    /// Stores the gameboard state.
    pub gameboard: Board,
    pub selected_cell: Option<[usize; 2]>,
    cursor_pos: [f64; 2]
}

impl GameboardController {
    /// Creates a new gameboard controller.
    pub fn new(gameboard: Board) -> GameboardController {
        GameboardController {
            gameboard: gameboard,
            selected_cell: None,
            cursor_pos: [0.0; 2],
        }
    }

    /// Handles events.
    pub fn event<E: GenericEvent>(&mut self, pos: [f64; 2], size: f64, e: &E) {
        use piston::input::{Button, MouseButton};

        if let Some(pos) = e.mouse_cursor_args() {
            self.cursor_pos = pos;
        }
        if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
            // Find coordinates relative to upper left corner.
            let x = self.cursor_pos[0] - pos[0];
            let y = self.cursor_pos[1] - pos[1];
            // Check that coordinates are inside board boundaries.
            if x >= 0.0 && x < size && y >= 0.0 && y < size {
                // Compute the cell position.
                let cell_x = (x / size * 8.0) as usize;
                let cell_y = (y / size * 8.0) as usize;
                self.gameboard.make_move(Move::new((cell_y * 8 + cell_x) as u32, self.gameboard.side_to_move));
            }
        }
    }
}