mod board;
mod pgn;

use board::{ChessBoard, PieceType};
use pgn::ChessGamePlayer;
use ggez::{Context, GameResult, ContextBuilder, event, GameError};
use ggez::graphics::{Color, DrawMode, DrawParam, Image, Mesh, Rect, Canvas, Text, TextFragment, Drawable};
use ggez::event::{EventHandler, MouseButton};

const BLACK_KING: &[u8] = include_bytes!("../resources/black-king.png");
const BLACK_QUEEN: &[u8] = include_bytes!("../resources/black-queen.png");
const BLACK_ROOK: &[u8] = include_bytes!("../resources/black-rook.png");
const BLACK_BISHOP: &[u8] = include_bytes!("../resources/black-bishop.png");
const BLACK_KNIGHT: &[u8] = include_bytes!("../resources/black-knight.png");
const BLACK_PAWN: &[u8] = include_bytes!("../resources/black-pawn.png");
const WHITE_KING: &[u8] = include_bytes!("../resources/white-king.png");
const WHITE_QUEEN: &[u8] = include_bytes!("../resources/white-queen.png");
const WHITE_ROOK: &[u8] = include_bytes!("../resources/white-rook.png");
const WHITE_BISHOP: &[u8] = include_bytes!("../resources/white-bishop.png");
const WHITE_KNIGHT: &[u8] = include_bytes!("../resources/white-knight.png");
const WHITE_PAWN: &[u8] = include_bytes!("../resources/white-pawn.png");

// Hard-coded sample PGN for testing
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

struct Button {
    rect: Rect,
    text: String,
    pressed: bool,
}

impl Button {
    fn new(x: f32, y: f32, width: f32, height: f32, text: &str) -> Self {
        Button {
            rect: Rect::new(x, y, width, height),
            text: text.to_string(),
            pressed: false,
        }
    }

    fn contains_point(&self, point: [f32; 2]) -> bool {
        self.rect.contains(point)
    }
}

struct GameState {
    board: ChessBoard,
    images: std::collections::HashMap<String, Image>,
    game_player: ChessGamePlayer,
    prev_button: Button,
    next_button: Button,
    reset_button: Button,
    end_button: Button,
    flip_button: Button,
    board_flipped: bool,
    game_info: String,
}

impl GameState {
    fn new(ctx: &mut Context) -> GameResult<GameState> {
        let grid_size = 72.0;
        let board = ChessBoard::new(grid_size);

        let mut images = std::collections::HashMap::new();

        images.insert("black-king".to_string(), Image::from_bytes(ctx, BLACK_KING)?);
        images.insert("black-queen".to_string(), Image::from_bytes(ctx, BLACK_QUEEN)?);
        images.insert("black-rook".to_string(), Image::from_bytes(ctx, BLACK_ROOK)?);
        images.insert("black-bishop".to_string(), Image::from_bytes(ctx, BLACK_BISHOP)?);
        images.insert("black-knight".to_string(), Image::from_bytes(ctx, BLACK_KNIGHT)?);
        images.insert("black-pawn".to_string(), Image::from_bytes(ctx, BLACK_PAWN)?);
        images.insert("white-king".to_string(), Image::from_bytes(ctx, WHITE_KING)?);
        images.insert("white-queen".to_string(), Image::from_bytes(ctx, WHITE_QUEEN)?);
        images.insert("white-rook".to_string(), Image::from_bytes(ctx, WHITE_ROOK)?);
        images.insert("white-bishop".to_string(), Image::from_bytes(ctx, WHITE_BISHOP)?);
        images.insert("white-knight".to_string(), Image::from_bytes(ctx, WHITE_KNIGHT)?);
        images.insert("white-pawn".to_string(), Image::from_bytes(ctx, WHITE_PAWN)?);

        let prev_button = Button::new(100.0, 800.0, 80.0, 40.0, "Prev");
        let next_button = Button::new(200.0, 800.0, 80.0, 40.0, "Next");
        let reset_button = Button::new(300.0, 800.0, 80.0, 40.0, "Start");
        let end_button = Button::new(400.0, 800.0, 80.0, 40.0, "End");
        let flip_button = Button::new(500.0, 800.0, 80.0, 40.0, "Flip");  // New flip board button

        let game_player = ChessGamePlayer::new(board.clone());

        let mut state = GameState {
            board,
            images,
            game_player,
            prev_button,
            next_button,
            reset_button,
            end_button,
            flip_button,
            board_flipped: false,
            game_info: "No game loaded".to_string(),
        };

        state.load_pgn_string(SAMPLE_PGN);

        Ok(state)
    }

    pub fn flip_board(&mut self) {
        self.board_flipped = !self.board_flipped;
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
        self.board = self.game_player.board.clone();
    }

    pub fn go_to_end(&mut self) {
        self.reset_position();

        let total_moves = self.game_player.get_total_moves();
        for _ in 0..total_moves {
            self.game_player.next_move();
        }

        self.board = self.game_player.board.clone();
    }

    pub fn prev_move(&mut self) {
        if self.game_player.previous_move() {
            self.board = self.game_player.board.clone();
        }
    }

