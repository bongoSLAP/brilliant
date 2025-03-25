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
pub(crate) struct ChessBoard {
    pub(crate) grid: Vec<Vec<Piece>>,
    pub(crate) grid_size: f32,
}

impl ChessBoard {
    pub fn new(grid_size: f32) -> ChessBoard {
        let mut grid: Vec<Vec<Piece>> = Vec::new();

        grid.push(Self::get_back_rank(Colour::Black));
        grid.push(Self::get_pawn_rank(Colour::Black));

        for _ in 0..4 {
            grid.push(Self::get_empty_rank());
        }

        grid.push(Self::get_pawn_rank(Colour::White));
        grid.push(Self::get_back_rank(Colour::White));

        ChessBoard {
            grid,
            grid_size,
        }
    }

    pub fn get_back_rank(colour: Colour) -> Vec<Piece> {
        vec![
            Piece::new(PieceType::Rook, colour.clone()),
            Piece::new(PieceType::Knight, colour.clone()),
            Piece::new(PieceType::Bishop, colour.clone()),
            Piece::new(PieceType::Queen, colour.clone()),
            Piece::new(PieceType::King, colour.clone()),
            Piece::new(PieceType::Bishop, colour.clone()),
            Piece::new(PieceType::Knight, colour.clone()),
            Piece::new(PieceType::Rook, colour.clone()),
        ]
    }

    pub fn get_pawn_rank(colour: Colour) -> Vec<Piece> {
        vec![Piece::new(PieceType::Pawn, colour.clone()); 8]
    }

    pub fn get_empty_rank() -> Vec<Piece> {
        vec![Piece::new(PieceType::None, Colour::None); 8]
    }
}
