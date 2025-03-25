mod board;

use board::{ChessBoard, PieceType};
use ggez::{Context, GameResult, ContextBuilder, event};
use ggez::graphics::{self, Color, DrawMode, DrawParam, Image, Mesh, Rect};
use ggez::event::{EventHandler};

// Include the image binary data directly in the executable
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

struct GameState {
    board: ChessBoard,
    images: std::collections::HashMap<String, Image>
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

        Ok(GameState { board, images })
    }

    pub fn draw_board(&self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, Color::BLACK);

        let light_color = Color::from_rgba(240, 217, 181, 255);
        let dark_color = Color::from_rgba(181, 136, 99, 255);

        let start_x = 100.0;
        let start_y = 100.0;
        let grid_size = self.board.grid_size;

        for row in 0..self.board.grid.len() {
            for col in 0..self.board.grid[row].len() {
                let color = if (row + col) % 2 == 0 {
                    light_color
                } else {
                    dark_color
                };

                let square = Mesh::new_rectangle(
                    ctx,
                    DrawMode::fill(),
                    Rect::new(
                        start_x + (col as f32 * grid_size),
                        start_y + (row as f32 * grid_size),
                        grid_size,
                        grid_size
                    ),
                    color,
                )?;

                graphics::draw(ctx, &square, DrawParam::default())?;

                if self.board.grid[row][col].piece_type != PieceType::None {
                    let piece_name = &self.board.grid[row][col].filename;

                    if let Some(image) = self.images.get(piece_name) {
                        let img_width = image.width() as f32;
                        let img_height = image.height() as f32;

                        let scale_factor = (grid_size / img_width).min(grid_size / img_height);

                        let x_offset = (grid_size - (img_width * scale_factor)) / 2.0;
                        let y_offset = (grid_size - (img_height * scale_factor)) / 2.0;

                        graphics::draw(
                            ctx,
                            image,
                            DrawParam::default()
                                .dest([
                                    start_x + (col as f32 * grid_size) + x_offset,
                                    start_y + (row as f32 * grid_size) + y_offset
                                ])
                                .scale([scale_factor, scale_factor])
                        )?;
                    }
                }
            }
        }

        graphics::present(ctx)?;
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
}

fn main() -> GameResult {
    let (mut ctx, event_loop) = ContextBuilder::new("Brilliant", "BongoSLAP")
        .window_mode(
            ggez::conf::WindowMode::default()
                .dimensions(800.0, 800.0)
        )
        .build()?;

    let state = GameState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}