
use crate::types::*;

use std::io;
use std::path::Path;

use byteorder::{ReadBytesExt, LittleEndian};


const SQUARE_NB: usize = 64;

const PS_NONE: usize     = 0;
const PS_W_PAWN: usize   = 0;
const PS_B_PAWN: usize   = 1 * SQUARE_NB;
const PS_W_KNIGHT: usize = 2 * SQUARE_NB;
const PS_B_KNIGHT: usize = 3 * SQUARE_NB;
const PS_W_BISHOP: usize = 4 * SQUARE_NB;
const PS_B_BISHOP: usize = 5 * SQUARE_NB;
const PS_W_ROOK: usize   = 6 * SQUARE_NB;
const PS_B_ROOK: usize   = 7 * SQUARE_NB;
const PS_W_QUEEN: usize  = 8 * SQUARE_NB;
const PS_B_QUEEN: usize  = 9 * SQUARE_NB;
const PS_KING: usize     = 10 * SQUARE_NB;
const PS_NB: usize       = 11 * SQUARE_NB;

const PIECE_SQ_INDEX: [[[usize; 8]; 2]; 2] = [
    [ [PS_NONE, PS_W_PAWN, PS_W_KNIGHT, PS_W_BISHOP, PS_W_ROOK, PS_W_QUEEN, PS_KING, PS_NONE],
      [PS_NONE, PS_B_PAWN, PS_B_KNIGHT, PS_B_BISHOP, PS_B_ROOK, PS_B_QUEEN, PS_KING, PS_NONE],
    ],
    [ [PS_NONE, PS_B_PAWN, PS_B_KNIGHT, PS_B_BISHOP, PS_B_ROOK, PS_B_QUEEN, PS_KING, PS_NONE],
      [PS_NONE, PS_W_PAWN, PS_W_KNIGHT, PS_W_BISHOP, PS_W_ROOK, PS_W_QUEEN, PS_KING, PS_NONE],
    ],
];

const KING_BUCKETS: [i8; 64] = [
    -1, -1, -1, -1, 31, 30, 29, 28,
    -1, -1, -1, -1, 27, 26, 25, 24,
    -1, -1, -1, -1, 23, 22, 21, 20,
    -1, -1, -1, -1, 19, 18, 17, 16,
    -1, -1, -1, -1, 15, 14, 13, 12,
    -1, -1, -1, -1, 11, 10, 9, 8,
    -1, -1, -1, -1, 7, 6, 5, 4,
    -1, -1, -1, -1, 3, 2, 1, 0
];

pub fn read_nnue<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let mut f = std::fs::File::open(path)?;
    _read_nnue(f)
}

fn _read_nnue(mut f: std::fs::File) -> io::Result<()> {
    let mut rdr = io::BufReader::new(f);

    let version   = rdr.read_u32::<LittleEndian>()?;
    eprintln!("version = {:?}", version);

    let hashvalue = rdr.read_u32::<LittleEndian>()?;
    eprintln!("hashvalue = {:?}", hashvalue);

    let size      = rdr.read_u32::<LittleEndian>()?;
    eprintln!("size = {:?}", size);

    unimplemented!()
}

fn orient(king_sq: u8, persp: Color, sq: u8) -> u8 {
    let p = persp.fold(0, 1);
    const A8: u64 = 56;
    const H1: u64 = 7;
    let x = if Coord::from(king_sq).file() < 4 { 1 } else { 0 };
    let out = sq as u64 ^ (p * A8) ^ (x * H1);
    out as u8
}

pub fn make_index_half_ka_v2(king_sq: u8, persp: Color, pc: Piece, side: Color, sq: u8) -> usize {
    let o_king_sq = orient(king_sq, persp, sq);

    // const WID: [usize; 8] = [0, ]

    let pidx = PIECE_SQ_INDEX[side][persp][pc.index() + 1];

    let pc_nb = KING_BUCKETS[o_king_sq as usize];
    assert!(pc_nb > 0);
    let pc_nb = PS_NB * pc_nb as usize;

    orient(king_sq, persp, sq) as usize
        + pidx
        + pc_nb

    // Square o_ksq = orient(perspective, ksq, ksq);
    // IndexType(
    //     orient(perspective, s, ksq)
    //         + PieceSquareIndex[perspective][pc]
    //         + PS_NB * KingBuckets[o_ksq]);

    // unimplemented!()
}

