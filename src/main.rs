mod board;
mod pgn;
mod graphics;
mod engine;
mod fen;

use std::str::FromStr;
use std::sync::mpsc;
use std::thread;
use board::ChessBoard;
use pgn::ChessGamePlayer;
use graphics::{Button, load_images, draw_ui};
use ggez::{Context, GameResult, ContextBuilder, event, GameError};
use ggez::event::{EventHandler, MouseButton};
use ggez::mint::Point2;
use shakmaty::Square;
use crate::engine::StockfishEngine;
use crate::fen::pgn_to_fen_at_move;
use crate::pgn::square_to_board_coord;

const SAMPLE_PGN: &str = r#"[Event "Live Chess"]
[Site "Chess.com"]
[Date "2025.03.24"]
[Round "?"]
[White "boolean0101"]
[Black "joks7777"]
[Result "1-0"]
[TimeControl "120+1"]
[WhiteElo "514"]
[BlackElo "481"]
[Termination "boolean0101 won by checkmate"]
[ECO "B23"]
[EndTime "16:39:30 GMT+0000"]
[Link "https://www.chess.com/game/live/136625271350?move=0"]

1. e4 c5 2. Nc3 e5 3. Bc4 a6 4. d3 d6 5. Nf3 Nf6 6. Bg5 Be7 7. O-O b5 8. Bd5
Nxd5 9. Nxd5 Bxg5 10. Nxg5 Qxg5 11. Nc7+ Kd7 12. Nxa8 Nc6 13. Nb6+ Ke6 14. Nxc8
Rxc8 15. Qf3 Qf4 16. Qxf4 exf4 17. c3 b4 18. Rfe1 bxc3 19. bxc3 Ne5 20. g3 fxg3
21. fxg3 Nf3+ 22. Kf2 Nxe1 23. Rxe1 a5 24. Rb1 g5 25. Rb5 a4 26. Ra5 f5 27.
exf5+ Kxf5 28. Rxa4 Rf8 29. Kg2 h5 30. Rc4 Re8 31. Kf2 Re5 32. a4 Ke6 33. a5
Rf5+ 34. Ke2 d5 35. Rxc5 Re5+ 36. Kd2 Kd6 37. Rb5 Kc6 38. Rb3 d4 39. Ra3 dxc3+
40. Kxc3 Rc5+ 41. Kd2 Kb5 42. a6 Rc8 43. a7 Ra8 44. Kc3 Kb6 45. d4 Rc8+ 46. Kb4
Ra8 47. d5 Kc7 48. Kc5 Kd7 49. d6 Rc8+ 50. Kd5 Ra8 51. Ke5 Re8+ 52. Kd5 Ra8 53.
Kc5 g4 54. Kd5 h4 55. gxh4 Rh8 56. a8=Q Rxa8 57. Rxa8 g3 58. Kc5 g2 59. Ra1 Ke6
60. Rg1 Kd7 61. Rxg2 Ke6 62. h5 Kf7 63. h6 Kf6 64. h7 Ke5 65. h8=Q+ Ke4 66. Qf8
Kd3 67. Rf2 Ke3 68. Qf3# 1-0"#;

struct GameState {
    engine: StockfishEngine,
    images: std::collections::HashMap<String, ggez::graphics::Image>,
    game_player: ChessGamePlayer,
    prev_button: Button,
    next_button: Button,
    reset_button: Button,
    end_button: Button,
    flip_button: Button,
    board_flipped: bool,
    game_info: String,
    current_arrow: Option<(Point2<f32>, Point2<f32>)>,
    best_move_receiver: Option<mpsc::Receiver<Option<Vec<String>>>>,
    finding_best_move: bool,
    evaluation: f32,
    debug_mode: bool,
}

