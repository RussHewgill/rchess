
use crate::types::*;
use crate::tables::*;
pub use self::impls::*;

use evmap_derive::ShallowCopy;

use serde::{Serialize,Deserialize};

// use packed_struct::prelude::*;

pub use self::D::*;

use std::str::FromStr;

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

// #[derive(Eq,Ord,PartialEq,PartialOrd,Hash,ShallowCopy,Clone,Copy)]
#[derive(Serialize,Deserialize,Eq,Ord,PartialEq,PartialOrd,Hash,ShallowCopy,Clone,Copy)]
// #[derive(Hash,Eq,PartialEq,Ord,PartialOrd,Clone,Copy)]
// pub struct Coord(pub u8, pub u8);
pub struct Coord(u8);

#[derive(Serialize,Deserialize,Eq,Ord,PartialEq,PartialOrd,Hash,ShallowCopy,Clone,Copy)]
pub enum Sq {
    A1 = 0, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

impl Sq {
    pub const fn to(&self) -> Coord {
        Coord(*self as u8)
    }
}

impl Coord {

    pub fn new_int<T: PrimInt + AsPrimitive<u8>>(sq: T) -> Self {
        // assert!(x < 64);
        // Self(x.into())
        let x: u8 = sq.as_();
        Coord(x)
    }

    pub fn new(file: u8, rank: u8) -> Self {
        assert!(file < 8);
        assert!(rank < 8);
        Self(file + 8 * rank)
    }

    pub const fn new_const(file: u8, rank: u8) -> Self {
        Self(file + 8 * rank)
    }

    pub fn inner(&self) -> u8 {
        self.0
    }

    pub fn to_rankfile(&self) -> (u8,u8) {
        (self.file(), self.rank())
    }

    // pub fn 

}

impl std::convert::From<&str> for Coord {
    fn from(sq: &str) -> Self {
        Coord::from_str(sq).unwrap()
    }
}

// pub type PackedCoords = Integer<u8, packed_bits::Bits::<6>>;

// #[derive(PackedStruct)]
// #[packed_struct(bit_numbering="msb0")]
// pub struct PCoord {
//     #[packed_field(bits="0", size_bits="6")]
//     sq: Integer<u8, packed_bits::Bits::<6>>,
// }

// impl PCoord {
//     pub fn new_sq<T: Into<PackedCoords>>(sq: T) -> Self {
//         Self { sq: sq.into() }
//     }
//     // pub fn new<T: Into<u8>>(x: T, y: T) -> Self {
//     //     // Self {
//     //     // }
//     // }
// }

impl<T> std::ops::Index<Coord> for [T; 64] {
    type Output = T;
    fn index(&self, c1: Coord) -> &Self::Output {
        let sq: usize = c1.into();
        &self[sq]
    }
}

impl<T> std::ops::IndexMut<Coord> for [T; 64] {
    fn index_mut(&mut self, c1: Coord) -> &mut Self::Output {
        let sq: usize = c1.into();
        &mut self[sq]
    }
}

impl Coord {

    pub fn file(self) -> u8 {
        // self.0
        self.0 & 7
    }
    pub fn rank(self) -> u8 {
        // self.1
        self.0 >> 3
    }

    pub fn mask_file(self) -> BitBoard {
        MASK_FILES[self.file() as usize]
    }
    pub fn mask_rank(self) -> BitBoard {
        MASK_RANKS[self.rank() as usize]
    }

    pub fn flip_diagonal_int<T>(x: T) -> T where
        T: num_traits::PrimInt + num_traits::WrappingMul,
    {
        x.wrapping_mul(&T::from(0x2080_0000).unwrap()) >> 26
    }

    pub fn flip_horizontal_int<T: num_traits::PrimInt>(x: T) -> T {
        x ^ T::from(0b000_111).unwrap()
    }

    pub fn flip_vertical_int<T: num_traits::PrimInt>(x: T) -> T {
        x ^ T::from(0b111_000).unwrap()
    }

    pub fn flip_diagonal(self) -> Self {
        let x = u32::from(self).wrapping_mul(0x2080_0000) >> 26;
        Coord::from(x)
    }

    pub fn flip_vertical(self) -> Self {
        Coord::new(self.file(), 7 - self.rank())
    }
    pub fn flip_horizontal(self) -> Self {
        Coord::new(7 - self.file(), self.rank())
    }
}

impl Coord {

    pub fn center_distance(&self) -> u8 {
        CENTERDIST[*self]
    }

