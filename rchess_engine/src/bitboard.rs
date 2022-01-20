
use crate::types::*;

pub use self::lookups::*;

use derive_more::*;

mod lookups {
    use lazy_static::lazy_static;

    pub const MASK_FILES: [BitBoard; 8] = [
        BitBoard(0x0101010101010101),
        BitBoard(0x0202020202020202),
        BitBoard(0x0404040404040404),
        BitBoard(0x0808080808080808),
        BitBoard(0x1010101010101010),
        BitBoard(0x2020202020202020),
        BitBoard(0x4040404040404040),
        BitBoard(0x8080808080808080),
    ];

    pub const MASK_RANKS: [BitBoard; 8] = [
        BitBoard(0x00000000000000ff),
        BitBoard(0x000000000000ff00),
        BitBoard(0x0000000000ff0000),
        BitBoard(0x00000000ff000000),
        BitBoard(0x000000ff00000000),
        BitBoard(0x0000ff0000000000),
        BitBoard(0x00ff000000000000),
        BitBoard(0xff00000000000000),
    ];

    pub const WHITE_SQUARES: BitBoard = BitBoard(0x55AA55AA55AA55AA);
    pub const BLACK_SQUARES: BitBoard = BitBoard(0xAA55AA55AA55AA55);

    pub const DIAG_A1_H8: BitBoard = BitBoard(0x8040201008040201);
    pub const DIAG_A8_H1: BitBoard = BitBoard(0x0102040810204080);

    lazy_static! { /// SQUARE_BB
        // pub static ref SQUARE_BB: [BitBoard; 64] = array_init::array_init(|x| BitBoard::single(Coord::new_int(x)));
        pub static ref SQUARE_BB: [BitBoard; 64] = array_init::array_init(|x| {
            // let mut b = BitBoard::empty();
            // b.flip_mut(Coord::new_int_const(x as u8));
            // b
            unimplemented!()
        });
    }
}


#[derive(Serialize,Deserialize,Hash,Eq,PartialEq,PartialOrd,Clone,Copy,
         Index,Add,Mul,Div,Sum,AddAssign,MulAssign,
         BitAnd,BitAndAssign,BitOr,BitOrAssign,BitXor,BitXorAssign,Not,
         From,Into,AsRef,AsMut
)]
pub struct BitBoard(pub u64);

/// new
impl BitBoard {
    pub fn new<T: Into<Coord> + Copy>(cs: &[T]) -> Self {
        unimplemented!()
    }

    pub fn empty() -> BitBoard {
        BitBoard(0)
    }

    pub fn single(c: Coord) -> BitBoard {
        SQUARE_BB[c]
    }

}

/// Queries
impl BitBoard {

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn is_not_empty(&self) -> bool {
        self.0 != 0
    }

}

/// Masks
impl BitBoard {

    pub fn mask_rank(r: u8) -> BitBoard {
        // assert!(r < 8);
        // let k = (!0u8) as u64;
        // BitBoard(k.overflowing_shl(r * 8).0)
        MASK_RANKS[r as usize]
    }

    pub fn mask_file(f: u8) -> BitBoard {
        // assert!(f < 8);
        // let k = 0x0101010101010101u64;
        // BitBoard(k.overflowing_shl(f).0)
        MASK_FILES[f as usize]
    }

    pub fn mask_between(ts: &Tables, c0: Coord, c1: Coord) -> BitBoard {
    // pub fn obstructed(&self, ts: &Tables, c0: Coord, c1: Coord) -> BitBoard {

        let (x0,y0) = c0.to_rankfile();
        let (x1,y1) = c1.to_rankfile();

        if x0 == x1 {
            // File
            let (x0,x1) = (x0.min(x1),x0.max(x1));
            let (y0,y1) = (y0.min(y1),y0.max(y1));
            let b0 = BitBoard::single(Coord::new(x0,y0));
            let b1 = BitBoard::single(Coord::new(x1,y1));
            let b = BitBoard(2 * b1.0 - b0.0);
            let m = BitBoard::mask_file(x0.into());
            (b & m) & !(b0 | b1)
        } else if y0 == y1 {
            // Rank
            let (x0,x1) = (x0.min(x1),x0.max(x1));
            let (y0,y1) = (y0.min(y1),y0.max(y1));
            let b0 = BitBoard::single(Coord::new(x0,y0));
            let b1 = BitBoard::single(Coord::new(x1,y1));
            let b = BitBoard(2 * b1.0 - b0.0);
            let m = BitBoard::mask_rank(y0.into());
            (b & m) & !(b0 | b1)
        // } else if (x1 - x0) == (y1 - y0) {
        } else if (x1 as i64 - x0 as i64).abs() == (y1 as i64 - y0 as i64).abs() {
            // Diagonal
            let b0 = BitBoard::single(Coord::new(x0,y0));
            let b1 = BitBoard::single(Coord::new(x1,y1));
            // let b = BitBoard::new(&[Coord(x0,y0),Coord(x1,y1)])

            let (bb0,bb1) = (b0.0.min(b1.0),b0.0.max(b1.0));

            // eprintln!("b0 = {:?}", b0);
            // eprintln!("b1 = {:?}", b1);

            // eprintln!("bb0 = {:?}", b0.bitscan());
            // eprintln!("bb1 = {:?}", b1.bitscan());

            let b = BitBoard(2u64.overflowing_mul(bb1).0.overflowing_sub(bb0).0);
            // let b = BitBoard(2 * b0.0 - b1.0);
            // eprintln!("b = {:?}", b);
            // let m = BitBoard::mask_rank(y0.into());
            let m = ts.get_bishop(c0);

            let xx = x1 as i64 - x0 as i64;
            let yy = y1 as i64 - y0 as i64;

            let m = if xx.signum() == yy.signum() {
                m.ne | m.sw
            } else {
                m.nw | m.se
            };

            (b & m) & !(b0 | b1)
        } else {
            // println!("wat 2");
            // unimplemented!()
            BitBoard::empty()
        }
    }


}

/// Bitscan, popcount
impl BitBoard {
}

/// iter_subsets
impl BitBoard {
    /// Used for magic gen
    pub fn iter_subsets(&self) -> Vec<BitBoard> {
        let mut out = vec![];
        let mut n: u64 = 0;

        loop {
            out.push(BitBoard(n));

            // n = (n - self.0) & self.0;
            n = (n.overflowing_sub(self.0).0) & self.0;

            if n == 0 { break; }
        }

        out
    }
}

/// Display
impl BitBoard {
    pub fn print_hex(&self) -> String {
        format!("hex: {:#8x}", self.0)
    }
}

impl Default for BitBoard {
    fn default() -> Self {
        Self(0)
    }
}

impl std::fmt::Debug for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ss: &str = &format!("{:0>64b}", self.0);
        f.write_str(&format!("BitBoard:\n"))?;
        for y in 0..8 {
            let (c, rest) = ss.split_at(8);
            let c = c.chars().rev().collect::<String>();
            if y == 7 {
                f.write_str(&format!("{}", c))?;
            } else {
                f.write_str(&format!("{}\n", c))?;
            }
            ss = rest;
        }
        Ok(())
    }
}


