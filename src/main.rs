#![allow(unused_variables, dead_code, non_snake_case)]

use rand::Rng;

use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::{EventLoop, RenderEvent, WindowSettings};

pub use crate::gameboard_controller::GameboardController;
pub use crate::gameboard_view::{GameboardView, GameboardViewSettings};

mod gameboard_controller;
mod gameboard_view;

#[repr(u64)]
enum BoardPositions {
    WStdStartPos = 0x1008000000,
    BStdStartPos = 0x810000000,
    

    BitBoardDown = 0x00FFFFFFFFFFFFFF,
    BitBoardRight = 0x7F7F7F7F7F7F7F7F,
    BitBoardDownRight = 0x007F7F7F7F7F7F7F,
    BitBoardDownLeft = 0x00FEFEFEFEFEFEFE,
    BitBoardUp = 0xFFFFFFFFFFFFFF00,
    BitBoardLeft = 0xFEFEFEFEFEFEFEFE,
    BitBoardUpRight = 0x7F7F7F7F7F7F7F00,
    BitBoardUpLeft = 0xFEFEFEFEFEFEFE00,
}

#[derive(Copy, Clone)]
pub struct Board {
    occ_squares: [u64; 2],
    num_pieces: [i32; 2],
    side_to_move: usize,
    dist_to_edge: [[usize; 8]; 64]
}

impl Board {

    fn new(w_start_pos: u64, b_start_pos: u64, side_to_move: usize) -> Board {
        let occ_squares = [w_start_pos, b_start_pos];
        let num_pieces = [w_start_pos.count_ones() as i32, b_start_pos.count_ones() as i32];

        let mut dist_to_edge = [[0; 8]; 64];
        for y in 0..8 {
            for x in 0..8 {
                dist_to_edge[y * 8 + x][0] = 8 - y - 1;
                dist_to_edge[y * 8 + x][1] = 8 - x - 1;
                dist_to_edge[y * 8 + x][2] = std::cmp::min(8 - y - 1, 8 - x - 1);
                dist_to_edge[y * 8 + x][3] = std::cmp::min(8 - y - 1, x);
                dist_to_edge[y * 8 + x][4] = y;
                dist_to_edge[y * 8 + x][5] = x;
                dist_to_edge[y * 8 + x][6] = std::cmp::min(y, 8 - x - 1);
                dist_to_edge[y * 8 + x][7] = std::cmp::min(x, y);
            }
        }

        return Board { 
            occ_squares,
            num_pieces,
            side_to_move,
            dist_to_edge
         };
    }

    //Assumes legal move
    fn make_move(&mut self, mv: Move) {

        let mut captured_pieces: u64 = 0;
        let mut n_captured_pieces = 0;
        for direction in 0..8 {
            let mut piece_pos: u64 = 1 << mv.square;
            let mut capture_pieces_dir = 0;
            let mut n_capture_piece_dir = 0;

            for _ in 0..self.dist_to_edge[mv.square as usize][direction] {

                piece_pos = self.shift_bitboard_in_dir(piece_pos, direction);
                if piece_pos & self.get_empty_squares() != 0 {
                    break;
                }
                if piece_pos & self.occ_squares[mv.side] != 0 {
                    captured_pieces |= capture_pieces_dir;
                    n_captured_pieces += n_capture_piece_dir;
                    break;
                }
                capture_pieces_dir |= piece_pos;
                n_capture_piece_dir += 1;
            }
        }
        self.occ_squares[mv.side ^ 1] ^= captured_pieces;
        self.occ_squares[mv.side] |= captured_pieces | (1 << mv.square);
        self.num_pieces[mv.side] += (n_captured_pieces + 1) as i32;
        self.num_pieces[mv.side ^ 1] -= n_captured_pieces as i32;
        self.side_to_move ^= 1;
    }

    //Difficult to implement, might add later
    fn unmake_move(&mut self, mv: Move) {
        self.occ_squares[mv.side] ^= 1 << mv.square;
    }

    fn print_board_state(&self) {
        let mut white_pieces = self.occ_squares[0];
        let mut black_pieces = self.occ_squares[1];
        for i in 0..64 {
            if i != 0 && i % 8 == 0 {
                println!();
            }
            if white_pieces % 2 == 1 {
                print!("W ");
            }
            else if black_pieces % 2 == 1 {
                print!("B ");
            }
            else {
                print!(". ");
            }
            white_pieces = white_pieces >> 1;
            black_pieces = black_pieces >> 1;
        }
        println!();
    }