    pub fn file_dist(&self, c1: Coord) -> u8 {
        (self.file() as i8 - c1.file() as i8).abs() as u8
    }

    pub fn rank_dist(&self, c1: Coord) -> u8 {
        (self.rank() as i8 - c1.rank() as i8).abs() as u8
    }

    pub fn square_dist(&self, c1: Coord) -> u8 {
        // let r = self.rank_dist(c1);
        // let f = self.file_dist(c1);
        // r.max(f)
        SQUAREDIST[*self][c1]
    }

}

use num_traits::{AsPrimitive,PrimInt};

mod impls {
    use super::*;

    macro_rules! impl_conv {
        ($t:ty) => {
            impl std::convert::From<$t> for Coord {
                fn from(sq: $t) -> Self {
                    // assert!(sq < 64);
                    Coord(sq as u8)
                }
            }
            impl std::convert::From<Coord> for $t {
            // impl std::convert::From<Coord> for usize {
                fn from(c: Coord) -> Self {
                    // assert!(sq < 64);
                    // Coord(sq as u8)
                    c.inner() as $t
                }
            }
        };
    }

    impl_conv!(usize);
    impl_conv!(u8);
    impl_conv!(u32);

}

// impl<T: PrimInt + AsPrimitive<u8>> std::convert::From<T> for Coord {
//     fn from(sq: T) -> Self {
//         // assert!(sq < 64);
//         // let x: u8 = sq.as_();
//         // Coord::new_int(x)
//         // BitBoard::index_bit(sq)
//         Coord::new_int(sq)
//     }
// }

// impl std::convert::From<usize> for Coord {
//     fn from(sq: usize) -> Self {
//         // assert!(sq < 64);
//         BitBoard::index_bit(sq as u8)
//     }
// }

// impl std::convert::From<Coord> for usize {
//     fn from(c: Coord) -> Self {
//         BitBoard::index_square(c) as usize
//     }
// }

// impl std::convert::From<u32> for Coord {
//     fn from(sq: u32) -> Self {
//         // assert!(sq < 64);
//         BitBoard::index_bit(sq as u8)
//     }
// }

// impl std::convert::From<u16> for Coord {
//     fn from(sq: u16) -> Self {
//         // assert!(sq < 64);
//         BitBoard::index_bit(sq as u8)
//     }
// }

// impl std::convert::From<u8> for Coord {
//     fn from(sq: u8) -> Self {
//         // assert!(sq < 64);
//         BitBoard::index_bit(sq)
//     }
// }

// impl std::convert::From<Coord> for u32 {
//     fn from(c: Coord) -> Self {
//         BitBoard::index_square(c) as u32
//     }
// }

// impl std::convert::From<Coord> for u16 {
//     fn from(c: Coord) -> Self {
//         BitBoard::index_square(c) as u16
//     }
// }

// impl std::convert::From<Coord> for u8 {
//     fn from(c: Coord) -> Self {
//         BitBoard::index_square(c)
//     }
// }

// impl std::convert::From<Coord> for usize {
//     fn from(c: Coord) -> Self {
//         BitBoard::index_square(c) as usize
//     }
// }

// impl std::convert::From<usize> for Coord {
//     fn from(sq: usize) -> Self {
//         BitBoard::index_bit(sq as u8)
//     }
// }

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

    pub fn shift_coord_mult(&self, c0: Coord, n: u32) -> Option<Coord> {
        if n > 0 {
            let c1 = self.shift_coord(c0)?;
            self.shift_coord_mult(c1, n - 1)
        } else {
            Some(c0)
        }
    }

    pub fn get_shift_n(&self) -> i8 {
        match *self {
            N  => 8,
            NE => 9,
            E  => 1,
            SE => -7,
            S  => -8,
            SW => -9,
            W  => -1,
            NW => 7,
        }
    }

    // pub fn shift_coord_idx_unchecked<T: Into<u8>>(&self, sq: T) -> u8 {
    // pub fn shift_coord_idx_unchecked<T: Into<u8>>(&self, sq: T, n: u8) -> u8 {
    pub fn shift_coord_idx_unchecked(&self, sq: u8, n: u8) -> u8 {

        let k = self.get_shift_n() * n as i8;
        (sq as i8 + k) as u8

        // match *self {
        //     N  => sq.into() + 8,
        //     NE => sq.into() + 9,
        //     E  => sq.into() + 1,
        //     SE => sq.into() - 7,
        //     S  => sq.into() - 8,
        //     SW => sq.into() - 9,
        //     W  => sq.into() - 1,
        //     NW => sq.into() + 7,
        // }

    }

