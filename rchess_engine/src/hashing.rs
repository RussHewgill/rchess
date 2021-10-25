
use crate::types::*;
use crate::tables::*;

use rand::{Rng,SeedableRng};
use rand::prelude::StdRng;

// use evmap_derive::ShallowCopy;

// #[derive(Hash,Eq,PartialEq,Ord,PartialOrd,ShallowCopy,Clone,Copy)]
#[derive(Hash,Eq,PartialEq,Ord,PartialOrd,Clone,Copy)]
pub struct Zobrist(pub u64);

impl Zobrist {
    pub fn new(ts: &Tables, g: Game) -> Self {
        let mut out = 0u64;
        let zb = &ts.zobrist_tables;

        for &col in [White,Black].iter() {
            for pc in Piece::iter_pieces() {
                let b = g.get(pc,col);
                for sq in b.into_iter() {
                    out ^= zb.get_piece(pc, col)[sq as usize];
                }
            }
        }

        if g.state.side_to_move == Black { out ^= zb.black_to_move; }

        let c = g.state.castling.get();
        let (w,b) = {
            let (a,b,c,d) = (c & 0b1000,c & 0b0100, c & 0b0010, c & 0b0001);
            ((a >> 2) | (b >> 2), c | d)
        };
        out ^= zb.castling[White][w as usize];
        out ^= zb.castling[Black][b as usize];

        if let Some(ep) = g.state.en_passant {
            out ^= zb.en_passant[ep.0 as usize];
        }

        Zobrist(out)
    }

    #[must_use]
    pub fn update_side_to_move(&self, ts: &Tables) -> Self {
        let mut out = self.0;
        out ^= ts.zobrist_tables.black_to_move;
        Self(out)
    }

    #[must_use]
    pub fn update_castling(&self, ts: &Tables, c: Castling) -> Self {
        let mut out = self.0;
        let c = c.get();
        let (w,b) = {
            let (a,b,c,d) = (c & 0b1000,c & 0b0100, c & 0b0010, c & 0b0001);
            ((a >> 2) | (b >> 2), c | d)
        };
        out ^= ts.zobrist_tables.castling[White][w as usize];
        out ^= ts.zobrist_tables.castling[Black][b as usize];
        Self(out)
    }

    #[must_use]
    pub fn update_ep(&self, ts: &Tables, c0: Coord) -> Self {
        let mut out = self.0;
        out ^= ts.zobrist_tables.en_passant[c0.0 as usize];
        Self(out)
    }

    #[must_use]
    pub fn update_piece(&self, ts: &Tables, pc: Piece, col: Color, c0: Coord) -> Self {
        let mut out = self.0;
        out ^= ts.zobrist_tables.pieces[col][pc.index()][c0];
        Self(out)
    }

}

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct ZbTable {
    pub pieces:        [[[u64; 64]; 6]; 2],
    pub black_to_move: u64,
    pub castling:      [[u64; 4]; 2],
    pub en_passant:    [u64; 8],
}

impl Default for ZbTable {
    fn default() -> Self {
        Self::new()
    }
}

impl ZbTable {

    pub fn get_piece(&self, pc: Piece, col: Color) -> [u64; 64] {
        let cc = if col == White { 0 } else { 1 };
        self.pieces[cc][pc.index()]
    }

    pub fn new() -> ZbTable {

        // let mut rng: StdRng = SeedableRng::seed_from_u64(8576372831478151420);
        // let mut rng: StdRng = SeedableRng::seed_from_u64(5474752555881496643);
        let mut rng: StdRng = SeedableRng::seed_from_u64(18105974836011991331);

        use array_init::array_init;
        let pieces = array_init(|_| {
            array_init(|_| {
                array_init(|_| rng.gen())
            })
        });

        let castling = array_init(|_| {
            array_init(|_| rng.gen())
        });
        let en_passant = array_init(|_| rng.gen());

        ZbTable {
            pieces,
            black_to_move: rng.gen(),
            castling,
            en_passant,
        }
    }

}

impl std::fmt::Debug for Zobrist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:#8x}", self.0))?;
        Ok(())
    }
}