impl GameState {
    fn new(ctx: &mut Context) -> GameResult<GameState> {
        let debug_mode = false;
        let grid_size = 72.0;
        let board = ChessBoard::new(grid_size);
        let context = ctx;
        let engine = StockfishEngine::new(debug_mode);
        let images = load_images(context)?;

        let prev_button = Button::new(100.0, 800.0, 80.0, 40.0, "Prev");
        let next_button = Button::new(200.0, 800.0, 80.0, 40.0, "Next");
        let reset_button = Button::new(300.0, 800.0, 80.0, 40.0, "Start");
        let end_button = Button::new(400.0, 800.0, 80.0, 40.0, "End");
        let flip_button = Button::new(500.0, 800.0, 80.0, 40.0, "Flip");

        let game_player = ChessGamePlayer::new(board.clone());

        let mut state = GameState {
            engine,
            images,
            game_player,
            prev_button,
            next_button,
            reset_button,
            end_button,
            flip_button,
            board_flipped: false,
            game_info: "No game loaded".to_string(),
            current_arrow: None,
            best_move_receiver: None,
            finding_best_move: false,
            evaluation: 0.0,
            debug_mode,
        };

        state.load_pgn_string(SAMPLE_PGN);
        Ok(state)
    }

    pub fn flip_board(&mut self) {
        self.board_flipped = !self.board_flipped;
        self.trigger_find_best_move();
    }

    pub fn load_pgn_string(&mut self, pgn_content: &str) {
        if self.game_player.load_pgn(pgn_content) {
            let headers = self.game_player.get_headers();
            let mut white = "Unknown";
            let mut black = "Unknown";
            let mut event = "Unknown";

            for (key, value) in headers {
                match key.as_str() {
                    "White" => white = value,
                    "Black" => black = value,
                    "Event" => event = value,
                    _ => {}
                }
            }

            self.game_info = format!("{}: {} vs {}", event, white, black);
        } else {
            println!("Failed to load PGN");
            self.game_info = "Failed to load game".to_string();
        }
    }

    pub fn reset_position(&mut self) {
        self.game_player.reset();
    }

    pub fn go_to_end(&mut self) {
        self.reset_position();

        let total_moves = self.game_player.get_total_moves();
        for _ in 0..total_moves {
            self.game_player.next_move();
        }
    }

    fn trigger_find_best_move(&mut self) {
        let current_move = self.game_player.get_current_move();
        let engine_clone = self.engine.clone();

        let (tx, rx) = mpsc::channel();
        self.best_move_receiver = Some(rx);

        let fen = pgn_to_fen_at_move(SAMPLE_PGN, current_move).unwrap();
        println!("Getting best move for FEN: {}", fen);
        
        thread::spawn(move || {
            {
                let engine = engine_clone.lock();
                engine.set_position(&fen).unwrap();
            }

            let best_move_option = {
                let engine = engine_clone.lock();
                engine.find_best_move(Some(16), None)
            };

            tx.send(best_move_option).unwrap();
        });
    }

    fn set_arrow_coords(&mut self, from_coords: Point2<usize>, to_coords: Point2<usize>) {
        let (display_from_row, display_from_col) = if self.board_flipped {
            (from_coords.x, 7 - from_coords.y)
        } else {
            (7 - from_coords.x, from_coords.y)
        };

        let (display_to_row, display_to_col) = if self.board_flipped {
            (to_coords.x, 7 - to_coords.y)
        } else {
            (7 - to_coords.x, to_coords.y)
        };

        let grid_size = self.game_player.board.grid_size;
        
        let from_center = Point2 {
            x: graphics::START_X + (display_from_col as f32 * grid_size) + (grid_size / 2.0),
            y: graphics::START_Y + (display_from_row as f32 * grid_size) + (grid_size / 2.0)
        };

        let to_center = Point2 {
            x: graphics::START_X + (display_to_col as f32 * grid_size) + (grid_size / 2.0),
            y: graphics::START_Y + (display_to_row as f32 * grid_size) + (grid_size / 2.0)
        };

        self.current_arrow = Some((from_center, to_center));
    }