    pub fn shift_coord(&self, x: Coord) -> Option<Coord> {
        let (x0,y0) = (x.file(),x.rank());
        match *self {
            N => {
                if y0 >= 7 { None } else {
                    Some(Coord::new(x0,y0+1))
                }
            },
            NE => {
                if (y0 >= 7) | (x0 >= 7) { None } else {
                    Some(Coord::new(x0+1,y0+1))
                }
            },
            E => {
                if x0 >= 7 { None } else {
                    Some(Coord::new(x0+1,y0))
                }
            },
            NW => {
                if (y0 >= 7) | (x0 == 0) { None } else {
                    Some(Coord::new(x0-1,y0+1))
                }
            },
            S => {
                if y0 == 0 { None } else {
                    Some(Coord::new(x0,y0-1))
                }
            },
            SE => {
                if (y0 == 0) | (x0 >= 7) { None } else {
                    Some(Coord::new(x0+1,y0-1))
                }
            },
            SW => {
                if (y0 == 0) | (x0 == 0) { None } else {
                    Some(Coord::new(x0-1,y0-1))
                }
            },
            W => {
                if x0 == 0 { None } else {
                    Some(Coord::new(x0-1,y0))
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

impl std::fmt::Debug for Sq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self.to()))?;
        Ok(())
    }
}

impl std::fmt::Debug for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let letters: [char; 8] = ['a','b','c','d','e','f','g','h'];
        let r = letters[self.file() as usize];
        // f.write_str(&format!("Coord({}{})", r, self.1+1))?;
        f.write_str(&format!("{}{}", r, self.rank()+1))?;
        Ok(())
    }
}

impl FromStr for Coord {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        assert_eq!(s.len(), 2);
        let s = s.to_ascii_uppercase();
        let letters: [char; 8] = ['A','B','C','D','E','F','G','H'];

        let x = s.chars().nth(0).unwrap();
        let x = x.to_ascii_uppercase();
        let x = letters.iter().position(|k| k == &x).unwrap();

        let y = format!("{}", s.chars().nth(1).unwrap());
        let y = y.parse::<u8>()?;
        let y = if y == 0 { 0 } else { y - 1 };

        assert!(x < 8);
        assert!(y < 8);

        Ok(Coord::new(x as u8,y))

        // let coords: Vec<&str> = s.trim_matches(|p| p == '(' || p == ')' )
        //     .split(',')
        //     .collect();

        // let x_fromstr = coords[0].parse::<i32>()?;
        // let y_fromstr = coords[1].parse::<i32>()?;

        // Ok(Point { x: x_fromstr, y: y_fromstr })
        // unimplemented!()
    }
}

fn test_directions() {

    let b = BitBoard::new(&[Coord::new(1,1)]);

    assert_eq!(b.shift_dir(D::E), BitBoard::new(&[Coord::new(2,1)]));
    assert_eq!(b.shift_dir(D::W), BitBoard::new(&[Coord::new(0,1)]));
    assert_eq!(b.shift_dir(D::N), BitBoard::new(&[Coord::new(1,2)]));
    assert_eq!(b.shift_dir(D::S), BitBoard::new(&[Coord::new(1,0)]));
    assert_eq!(b.shift_dir(D::NE), BitBoard::new(&[Coord::new(2,2)]));
    assert_eq!(b.shift_dir(D::NW), BitBoard::new(&[Coord::new(0,2)]));
    assert_eq!(b.shift_dir(D::SE), BitBoard::new(&[Coord::new(2,0)]));
    assert_eq!(b.shift_dir(D::SW), BitBoard::new(&[Coord::new(0,0)]));

    // let b = BitBoard::new(&vec![Coord(0,0)]);
    // assert_eq!(b.shift(D::W), BitBoard::new(&vec![]));
    // let b = BitBoard::new(&vec![Coord(7,0)]);
    // assert_eq!(b.shift(D::E), BitBoard::new(&vec![]));
    // let b = BitBoard::new(&vec![Coord(0,0)]);
    // assert_eq!(b.shift(D::S), BitBoard::new(&vec![]));
    // let b = BitBoard::new(&vec![Coord(0,7)]);
    // assert_eq!(b.shift(D::N), BitBoard::new(&vec![]));

}
