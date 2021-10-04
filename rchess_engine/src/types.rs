
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
    pub side_to_move: Color,
    // TODO: castling
    // TODO: en passant
    pub pawns:        BitBoard,
    pub rooks:        BitBoard,
    pub knights:      BitBoard,
    pub bishops:      BitBoard,
    pub queens:       BitBoard,
    pub kings:        BitBoard,
    pub white:        BitBoard,
    pub black:        BitBoard,
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

        let mut white = BitBoard::empty();
        let mut black = BitBoard::empty();
        // for x in 0..8 {
        //     for y in 0..3 {
        //         white.flip_mut(Coord(x,y));
        //     }
        //     for y in 6..8 {
        //         black.flip_mut(Coord(x,y));
        //     }
        // }

        let k = (!0u8) as u64 | (((!0u8) as u64) << 8);
        white.0 |= k;
        black.0 |= k << (6 * 8);

        white &= pawns | rooks | knights | bishops | queens | kings;
        black &= pawns | rooks | knights | bishops | queens | kings;

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