    pub fn next_move(&mut self) {
        if self.finding_best_move {
            return;
        }

        if self.game_player.next_move() {
            {
                let engine = self.engine.lock();
                let fen = pgn_to_fen_at_move(SAMPLE_PGN, self.game_player.current_move).unwrap();
                let is_white_move = fen.split_whitespace().nth(1).unwrap_or("b") == "w";
                let evaluation = engine.get_evaluation_score(17, is_white_move).unwrap();
                self.evaluation = evaluation;
                println!("eval score: {evaluation}");
            }

            self.finding_best_move = true;
            self.trigger_find_best_move();
        }
    }

    pub fn prev_move(&mut self) {
        if self.finding_best_move {
            return;
        }

        if self.game_player.previous_move() {
            {
                let engine = self.engine.lock();
                let fen = pgn_to_fen_at_move(SAMPLE_PGN, self.game_player.current_move).unwrap();
                let is_white_move = fen.split_whitespace().nth(1).unwrap_or("b") == "w";
                let evaluation = engine.get_evaluation_score(17, is_white_move).unwrap();
                self.evaluation = evaluation;
                println!("eval score: {evaluation}");
            }

            self.finding_best_move = true;
            self.trigger_find_best_move();
        }
    }
}

impl EventHandler for GameState {
    fn update(&mut self, _: &mut Context) -> GameResult {
        if let Some(ref receiver) = self.best_move_receiver {
            if let Ok(best_move_option) = receiver.try_recv() {
                if let Some(best_move) = best_move_option {
                    let from_coords = square_to_board_coord(Square::from_str(&best_move[0]).unwrap());
                    let to_coords = square_to_board_coord(Square::from_str(&best_move[1]).unwrap());
                    self.set_arrow_coords(from_coords, to_coords);
                }

                self.best_move_receiver = None;
                self.finding_best_move = false;
            }
        }



        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let buttons = [
            &self.prev_button,
            &self.next_button,
            &self.reset_button,
            &self.end_button,
            &self.flip_button,
        ];

        draw_ui(
            ctx,
            &self.game_player.board,
            &self.images,
            &buttons,
            &self.game_info,
            self.game_player.get_current_move(),
            self.game_player.get_total_moves(),
            self.board_flipped,
            self.current_arrow,
            self.debug_mode,
            self.evaluation,
        )
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> Result<(), GameError> {
        if button == MouseButton::Left {
            let pos = [x, y];

            if self.prev_button.contains_point(pos) {
                self.prev_button.pressed = true
            } else if self.next_button.contains_point(pos) {
                self.next_button.pressed = true;
            } else if self.reset_button.contains_point(pos) {
                self.reset_button.pressed = true;
            } else if self.end_button.contains_point(pos) {
                self.end_button.pressed = true;
            } else if self.flip_button.contains_point(pos) {
                self.flip_button.pressed = true;
            }
        }

        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> Result<(), GameError> {
        if button == MouseButton::Left {
            let pos = [x, y];

            if self.prev_button.contains_point(pos) && self.prev_button.pressed {
                self.prev_move();
            } else if self.next_button.contains_point(pos) && self.next_button.pressed {
                self.next_move();
            } else if self.reset_button.contains_point(pos) && self.reset_button.pressed {
                self.reset_position();
            } else if self.end_button.contains_point(pos) && self.end_button.pressed {
                self.go_to_end();
            } else if self.flip_button.contains_point(pos) && self.flip_button.pressed {
                self.flip_board();
            }

            self.prev_button.pressed = false;
            self.next_button.pressed = false;
            self.reset_button.pressed = false;
            self.end_button.pressed = false;
            self.flip_button.pressed = false;
        }

        Ok(())
    }
}

fn main() -> GameResult {
    let (mut ctx, event_loop) = ContextBuilder::new("Brilliant", "BongoSLAP")
        .window_mode(
            ggez::conf::WindowMode::default()
                .dimensions(900.0, 900.0)
        )
        .build()?;

    let state = GameState::new(&mut ctx)?;
    event::run(ctx, event_loop, state);
}