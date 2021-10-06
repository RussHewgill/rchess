
use crate::types::*;

pub use self::D::*;

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub enum D {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

#[derive(Debug,Eq,Hash,PartialEq,PartialOrd,Clone,Copy)]
pub struct Coord(pub u8, pub u8);

impl std::convert::From<u32> for Coord {
    fn from(sq: u32) -> Self {
        // assert!(sq < 64);
        BitBoard::index_bit(sq)
    }
}

impl std::convert::From<u64> for Coord {
    fn from(sq: u64) -> Self {
        // assert!(sq < 64);
        BitBoard::index_bit(sq)
    }
}

impl std::convert::From<Coord> for u32 {
    fn from(c: Coord) -> Self {
        BitBoard::index_square(c)
    }
}

impl D {

    // pub fn shift(&self) -> i8 {
    //     match *self {
    //         D::N  => 8,
    //         D::NE => 9,
    //         D::E  => 1,
    //         D::SE => -7,
    //         D::S  => -8,
    //         D::SW => -9,
    //         D::W  => -1,
    //         D::NW => 7,
    //     }
    // }

    pub fn shift(&self, x: u32) -> u32 {
        match *self {
            D::N  => x + 8,
            D::NE => x + 9,
            D::E  => x + 1,
            D::SE => x - 7,
            D::S  => x - 8,
            D::SW => x - 9,
            D::W  => x - 1,
            D::NW => x + 7,
        }
    }

}

impl std::ops::Not for D {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            N  => S,
            NE => SW,
            E  => W,
            SE => NW,
            S  => N,
            SW => NE,
            W  => E,
            NW => SE,
        }
    }
}

fn test_directions() {

    let b = BitBoard::new(&vec![Coord(1,1)]);

    assert_eq!(b.shift(D::E), BitBoard::new(&vec![Coord(2,1)]));
    assert_eq!(b.shift(D::W), BitBoard::new(&vec![Coord(0,1)]));
    assert_eq!(b.shift(D::N), BitBoard::new(&vec![Coord(1,2)]));
    assert_eq!(b.shift(D::S), BitBoard::new(&vec![Coord(1,0)]));
    assert_eq!(b.shift(D::NE), BitBoard::new(&vec![Coord(2,2)]));
    assert_eq!(b.shift(D::NW), BitBoard::new(&vec![Coord(0,2)]));
    assert_eq!(b.shift(D::SE), BitBoard::new(&vec![Coord(2,0)]));
    assert_eq!(b.shift(D::SW), BitBoard::new(&vec![Coord(0,0)]));

    let b = BitBoard::new(&vec![Coord(0,0)]);
    assert_eq!(b.shift(D::W), BitBoard::new(&vec![]));
    let b = BitBoard::new(&vec![Coord(7,0)]);
    assert_eq!(b.shift(D::E), BitBoard::new(&vec![]));
    let b = BitBoard::new(&vec![Coord(0,0)]);
    assert_eq!(b.shift(D::S), BitBoard::new(&vec![]));
    let b = BitBoard::new(&vec![Coord(0,7)]);
    assert_eq!(b.shift(D::N), BitBoard::new(&vec![]));

}
