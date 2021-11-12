
use crate::tables::*;
use crate::types::*;
use crate::evaluate::*;
use crate::brain::types::*;

use ndarray::prelude::*;
use ndarray_rand::RandomExt;
use ndarray_rand::rand_distr::Uniform;

use rand::{Rng,SeedableRng};
use rand::prelude::StdRng;

#[derive(Debug,Clone)]
pub struct NNUE {
}


// #[derive(Debug,Clone)]
// pub struct NNUE {
//     layer1_own:     Network<u16>,
//     layer1_other:   Network<u16>,
// }

// impl NNUE {
//     pub fn new() -> Self {
//         let layer1_own = Network::new(
//             63 * 64 * 10,
//             0,
//             0,
//             32
//         );
//         let layer1_other = layer1_own.clone();
//         Self {
//             layer1_own,
//             layer1_other,
//         }
//     }
// }

// impl NNUE {

//     pub fn index(king_sq: Coord, pc: Piece, c0: Coord, side: Color) -> usize {
//         assert!(pc != King);
//         let king_sq: u64 = BitBoard::index_square(king_sq) as u64;
//         let c0: u64      = BitBoard::index_square(c0) as u64;
//         let mut out = king_sq * (64 * 5 * 2);
//         let pc1 = if side == White {
//             pc.index()
//         } else {
//             pc.index() + 5
//         };
//         let c1 = c0 as usize * 10 + pc1;
//         (out as usize) + c1
//     }

// }


