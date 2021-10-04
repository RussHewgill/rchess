
pub use crate::bitboard::*;
pub use crate::coords::*;

pub use self::{Color::*,Piece::*};

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub enum Color {
    White,
    Black,
}

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub enum Piece {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct Game {
    side_to_move: Color,
    // TODO: castling
    // TODO: en passant
    pawns:        BitBoard,
    rooks:        BitBoard,
    knights:      BitBoard,
    bishops:      BitBoard,
    queens:       BitBoard,
    kings:        BitBoard,
    white:        BitBoard,
    black:        BitBoard,
}

impl Game {

    pub fn get_color(&self, c: Color) -> BitBoard {
        match c {
            White => self.white,
            Black => self.black,
        }
    }

    pub fn get_piece(&self, piece: Piece) -> BitBoard {
        match piece {
            Pawn   => self.pawns,
            Rook   => self.rooks,
            Knight => self.knights,
            Bishop => self.bishops,
            Queen  => self.queens,
            King   => self.kings,
        }
    }

    pub fn get(&self, piece: Piece, c: Color) -> BitBoard {
        self.get_color(c) & self.get_piece(piece)
    }

    pub fn new() -> Game {

        let pawns   = BitBoard::empty();
        let rooks   = BitBoard::empty();
        let knights = BitBoard::empty();
        let bishops = BitBoard::empty();
        let queens  = BitBoard::empty();
        // let queens  = BitBoard::new(&vec![Coord(3,0),Coord(3,7)]);
        let kings   = BitBoard::new(&vec![Coord(4,0),Coord(4,7)]);

        let white = BitBoard::empty();
        let black = BitBoard::empty();

        Game {
            side_to_move: White,
            pawns,
            rooks,
            knights,
            bishops,
            queens,
            kings,
            white,
            black,
        }
    }
}
