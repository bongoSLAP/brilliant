mod board;

use ggez::{Context, GameResult, ContextBuilder, event};
use ggez::graphics::{self, Color, DrawMode, Mesh, Rect};
use ggez::event::{EventHandler};
struct GameState {
    // Game state variables go here
}

impl GameState {
    fn new(_ctx: &mut Context) -> GameResult<GameState> {
        Ok(GameState {
        })
    }

    pub fn draw_board(ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, Color::BLACK);

        let square = Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            Rect::new(100.0, 100.0, 50.0, 50.0),
            Color::from_rgba(240,217,181,255)
        )?;

        graphics::draw(ctx, &square, graphics::DrawParam::default())?;
        graphics::present(ctx)?;
        Ok(())
    }
}

impl EventHandler for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        Self::draw_board(ctx)
    }
}

fn main() -> GameResult {
    let (mut ctx, event_loop) = ContextBuilder::new("Brilliant", "BongoSLAP")
        .build()?;

    let state = GameState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}