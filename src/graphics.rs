use crate::board::{ChessBoard, PieceType};
use ggez::{graphics, Context, GameResult};
use ggez::graphics::{Color, DrawMode, DrawParam, Image, Mesh, Rect, Canvas, Text, TextFragment, Drawable, MeshBuilder};
use std::collections::HashMap;
use ggez::mint::Point2;

pub const BLACK_KING: &[u8] = include_bytes!("../resources/black-king.png");
pub const BLACK_QUEEN: &[u8] = include_bytes!("../resources/black-queen.png");
pub const BLACK_ROOK: &[u8] = include_bytes!("../resources/black-rook.png");
pub const BLACK_BISHOP: &[u8] = include_bytes!("../resources/black-bishop.png");
pub const BLACK_KNIGHT: &[u8] = include_bytes!("../resources/black-knight.png");
pub const BLACK_PAWN: &[u8] = include_bytes!("../resources/black-pawn.png");
pub const WHITE_KING: &[u8] = include_bytes!("../resources/white-king.png");
pub const WHITE_QUEEN: &[u8] = include_bytes!("../resources/white-queen.png");
pub const WHITE_ROOK: &[u8] = include_bytes!("../resources/white-rook.png");
pub const WHITE_BISHOP: &[u8] = include_bytes!("../resources/white-bishop.png");
pub const WHITE_KNIGHT: &[u8] = include_bytes!("../resources/white-knight.png");
pub const WHITE_PAWN: &[u8] = include_bytes!("../resources/white-pawn.png");

pub const START_X: f32 = 100.0;
pub const START_Y: f32 = 100.0;

pub struct Button {
    pub rect: Rect,
    pub text: String,
    pub pressed: bool,
}

impl Button {
    pub fn new(x: f32, y: f32, width: f32, height: f32, text: &str) -> Self {
        Button {
            rect: Rect::new(x, y, width, height),
            text: text.to_string(),
            pressed: false,
        }
    }

    pub fn contains_point(&self, point: [f32; 2]) -> bool {
        self.rect.contains(point)
    }
}

pub fn load_images(ctx: &mut Context) -> GameResult<HashMap<String, Image>> {
    let mut images = HashMap::new();

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

    Ok(images)
}

pub fn draw_board_labels(
    canvas: &mut Canvas,
    ctx: &mut Context,
    grid_size: f32,
    board_flipped: bool
) -> GameResult {
    let files = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
    let ranks = ['1', '2', '3', '4', '5', '6', '7', '8'];

    for i in 0..8 {
        let file_idx = if board_flipped { 7 - i } else { i };
        let file_label = Text::new(TextFragment::from(files[file_idx].to_string()).color(Color::from_rgba(200, 200, 200, 255)));

        let x_pos = START_X + (i as f32 * grid_size) + grid_size/2.0 - 5.0;
        let y_pos = START_Y + (8.0 * grid_size) + 10.0;

        canvas.draw(&file_label, DrawParam::default().dest([x_pos, y_pos]));
    }

    for i in 0..8 {
        let rank_idx = if board_flipped { i } else { 7 - i };
        let rank_label = Text::new(TextFragment::from(ranks[rank_idx].to_string()).color(Color::from_rgba(200, 200, 200, 255)));

        let x_pos = START_X - 15.0;
        let y_pos = START_Y + (i as f32 * grid_size) + grid_size/2.0 - 5.0;

        canvas.draw(&rank_label, DrawParam::default().dest([x_pos, y_pos]));
    }

    Ok(())
}

pub fn draw_button(canvas: &mut Canvas, ctx: &mut Context, button: &Button) -> GameResult {
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

pub fn draw_info_text(
    canvas: &mut Canvas,
    game_info: &str,
    current_move: usize,
    total_moves: usize
) {
    let info_text = Text::new(TextFragment::from(format!("Game: {}", game_info)));
    canvas.draw(&info_text, DrawParam::default().dest([100.0, 720.0]));

    let move_text = format!("Move: {}/{}", current_move, total_moves);
    let move_info = Text::new(TextFragment::from(move_text));
    canvas.draw(&move_info, DrawParam::default().dest([100.0, 750.0]));
}

pub fn draw_arrow(
    ctx: &mut Context,
    start: Point2<f32>,
    end: Point2<f32>,
    color: Color,
) -> GameResult {
    let mut canvas = Canvas::from_frame(ctx, Color::BLACK);

    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let angle = dy.atan2(dx);

    let shaft = Mesh::new_line(
        ctx,
        &[start, end],
        30.0,
        color
    )?;

    canvas.draw(&shaft, DrawParam::default());

    let head_angle = 2.5;
    let head_length = 30.0;

    let p1 = Point2 {
        x: end.x - head_length * (angle + head_angle).cos(),
        y: end.y - head_length * (angle + head_angle).sin(),
    };

    let p2 = Point2 {
        x: end.x - head_length * (angle - head_angle).cos(),
        y: end.y - head_length * (angle - head_angle).sin(),
    };

    let head = Mesh::new_polygon(
        ctx,
        DrawMode::fill(),
        &[end, p1, p2],
        color
    )?;

    canvas.draw(&head, DrawParam::default());
    canvas.finish(ctx)?;

    Ok(())
}

pub fn draw_ui(
    ctx: &mut Context,
    board: &ChessBoard,
    images: &HashMap<String, Image>,
    buttons: &[&Button],
    game_info: &str,
    current_move: usize,
    total_moves: usize,
    board_flipped: bool
) -> GameResult {
    let mut canvas = Canvas::from_frame(ctx, Color::BLACK);
    let grid_size = board.grid_size;

    for row in 0..8 {
        for col in 0..8 {
            let (display_row, display_col) = if board_flipped {
                (7 - row, 7 - col)
            } else {
                (row, col)
            };

            let color = if (row + col) % 2 == 0 {
                Color::from_rgba(240, 217, 181, 255)
            } else {
                Color::from_rgba(181, 136, 99, 255)
            };

            let square = Mesh::new_rectangle(
                ctx,
                DrawMode::fill(),
                Rect::new(
                    START_X + (display_col as f32 * grid_size),
                    START_Y + (display_row as f32 * grid_size),
                    grid_size,
                    grid_size
                ),
                color,
            )?;

            canvas.draw(&square, DrawParam::default());

            if board.grid[row][col].piece_type != PieceType::None {
                let piece_name = &board.grid[row][col].filename;

                if let Some(image) = images.get(piece_name) {
                    let img_width = image.width() as f32;
                    let img_height = image.height() as f32;

                    let scale_factor = (grid_size / img_width).min(grid_size / img_height);

                    let x_offset = (grid_size - (img_width * scale_factor)) / 2.0;
                    let y_offset = (grid_size - (img_height * scale_factor)) / 2.0;

                    canvas.draw(
                        image,
                        DrawParam::default()
                            .dest([
                                START_X + (display_col as f32 * grid_size) + x_offset,
                                START_Y + (display_row as f32 * grid_size) + y_offset
                            ])
                            .scale([scale_factor, scale_factor])
                    );
                }
            }
        }
    }

    draw_board_labels(&mut canvas, ctx, grid_size, board_flipped)?;

    for button in buttons {
        draw_button(&mut canvas, ctx, button)?;
    }

    draw_info_text(&mut canvas, game_info, current_move, total_moves);
    canvas.finish(ctx)?;
    Ok(())
}