    fn shift_bitboard_in_dir(&self, bitboard: u64, direction: usize) -> u64 {
        match direction {
            0 => bitboard << 8,
            1 => bitboard << 1,
            2 => bitboard << 9,
            3 => bitboard << 7,
            4 => bitboard >> 8,
            5 => bitboard >> 1,
            6 => bitboard >> 7,
            7 => bitboard >> 9,
            _ => bitboard
        }
    }

    fn get_occ_squares(&self) -> u64 {
        return self.occ_squares[0] | self.occ_squares[1];
    }

    fn get_empty_squares(&self) -> u64 {
        return !self.get_occ_squares();
    }
}

#[derive(Copy, Clone)]
struct MoveGenerator {}

impl MoveGenerator {

    fn new() -> MoveGenerator {
        return MoveGenerator {};
    }

    fn generate_legal_moves(&self, board: &Board) -> u64 {
        let mut legal_moves: u64 = 0;
        for direction in 0..8 {
            legal_moves |= self.generate_moves_in_dir(board, direction);
        }
        return legal_moves;
    }


    fn generate_moves_in_dir(&self, board: &Board, direction: usize) -> u64 {
        let opp = match direction {
            0 => board.occ_squares[(board.side_to_move ^ 1) as usize] & (BoardPositions::BitBoardDown as u64),
            1 => board.occ_squares[(board.side_to_move ^ 1) as usize] & (BoardPositions::BitBoardRight as u64),
            2 => board.occ_squares[(board.side_to_move ^ 1) as usize] & (BoardPositions::BitBoardDownRight as u64),
            3 => board.occ_squares[(board.side_to_move ^ 1) as usize] & (BoardPositions::BitBoardDownLeft as u64),
            4 => board.occ_squares[(board.side_to_move ^ 1) as usize] & (BoardPositions::BitBoardUp as u64),
            5 => board.occ_squares[(board.side_to_move ^ 1) as usize] & (BoardPositions::BitBoardLeft as u64),
            6 => board.occ_squares[(board.side_to_move ^ 1) as usize] & (BoardPositions::BitBoardUpRight as u64),
            7 => board.occ_squares[(board.side_to_move ^ 1) as usize] & (BoardPositions::BitBoardUpLeft as u64),
            _ => board.occ_squares[(board.side_to_move ^ 1) as usize]
        } as u64;

        let own = board.occ_squares[board.side_to_move];
        let mut p_atts = opp & board.shift_bitboard_in_dir(own, direction);
        p_atts |= opp & board.shift_bitboard_in_dir(p_atts, direction);
        p_atts |= opp & board.shift_bitboard_in_dir(p_atts, direction);
        p_atts |= opp & board.shift_bitboard_in_dir(p_atts, direction);
        p_atts |= opp & board.shift_bitboard_in_dir(p_atts, direction);
        p_atts |= opp & board.shift_bitboard_in_dir(p_atts, direction);

        return board.get_empty_squares() & board.shift_bitboard_in_dir(p_atts, direction);
    }


    fn conv_move_bitboard2vec(&self, mut bitboard: u64, side: usize) -> Vec<Move> {
        let mut moves: Vec<Move> = vec![];
        for i in 0..64 {
            let b = bitboard & 1;
            bitboard = bitboard >> 1;
            if b == 1 {
                moves.push(Move::new(i, side));
            }
        }
        return moves;
    }
}

#[derive(Copy, Clone)]
struct Move {
    square: u32,
    side: usize
}

impl Move {
    fn new(square: u32, side: usize) -> Move {
        return Move {
            square,
            side
        };
    }

    fn invalid_move() -> Move {
        let square = 128;
        let side = 2;
        return Move {
            square,
            side
        }
    }
}
struct Search {
    board: Board,
    move_generator: MoveGenerator,
    best_move_this_iter: Move,
    best_move: Move,
    best_eval_this_iter: i32,
    best_eval: i32
}

impl Search {
    fn new(board: Board, move_generator: MoveGenerator) -> Search {
        let best_move_this_iter = Move::invalid_move();
        let best_move = Move::invalid_move();
        let best_eval_this_iter = i32::MIN;
        let best_eval = i32::MIN;
        return Search{
            board, 
            move_generator,
            best_move_this_iter,
            best_move,
            best_eval_this_iter,
            best_eval
        };
    }

