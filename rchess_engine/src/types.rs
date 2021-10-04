
// use std::ops::{Index,IndexMut};

pub use self::Color::*;

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
pub struct BitBoard(pub u64);

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct Coord(pub u64, pub u64);

// impl Index<Coord> for BitBoard {
//     type Output = bool;
//     fn index(&self, c: Coord) -> &Self::Output {
//         let p: u64 = (c.0 + 8 * c.1).into();
//         let k = 1 << p;
//         let k = k & self.0;
//         // let k = self.0 >> p;
//         // &(k == 1)
//         // unimplemented!()
//     }
// }

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

impl BitBoard {

    pub fn empty() -> BitBoard {
        BitBoard(0)
    }
    pub fn new(cs: &[Coord]) -> BitBoard {
        let mut b = BitBoard::empty();
        for c in cs.iter() {
            b.flip(*c);
        }
        b
    }

    pub fn single(c: Coord) -> BitBoard {
        let mut b = BitBoard::empty();
        b.flip(c);
        b
    }

    pub fn get(&self, c: Coord) -> bool {
        let p: u64 = (c.0 + 8 * c.1).into();
        let k = (self.0 >> p) & 1;
        k == 1
    }

    pub fn flip(&mut self, c: Coord) {
        let p: u64 = (c.0 + 8 * c.1).into();
        let k = 1 << p;
        self.0 |= k;
    }

}

impl Game {
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
