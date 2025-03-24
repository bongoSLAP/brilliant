#[derive(Clone)]
enum PieceType {
    None,
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[derive(Clone)]
enum Colour {
    None,
    White,
    Black,
}

#[derive(Clone)]
struct Piece {
    piece_type: PieceType,
    colour: Colour,
}

impl Piece {
    fn new(piece_type: PieceType, colour: Colour) -> Self {
        Piece { piece_type, colour }
    }
}

struct ChessBoard {
    board: Vec<Vec<Piece>>,
    grid_size: i32,
}

impl ChessBoard {
    pub fn new(&self) {
        let mut grid: Vec<Vec<Piece>> = Vec::new();

        grid.push(Self::get_back_rank(Colour::Black));
        grid.push(Self::get_pawn_rank(Colour::Black));

        for _ in 0..4 {
            grid.push(Self::get_empty_rank());
        }

        grid.push(Self::get_pawn_rank(Colour::White));
        grid.push(Self::get_back_rank(Colour::White));
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
