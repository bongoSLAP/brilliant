use ggez::mint::Point2;

#[derive(PartialEq, Clone)]
pub(crate) enum PieceType {
    None,
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[derive(PartialEq, Clone)]
pub(crate) enum Colour {
    None,
    White,
    Black,
}

#[derive(Clone)]
pub struct Piece {
    pub(crate) piece_type: PieceType,
    colour: Colour,
    pub(crate) filename: String,
}

impl Piece {
    pub(crate) fn new(piece_type: PieceType, colour: Colour) -> Self {
        let mut filename = String::new();

        if piece_type != PieceType::None {
            filename = Self::get_filename(piece_type.clone(), colour.clone());
        }

        Piece { piece_type, colour, filename}
    }

    fn get_filename(piece_type: PieceType, colour: Colour) -> String {
        let colour_str = match colour {
            Colour::White => "white",
            Colour::Black => "black",
            Colour::None => panic!()
        };

        let type_str = match piece_type {
            PieceType::King => "king",
            PieceType::Queen => "queen",
            PieceType::Rook => "rook",
            PieceType::Bishop => "bishop",
            PieceType::Knight => "knight",
            PieceType::Pawn => "pawn",
            PieceType::None => panic!()
        };

        format!("{}-{}", colour_str, type_str)
    }
}

#[derive(Clone)]
pub struct BoardSquare {
    pub piece: Piece,
    pub display_coords: Point2<f32>,
}

impl BoardSquare {
    pub fn new(piece: Piece, grid_size: f32, row_index: usize, col_index: usize) -> BoardSquare {
        let display_x = (col_index as f32) * grid_size;
        let display_y = (row_index as f32) * grid_size;
        let display_coords = Point2 { x: display_x, y: display_y };

        BoardSquare {
            piece,
            display_coords
        }
    }
}

#[derive(Clone)]
pub(crate) struct ChessBoard {
    pub(crate) grid: Vec<Vec<BoardSquare>>,
    pub(crate) grid_size: f32,
}

impl ChessBoard {
    pub fn new(grid_size: f32) -> ChessBoard {
        let mut grid: Vec<Vec<BoardSquare>> = Vec::new();

        grid.push(Self::get_back_rank(grid_size, Colour::Black));
        grid.push(Self::get_pawn_rank(grid_size, Colour::Black));

        for i in 0..4 {
            grid.push(Self::get_empty_rank(grid_size, i));
        }

        grid.push(Self::get_pawn_rank(grid_size, Colour::White));
        grid.push(Self::get_back_rank(grid_size, Colour::White));

        ChessBoard {
            grid,
            grid_size,
        }
    }

    pub fn get_back_rank(grid_size: f32, colour: Colour) -> Vec<BoardSquare> {
        let row_index = match colour {
            Colour::White => 0,
            Colour::Black => 7,
            Colour::None => panic!()
        };

        vec![
            BoardSquare::new(Piece::new(PieceType::Rook, colour.clone()), grid_size, row_index, 0),
            BoardSquare::new(Piece::new(PieceType::Knight, colour.clone()), grid_size, row_index, 1),
            BoardSquare::new(Piece::new(PieceType::Bishop, colour.clone()), grid_size, row_index, 2),
            BoardSquare::new(Piece::new(PieceType::Queen, colour.clone()), grid_size, row_index, 3),
            BoardSquare::new(Piece::new(PieceType::King, colour.clone()), grid_size, row_index, 4),
            BoardSquare::new(Piece::new(PieceType::Bishop, colour.clone()), grid_size, row_index, 5),
            BoardSquare::new(Piece::new(PieceType::Knight, colour.clone()), grid_size, row_index, 6),
            BoardSquare::new(Piece::new(PieceType::Rook, colour.clone()), grid_size, row_index, 7),
        ]
    }

    pub fn get_pawn_rank(grid_size: f32, colour: Colour) -> Vec<BoardSquare> {
        let row_index = match colour {
            Colour::White => 1,
            Colour::Black => 6,
            Colour::None => panic!()
        };

        (0..8).map(|i| BoardSquare::new(Piece::new(PieceType::Pawn, colour.clone()), grid_size, row_index, i))
            .collect()
    }

    pub fn get_empty_rank(grid_size: f32, row_index: usize) -> Vec<BoardSquare> {
        (0..8).map(|i| BoardSquare::new(Piece::new(PieceType::None, Colour::None), grid_size, row_index, i))
            .collect()
    }
}
