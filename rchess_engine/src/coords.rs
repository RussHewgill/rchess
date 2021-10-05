
use crate::types::*;

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

impl D {
    pub fn shift(&self) -> i8 {
        match *self {
            D::N  => 8,
            D::NE => 9,
            D::E  => 1,
            D::SE => -7,
            D::S  => -8,
            D::SW => -9,
            D::W  => -1,
            D::NW => 7,
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
