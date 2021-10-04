
use crate::types::*;
// pub use self::bits::*;

#[derive(Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct BitBoard(pub u64);

impl BitBoard {

    pub fn empty() -> BitBoard {
        BitBoard(0)
    }
    pub fn new(cs: &[Coord]) -> BitBoard {
        let mut b = BitBoard::empty();
        for c in cs.iter() {
            b.flip_mut(*c);
        }
        b
    }

    pub fn single(c: Coord) -> BitBoard {
        let mut b = BitBoard::empty();
        b.flip_mut(c);
        b
    }

    pub fn get(&self, c: Coord) -> bool {
        let p = Self::index_square(c);
        let k = (self.0 >> p) & 1;
        k == 1
    }

    pub fn flip_mut(&mut self, c: Coord) {
        let p = Self::index_square(c);
        // eprintln!("c, p = {:?}, {:?}", c, p);
        let k = 1 << p;
        self.0 |= k;
    }

    pub fn flip_mut_mult(&mut self, cs: &[Coord]) {
        for c in cs.iter() {
            self.flip_mut(*c);
        }
    }

    pub fn index_square(c: Coord) -> u64 {
        // Little Endian Rank File Mapping
        // Least Significant File Mapping
        let p: u64 = c.0 as u64 + 8 * c.1 as u64;
        p
    }

    pub fn index_rank(s: u64) -> u64 {
        s >> 3
    }

    pub fn index_file(s: u64) -> u64 {
        s & 7
    }

    pub fn mask_file(f: u64) -> Self {
        unimplemented!()
    }

}

impl BitBoard {

    pub fn flip_vert(&self) -> Self {
        let mut x = self.0;
        let x = x.reverse_bits();
        Self(x)
    }

    pub fn shift_unwrapped(&self, d: D) -> Self {

        let k = d.shift();

        // let b = self.0 << k;
        let b = if k > 0 {
            self.0 << (k as u64)
        } else {
            self.0 >> (k.abs() as u64)
        };

        // TODO: unwrap

        Self(b)
        // unimplemented!()
    }
}

impl std::fmt::Debug for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ss: &str = &format!("{:0>64b}", self.0);
        f.write_str(&format!("\nBitBoard:\n"))?;
        for y in 0..8 {
            let (c, rest) = ss.split_at(8);
            let c = c.chars().rev().collect::<String>();
            f.write_str(&format!("{}\n", c))?;
            ss = rest;
        }
        Ok(())
    }
}

pub mod bits {

    use super::BitBoard;
    use std::ops::{BitAnd,BitAndAssign,BitOr,BitOrAssign,BitXor,BitXorAssign,Not};

    impl BitAnd for BitBoard {
        type Output = Self;
        fn bitand(self, rhs: Self) -> Self::Output {
            Self(self.0 & rhs.0)
        }
    }

    impl BitAndAssign for BitBoard {
        fn bitand_assign(&mut self, rhs: Self) {
            *self = Self(self.0 & rhs.0)
        }
    }

    impl BitOr for BitBoard {
        type Output = Self;
        fn bitor(self, rhs: Self) -> Self::Output {
            Self(self.0 | rhs.0)
        }
    }

    impl BitOrAssign for BitBoard {
        fn bitor_assign(&mut self, rhs: Self) {
            *self = Self(self.0 | rhs.0)
        }
    }

    impl BitXor for BitBoard {
        type Output = Self;
        fn bitxor(self, rhs: Self) -> Self::Output {
            Self(self.0 ^ rhs.0)
        }
    }

    impl BitXorAssign for BitBoard {
        fn bitxor_assign(&mut self, rhs: Self) {
            *self = Self(self.0 ^ rhs.0)
        }
    }


    impl Not for BitBoard {
        type Output = Self;
        fn not(self) -> Self::Output {
            Self(!self.0)
        }
    }

}
