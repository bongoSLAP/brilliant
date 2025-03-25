use std::io::{BufReader, Cursor};
use pgn_reader::{BufferedReader, Visitor, Skip, RawHeader, SanPlus};
use shakmaty::{Square, Chess, Position, Move};

use crate::board::{ChessBoard, PieceType, Piece, Colour};

pub struct ChessGamePlayer {
    pub board: ChessBoard,
    moves: Vec<Move>,
    position: Chess,
    current_move: usize,
    headers: Vec<(String, String)>,
}

impl ChessGamePlayer {
    pub fn new(board: ChessBoard) -> Self {
        ChessGamePlayer {
            board,
            moves: Vec::new(),
            position: Chess::default(),
            current_move: 0,
            headers: Vec::new(),
        }
    }

    pub fn load_pgn(&mut self, pgn: &str) -> bool {
        let cursor = Cursor::new(pgn);
        let reader = BufReader::new(cursor);
        let mut buffered_reader = BufferedReader::new(reader);

        let mut visitor = PgnVisitor::new();

        match buffered_reader.read_game(&mut visitor) {
            Ok(Some(())) => {
                self.moves = visitor.moves;
                self.headers = visitor.headers;

                self.reset();
                true
            },
            Ok(None) => {
                eprintln!("No game found in PGN");
                false
            },
            Err(err) => {
                eprintln!("Error reading PGN: {}", err);
                false
            }
        }
    }

    pub fn reset(&mut self) {
        self.position = Chess::default();
        self.current_move = 0;
        self.board = ChessBoard::new(self.board.grid_size);
    }

    pub fn next_move(&mut self) -> bool {
        if self.current_move >= self.moves.len() {
            return false;
        }

        let mv = &self.moves[self.current_move].clone();
        self.position.play_unchecked(mv);
        self.apply_move_to_board(mv);
        self.current_move += 1;
        true
    }

    pub fn previous_move(&mut self) -> bool {
        if self.current_move == 0 {
            return false;
        }

        self.reset();

        let target = self.current_move - 1;
        for i in 0..target {
            let mv = &self.moves[i].clone();
            self.position.play_unchecked(mv);
            self.apply_move_to_board(mv);
        }

        self.current_move = target;
        true
    }

    fn apply_move_to_board(&mut self, mv: &Move) {
        match mv {
            Move::Normal { from, to, promotion, .. } => {
                let from_coord = square_to_board_coord(*from);
                let to_coord = square_to_board_coord(*to);

                self.move_piece(from_coord, to_coord);

                if let Some(role) = promotion {
                    let piece_type = match role {
                        shakmaty::Role::Queen => PieceType::Queen,
                        shakmaty::Role::Rook => PieceType::Rook,
                        shakmaty::Role::Bishop => PieceType::Bishop,
                        shakmaty::Role::Knight => PieceType::Knight,
                        _ => panic!("Invalid promotion piece"),
                    };

                    self.promote_piece(to_coord, piece_type);
                }
            },
            Move::Castle { king, rook, .. } => {
                let is_kingside = rook.file() as usize > king.file() as usize;

                if is_kingside {
                    let rank = king.rank().char() as usize - '1' as usize;

                    self.move_piece(
                        (4, rank),
                        (6, rank)
                    );

                    self.move_piece(
                        (7, rank),
                        (5, rank)
                    );
                } else {
                    let rank = king.rank().char() as usize - '1' as usize;

                    self.move_piece(
                        (4, rank),
                        (2, rank)
                    );

                    self.move_piece(
                        (0, rank),
                        (3, rank)
                    );
                }
            },
            Move::EnPassant { from, to, .. } => {
                let from_coord = square_to_board_coord(*from);
                let to_coord = square_to_board_coord(*to);

                self.move_piece(from_coord, to_coord);

                let captured_rank = if from.rank().char() as usize - '1' as usize > 3 {
                    to.rank().char() as usize - '1' as usize + 1
                } else {
                    to.rank().char() as usize - '1' as usize - 1
                };

                self.remove_piece((to.file() as usize, captured_rank));
            },
            _ => panic!("Unexpected move type"),
        }
    }

    fn move_piece(&mut self, from: (usize, usize), to: (usize, usize)) {
        let from_row = 7 - from.1;
        let from_col = from.0;

        let to_row = 7 - to.1;
        let to_col = to.0;

        self.board.grid[to_row][to_col] = self.board.grid[from_row][from_col].clone();
        self.board.grid[from_row][from_col] = Piece::new(PieceType::None, Colour::None);
    }

    fn promote_piece(&mut self, coord: (usize, usize), piece_type: PieceType) {
        let row = 7 - coord.1;
        let col = coord.0;

        let is_white_turn = self.current_move % 2 == 0;
        let color = if is_white_turn { Colour::White } else { Colour::Black };

        self.board.grid[row][col] = Piece::new(piece_type, color);
    }

    fn remove_piece(&mut self, coord: (usize, usize)) {
        let row = 7 - coord.1;
        let col = coord.0;

        self.board.grid[row][col] = Piece::new(PieceType::None, Colour::None);
    }

    pub fn get_current_move(&self) -> usize {
        self.current_move
    }

    pub fn get_total_moves(&self) -> usize {
        self.moves.len()
    }

    pub fn get_headers(&self) -> &[(String, String)] {
        &self.headers
    }
}

fn square_to_board_coord(square: Square) -> (usize, usize) {
    let file = square.file() as usize;
    let rank = square.rank().char() as usize - '1' as usize;
    (file, rank)
}

struct PgnVisitor {
    position: Chess,
    moves: Vec<Move>,
    headers: Vec<(String, String)>,
}

impl PgnVisitor {
    fn new() -> Self {
        PgnVisitor {
            position: Chess::default(),
            moves: Vec::new(),
            headers: Vec::new(),
        }
    }
}

impl Visitor for PgnVisitor {
    type Result = ();

    fn begin_game(&mut self) {
        self.position = Chess::default();
        self.moves.clear();
        self.headers.clear();
    }

    fn header(&mut self, key: &[u8], value: RawHeader<'_>) {
        if let (Ok(key_str), Ok(value_str)) = (
            std::str::from_utf8(key),
            value.decode_utf8()
        ) {
            self.headers.push((key_str.to_string(), value_str.to_string()));
        }
    }

    fn san(&mut self, san_plus: SanPlus) {
        let san = san_plus.san;

        if let Ok(mv) = san.to_move(&self.position) {
            self.moves.push(mv.clone());
            self.position.play_unchecked(&mv);
        }
    }

    fn begin_variation(&mut self) -> Skip {
        Skip(true)
    }

    fn end_game(&mut self) -> Self::Result {
        ()
    }
}