    fn evaluate_position(&self) -> i32 {
        return self.board.num_pieces[self.board.side_to_move] - self.board.num_pieces[self.board.side_to_move ^ 1];
    }

    fn find_best_move(&mut self, board: Board, target_depth: u32) {

        self.board = board;
        self.best_move = Move::invalid_move();
        self.best_eval = i32::MIN + 1;
        
        for search_depth in 1..=target_depth {
            self.best_move_this_iter = Move::invalid_move();
            self.best_eval_this_iter = i32::MIN;
            self.search_depth(i32::MIN + 1, i32::MAX, search_depth, 0);
            if self.best_move_this_iter.square < 64 {
                self.best_move = self.best_move_this_iter;
                self.best_eval = self.best_eval_this_iter;
            }
        }
    }

    fn search_depth(&mut self, mut alpha: i32, beta: i32, depth: u32, mvs_made: u32) -> i32 {

        if depth == 0 {
            return self.evaluate_position();
        }
        let moves: u64 = self.move_generator.generate_legal_moves(&self.board);

        if moves == 0 {
            self.board.side_to_move ^= 1;
            let moves: u64 = self.move_generator.generate_legal_moves(&self.board);
            if moves == 0 {
                self.board.side_to_move ^= 1;
                let eval = self.evaluate_position();
                if eval > 0 {
                    return i32::MAX;
                }
                else if eval < 0 {
                    return i32::MIN + 1;
                }
                else {
                    return 0;
                }
            }
            return -self.search_depth(-beta, -alpha, depth - 1, mvs_made + 1);
        }

        let moves: Vec<Move> = self.move_generator.conv_move_bitboard2vec(moves, self.board.side_to_move);

        for i in 0..moves.len() {
            let mv: Move = moves[i];

            let buffer = self.board;
            self.board.make_move(mv);
            let eval = -self.search_depth(-beta, -alpha, depth - 1, mvs_made + 1);
            //self.board.unmake_move(mv);
            self.board = buffer;

            if eval >= beta {
                return beta;
            }
            if eval > alpha {
                alpha = eval;
                if mvs_made == 0 {
                    self.best_move_this_iter = mv;
                    self.best_eval_this_iter = eval;
                }
            }
        }
        return alpha;
    }
}

fn print_bitboard(bitboard: u64) {
    let mut bb = bitboard;
    for i in 0..64 {
        if i != 0 && i % 8 == 0 {
            println!();
        }
        print!("{} ", (bb & 1));
        bb = bb >> 1;
    }
    println!();
}

fn main() {
    let opengl = OpenGL::V3_2;
    let mut window: GlutinWindow = WindowSettings::new("Othello", [900, 600])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut events = Events::new(EventSettings::new().lazy(true));
    let mut gl = GlGraphics::new(opengl);

    let board= Board::new(BoardPositions::WStdStartPos as u64, BoardPositions::BStdStartPos as u64, 1);
    let mut gameboard_controller = GameboardController::new(board);
    let gameboard_view_settings = GameboardViewSettings::new();
    let gameboard_view = GameboardView::new(gameboard_view_settings);

    let move_gen = MoveGenerator::new();
    let mut bot = Search::new(board, move_gen);
    let bot_depth = 9;
    let mut game_over = false;

    while let Some(e) = events.next(&mut window) {

        if !game_over {
            gameboard_controller.event(
                gameboard_view.settings.position,
                gameboard_view.settings.size,
                &e,
            );
        }
        
        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, g| {
                use graphics::clear;

                clear([1.0; 4], g);
                gameboard_view.draw(&gameboard_controller, &c, g);
            });
        }
        if gameboard_controller.gameboard.side_to_move == 1 && !game_over {
            bot.find_best_move(gameboard_controller.gameboard, bot_depth);

            if bot.best_move.square > 63 {
                gameboard_controller.gameboard.side_to_move ^= 1;
                if move_gen.generate_legal_moves(&gameboard_controller.gameboard) == 0 {
                    println!("Game Finished!");
                    game_over = true;
                }
            }
            else {
                gameboard_controller.gameboard.make_move(bot.best_move);
                println!("{}", move_gen.generate_legal_moves(&gameboard_controller.gameboard));
                if move_gen.generate_legal_moves(&gameboard_controller.gameboard) == 0 {
                    gameboard_controller.gameboard.side_to_move ^= 1;
                    if move_gen.generate_legal_moves(&gameboard_controller.gameboard) == 0 {
                        println!("Game Finished!");
                        game_over = true;
                    }
                }
            }
        }
    }
}

