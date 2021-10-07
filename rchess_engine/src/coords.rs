
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

#[derive(Eq,Hash,PartialEq,PartialOrd,Clone,Copy)]
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

    // pub fn shift_sq(&self, x: u32) -> Option<u32> {
    //     match *self {
    //         // D::N  => x + 8,
    //         // D::NE => x + 9,
    //         // D::E  => x + 1,
    //         // D::SE => x - 7,
    //         // D::S  => x - 8,
    //         // D::SW => x - 9,
    //         // D::W  => x - 1,
    //         // D::NW => x + 7,
    //         D::SE => x.checked_sub(7),
    //         D::S  => x.checked_sub(8),
    //         D::SW => x.checked_sub(9),
    //         D::W  => x.checked_sub(1),
    //         D::N  => {
    //             let k = x + 8;
    //             if k > 63 { None } else { Some(k) }
    //         },
    //         D::NE  => {
    //             let k = x + 9;
    //             if k > 63 { None } else { Some(k) }
    //         },
    //         D::E  => {
    //             let k = x + 1;
    //             if k > 63 { None } else { Some(k) }
    //         },
    //         D::NW  => {
    //             let k = x + 7;
    //             if k > 63 { None } else { Some(k) }
    //         },
    //     }
    //     // panic!("D::shift")
    // }

    pub fn shift_coord(&self, Coord(x0,y0): Coord) -> Option<Coord> {
        match *self {
            N => {
                if y0 > 7 { None } else {
                    Some(Coord(x0,y0+1))
                }
            },
            NE => {
                if (y0 > 7) | (x0 > 7) { None } else {
                    Some(Coord(x0+1,y0+1))
                }
            },
            E => {
                if x0 > 7 { None } else {
                    Some(Coord(x0+1,y0))
                }
            },
            NW => {
                if (y0 > 7) | (x0 == 0) { None } else {
                    Some(Coord(x0-1,y0+1))
                }
            },
            S => {
                if y0 == 0 { None } else {
                    Some(Coord(x0,y0-1))
                }
            },
            SE => {
                if (y0 == 0) | (x0 > 7) { None } else {
                    Some(Coord(x0+1,y0-1))
                }
            },
            SW => {
                if (y0 == 0) | (x0 == 0) { None } else {
                    Some(Coord(x0-1,y0-1))
                }
            },
            W => {
                if x0 == 0 { None } else {
                    Some(Coord(x0-1,y0))
                }
            },
        }
        // let k = self.shift_sq(c.into())?;
        // Some(k.into())
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

impl std::fmt::Debug for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let letters: [char; 8] = ['A','B','C','D','E','F','G','H'];
        let r = letters[self.0 as usize];
        // f.write_str(&format!("Coord({}{})", r, self.1+1))?;
        f.write_str(&format!("{}{}", r, self.1+1))?;
        Ok(())
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