    pub fn next_move(&mut self) {
        if self.game_player.next_move() {
            self.board = self.game_player.board.clone();
        }
    }

    pub fn draw_board(&self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::BLACK);

        let light_color = Color::from_rgba(240, 217, 181, 255);
        let dark_color = Color::from_rgba(181, 136, 99, 255);

        let start_x = 100.0;
        let start_y = 100.0;
        let grid_size = self.board.grid_size;

        for row in 0..8 {
            for col in 0..8 {
                let (display_row, display_col) = if self.board_flipped {
                    (7 - row, 7 - col)
                } else {
                    (row, col)
                };

                let color = if (row + col) % 2 == 0 {
                    light_color
                } else {
                    dark_color
                };

                let square = Mesh::new_rectangle(
                    ctx,
                    DrawMode::fill(),
                    Rect::new(
                        start_x + (display_col as f32 * grid_size),
                        start_y + (display_row as f32 * grid_size),
                        grid_size,
                        grid_size
                    ),
                    color,
                )?;

                canvas.draw(&square, DrawParam::default());

                if self.board.grid[row][col].piece_type != PieceType::None {
                    let piece_name = &self.board.grid[row][col].filename;

                    if let Some(image) = self.images.get(piece_name) {
                        let img_width = image.width() as f32;
                        let img_height = image.height() as f32;

                        let scale_factor = (grid_size / img_width).min(grid_size / img_height);

                        let x_offset = (grid_size - (img_width * scale_factor)) / 2.0;
                        let y_offset = (grid_size - (img_height * scale_factor)) / 2.0;

                        canvas.draw(
                            image,
                            DrawParam::default()
                                .dest([
                                    start_x + (display_col as f32 * grid_size) + x_offset,
                                    start_y + (display_row as f32 * grid_size) + y_offset
                                ])
                                .scale([scale_factor, scale_factor])
                        );
                    }
                }
            }
        }

        self.draw_board_labels(&mut canvas, ctx, start_x, start_y, grid_size)?;

        self.draw_button(&mut canvas, ctx, &self.prev_button)?;
        self.draw_button(&mut canvas, ctx, &self.next_button)?;
        self.draw_button(&mut canvas, ctx, &self.reset_button)?;
        self.draw_button(&mut canvas, ctx, &self.end_button)?;
        self.draw_button(&mut canvas, ctx, &self.flip_button)?;

        let mut info_text = Text::new(TextFragment::from(format!("Game: {}", self.game_info)));
        canvas.draw(&info_text, DrawParam::default().dest([100.0, 720.0]));

        let current_move = self.game_player.get_current_move();
        let total_moves = self.game_player.get_total_moves();
        let move_text = format!("Move: {}/{}", current_move, total_moves);

        info_text = Text::new(TextFragment::from(move_text));
        canvas.draw(&info_text, DrawParam::default().dest([100.0, 750.0]));

        canvas.finish(ctx)?;
        Ok(())
    }

    fn draw_board_labels(&self, canvas: &mut Canvas, ctx: &mut Context, start_x: f32, start_y: f32, grid_size: f32) -> GameResult {
        let files = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
        let ranks = ['1', '2', '3', '4', '5', '6', '7', '8'];
        let label_color = Color::from_rgba(200, 200, 200, 255);

        for i in 0..8 {
            let file_idx = if self.board_flipped { 7 - i } else { i };
            let file_label = Text::new(TextFragment::from(files[file_idx].to_string()).color(label_color));

            let x_pos = start_x + (i as f32 * grid_size) + grid_size/2.0 - 5.0;
            let y_pos = start_y + (8.0 * grid_size) + 10.0;

            canvas.draw(&file_label, DrawParam::default().dest([x_pos, y_pos]));
        }

        for i in 0..8 {
            let rank_idx = if self.board_flipped { i } else { 7 - i };
            let rank_label = Text::new(TextFragment::from(ranks[rank_idx].to_string()).color(label_color));

            let x_pos = start_x - 15.0;
            let y_pos = start_y + (i as f32 * grid_size) + grid_size/2.0 - 5.0;

            canvas.draw(&rank_label, DrawParam::default().dest([x_pos, y_pos]));
        }

        Ok(())
    }

    fn draw_button(&self, canvas: &mut Canvas, ctx: &mut Context, button: &Button) -> GameResult {
        let button_color = if button.pressed {
            Color::from_rgba(100, 100, 100, 255)
        } else {
            Color::from_rgba(125, 125, 125, 255)
        };

        let button_mesh = Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            button.rect,
            button_color,
        )?;

        canvas.draw(&button_mesh, DrawParam::default());

        let button_text = Text::new(TextFragment::from(button.text.clone()));
        let text_width = button_text.dimensions(ctx).unwrap().w;
        let text_x = button.rect.x + (button.rect.w - text_width) / 2.0;
        let text_y = button.rect.y + 10.0;

        canvas.draw(&button_text, DrawParam::default().dest([text_x, text_y]));

        Ok(())
    }
}

impl EventHandler for GameState {
    fn update(&mut self, _: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        Self::draw_board(self, ctx)
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
    event::run(ctx, event_loop, state)
}