fn bot_vs_human(target_depth: u32, play_white: bool) {
    let mut player_side = 0;
    if !play_white {
        player_side = 1;
    }
    let mut board = Board::new(BoardPositions::WStdStartPos as u64, BoardPositions::BStdStartPos as u64, 1);
    let move_generator = MoveGenerator::new();
    let mut bot = Search::new(board, move_generator);
    let stdio = std::io::stdin();
    loop {
        board.print_board_state();
        println!("{} {}", board.num_pieces[0], board.num_pieces[1]);
        if board.side_to_move == player_side {
            let move_board = move_generator.generate_legal_moves(&board);
            let moves = move_generator.conv_move_bitboard2vec(move_board, board.side_to_move);
            for mv in moves {
                print!("{} ", mv.square);
            }
            println!();
            let mut line = String::new();
            stdio.read_line(&mut line).unwrap();
            let square = line.trim().parse::<u32>().unwrap();
            if square > 63 {
                board.side_to_move ^= 1;
                if move_generator.generate_legal_moves(&board) == 0 {
                    println!("Game finished!");
                    break;
                }
                continue;
            }
            board.make_move(Move::new(square, board.side_to_move));
        }
        else {
            bot.find_best_move(board, target_depth);
            if bot.best_move.square > 63 {
                board.side_to_move ^= 1;
                if move_generator.generate_legal_moves(&board) == 0 {
                    println!("Game finished!");
                    break;
                }
                continue;
            }
            board.make_move(bot.best_move);
        }
    }
}

fn bot_vs_bot(target_depth: u32) {
    let mut board = Board::new(BoardPositions::WStdStartPos as u64, BoardPositions::BStdStartPos as u64, 0);
    let move_generator = MoveGenerator::new();
    let mut bot = Search::new(board, move_generator);
    loop {
        board.print_board_state();
        bot.find_best_move(board, target_depth as u32);
        println!("{} {}", bot.best_move.square, board.side_to_move);
        if bot.best_move.square > 63 {
            board.side_to_move ^= 1;
            if move_generator.generate_legal_moves(&board) == 0 {
                println!("Game finished!");
                let diff_pieces = board.num_pieces[0] - board.num_pieces[1];
                if diff_pieces > 0 {
                    println!("White wins with {} pieces to {}!\n", board.num_pieces[0], board.num_pieces[1]);
                }
                else if diff_pieces < 0 {
                    println!("Black wins with {} pieces to {}!\n", board.num_pieces[1], board.num_pieces[0]);
                }
                else if diff_pieces == 0 {
                    println!("Draw!\n");
                }
                break;
            }
            continue;
        }
        board.make_move(bot.best_move);
    }
}

fn gen_random_games(n_games: u32) {
    let move_generation = MoveGenerator::new();
    let mut rng = rand::thread_rng();

    let mut w_wins = 0;
    let mut b_wins = 0;
    let mut draws = 0;

    for rounds in 0..n_games {
        let mut board = Board::new(BoardPositions::WStdStartPos as u64, BoardPositions::BStdStartPos as u64, 0);
        loop {
            let mut move_board = move_generation.generate_legal_moves(&board);
            if move_board == 0 {  
                board.side_to_move ^= 1;
                move_board = move_generation.generate_legal_moves(&board);

                if move_board == 0 {
                    let diff_pieces = board.num_pieces[0] - board.num_pieces[1];
                    if diff_pieces > 0 {
                        w_wins += 1;
                        //println!("Round finished! White wins");
                    }
                    else if diff_pieces < 0 {
                        b_wins += 1;
                        //println!("Round finished! Black wins");
                    }
                    else if diff_pieces == 0 {
                        draws += 1;
                        //println!("Round finished! Draw");
                    }
                    break;
                }
            }
            let moves = move_generation.conv_move_bitboard2vec(move_board, board.side_to_move);
            let random_number = rng.gen_range(0..moves.len());
            board.make_move(moves[random_number]);
        }
    }
    println!("Finished with {} wins for white, {} wins for black and {} draws", w_wins, b_wins, draws);
}