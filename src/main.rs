mod board;

use board::{ChessBoard, PieceType};
use ggez::{Context, GameResult, ContextBuilder, event};
use ggez::graphics::{self, Color, DrawMode, DrawParam, Image, Mesh, Rect};
use ggez::event::{EventHandler};

struct GameState {
    board: ChessBoard
}

impl GameState {
    fn new(_ctx: &mut Context) -> GameResult<GameState> {
        let grid_size = 72.0;
        let mut board = ChessBoard::new(grid_size);
        Ok(GameState { board })
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

                if self.board.grid[row][col].piece_type == PieceType::None {
                    continue;
                }

                let image = Image::new(ctx, format!("../{}.png", self.board.grid[row][col].filename))?;

                graphics::draw(
                    ctx,
                    &image,
                    DrawParam::default()
                        .dest([
                            start_x + (col as f32 * grid_size),
                            start_y + (row as f32 * grid_size)
                        ])
                )?;
            }
        }

        graphics::present(ctx)?;
        Ok(())
    }
}

impl EventHandler for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
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