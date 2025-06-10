use shakmaty::{CastlingMode, Chess, Position};
use pgn_reader::{BufferedReader, RawHeader, SanPlus, Visitor};
use shakmaty::fen::Fen;

pub(crate) struct FenVisitor {
    pos: Chess,
    final_fen: Option<String>,
    move_count: i32,
    target_move: Option<i32>,
}

impl FenVisitor {
    pub(crate) fn new() -> Self {
        FenVisitor {
            pos: Chess::default(),
            final_fen: None,
            move_count: -1,
            target_move: None,
        }
    }

    pub(crate) fn with_target_move(target_move: usize) -> Self {
        let mut visitor = Self::new();
        visitor.target_move = Some(target_move.try_into().unwrap());
        visitor
    }
}

impl Visitor for FenVisitor {
    type Result = Option<String>;

    fn begin_game(&mut self) {
        self.pos = Chess::default();
        self.final_fen = None;
        self.move_count = 0;
    }

    fn header(&mut self, key: &[u8], value: RawHeader<'_>) {
        if key == b"FEN" {
            if let Ok(value_str) = value.decode_utf8() {
                if let Ok(fen) = value_str.parse::<Fen>() {
                    if let Ok(pos) = fen.into_position(CastlingMode::Standard) {
                        self.pos = pos;
                    }
                }
            }
        }
    }

    fn san(&mut self, san_plus: SanPlus) {
        if let Ok(m) = san_plus.san.to_move(&self.pos) {
            self.move_count += 1;

            self.pos.play_unchecked(&m);

            if let Some(target) = self.target_move {
                if self.move_count == target {
                    self.final_fen = Some(Fen::from_position(self.pos.clone(), shakmaty::EnPassantMode::Legal).to_string());
                }
            }
        }
    }

    fn end_game(&mut self) -> Self::Result {
        if self.final_fen.is_none() {
            self.final_fen = Some(Fen::from_position(self.pos.clone(), shakmaty::EnPassantMode::Legal).to_string());
        }

        self.final_fen.clone()
    }
}

pub fn pgn_to_fen_at_move(pgn: &str, move_number: usize) -> Option<String> {
    let mut reader = BufferedReader::new_cursor(pgn.as_bytes());
    println!("move number: {}", move_number);
    let mut visitor = FenVisitor::with_target_move(move_number);

    if reader.read_game(&mut visitor).is_ok() {
        visitor.end_game()
    }
    else {
        None
    }
}