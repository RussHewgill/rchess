
use crate::types::*;
use crate::tables::*;

use rand::{Rng,SeedableRng};
use rand::prelude::StdRng;

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct Zobrist(pub u64);

impl Zobrist {
    pub fn new(ts: &Tables, g: Game) -> Self {
        let mut out = 0u64;

        // for sq in 0..64u32 {
        //     let c0: Coord = sq.into();
        // }

        for &col in [White,Black].iter() {
            for pc in Piece::iter_pieces() {
                let b = g.get(pc,col);
                for sq in b.into_iter() {
                    out ^= ts.zobrist_tables.get_piece(pc, col)[sq as usize];
                }
            }
        }

        if g.state.side_to_move == Black { out ^= ts.zobrist_tables.black_to_move; }

        Zobrist(out)
        // unimplemented!()
    }

    pub fn update(&self, m: Move) -> Self {
        unimplemented!()
    }

}

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct ZbTable {
    pieces:        [[[u64; 64]; 6]; 2],
    black_to_move: u64,
    castling:      [u64; 16],
    en_passant:    [u64; 8],
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

        let mut pieces = [[[0u64; 64]; 6]; 2];
        for mut col in pieces.iter_mut() {
            for pc in Piece::iter_pieces() {
                col[pc.index()][0] = rng.gen();
                col[pc.index()][1] = rng.gen();
            }
        }
        let castling = [rng.gen(); 16];
        let en_passant = [rng.gen(); 8];

        ZbTable {
            pieces,
            black_to_move: rng.gen(),
            castling,
            en_passant,
        }
    }

}


/// Zobrist Hashing
impl GameState {
    // pub fn 
}



