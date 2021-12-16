
pub mod layers;
pub mod accumulator;
pub mod feature_trans;

pub use self::feature_trans::NNFeatureTrans;
pub use self::accumulator::NNAccum;
pub use self::layers::{NNAffine,NNClippedRelu,NNInput,NNLayer};

use crate::evaluate::Score;
use crate::types::*;

use std::io::{self,Read,BufReader,Write,BufWriter};
use std::fs::File;
use std::path::Path;

use aligned::{Aligned,A64};
use byteorder::{ReadBytesExt, LittleEndian, WriteBytesExt};

pub use self::index::NNIndex;
mod index {
    use super::*;
    use derive_more::*;
    use lazy_static::lazy_static;
    use parking_lot::Mutex;
    use std::collections::HashMap;

    lazy_static!{
        static ref NNINDEX_MAP: Mutex<HashMap<(Color,NNIndex),(Coord,Piece,Color,Coord)>> = {
            Mutex::new(HashMap::new())
        };
    }

    #[derive(Debug,Deref,Eq,Ord,PartialEq,PartialOrd,Hash,Clone,Copy,
             Index,Add,Mul,Div,Sum,AddAssign,MulAssign,
             From,Into,AsRef,AsMut
    )]
    pub struct NNIndex(pub usize);

    impl NNIndex {
        pub fn get_index(&self, persp: Color) -> (Coord,Piece,Color,Coord) {
            let mut map = NNINDEX_MAP.lock();
            if let Some(xs) = map.get(&(persp,*self)) { *xs } else {
                // let mut xs
                for ksq in 0u8..64 {
                    let ksq = Coord::new_int(ksq);
                    for pc in Piece::iter_pieces() {
                        for side in [White,Black] {
                            for sq in 0u8..64 {
                                let sq = Coord::new_int(sq);
                                let idx = NNUE4::make_index_half_ka_v2(ksq, persp, pc, side, sq);
                                if self == &idx {
                                    map.insert((persp,*self), (ksq,pc,side,sq));
                                    return (ksq,pc,side,sq);
                                }
                            }
                        }
                    }
                }
                panic!("index not found?");
            }
        }
    }

}

// XXX: manually specifying input dims allows better const optimizations maybe?
pub type Layer0 = NNInput<{HALF_DIMS * 2}>;
pub type Layer1 = NNClippedRelu<NNAffine<Layer0, 8, {HALF_DIMS * 2}>, 8>;
pub type Layer2 = NNClippedRelu<NNAffine<Layer1, 32, 8>, 32>;
pub type Layer3 = NNAffine<Layer2, 1, 32>;

const HALF_DIMS: usize = 1024;
const OUTPUT_SCALE: Score = 16;

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

#[derive(Debug,Eq,PartialEq,Clone)]
pub struct NNUE4 {
    pub ft:      NNFeatureTrans,
    pub layers:  Vec<Layer3>,
}

/// Misc, Consts
impl NNUE4 {
    pub const HASH: u32 = NNFeatureTrans::HASH ^ Layer3::HASH;
}

/// Read, write from file
impl NNUE4 {

    pub fn write_nnue<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut f = std::fs::File::create(path)?;
        let mut w = io::BufWriter::new(f);

        let version   = 0x7af32f20;
        let hashvalue = NNUE4::HASH;
        let size = 75;
        let desc: &[u8] = b"Network trained with the https://github.com/glinscott/nnue-pytorch trainer.";

        w.write_u32::<LittleEndian>(version)?;
        w.write_u32::<LittleEndian>(hashvalue)?;
        w.write_u32::<LittleEndian>(size)?;

        assert_eq!(desc.len(), size as usize);
        w.write_all(desc)?;

        self.ft.write_parameters(&mut w)?;

        for layer in self.layers.iter() {
            w.write_u32::<LittleEndian>(Self::HASH)?;
            layer.write_parameters(&mut w)?;
        }

        Ok(())
    }

    pub fn read_nnue<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut f = std::fs::File::open(path)?;
        let (ft,layers) = Self::_read_nnue(f)?;
        Ok(Self {
            ft,
            layers,
        })
    }

    fn _read_nnue(mut f: std::fs::File) -> io::Result<(NNFeatureTrans,Vec<Layer3>)> {
        let mut rdr = io::BufReader::new(f);

        let version   = rdr.read_u32::<LittleEndian>()?;
        // eprintln!("version = {:#8x}", version);

        let hashvalue = rdr.read_u32::<LittleEndian>()?;
        // eprintln!("hashvalue = {:#8x}", hashvalue);

        let size      = rdr.read_u32::<LittleEndian>()?;
        // eprintln!("size = {:?}", size);

        let mut desc = vec![0u8; size as usize];
        rdr.read_exact(&mut desc)?;
        let desc = String::from_utf8_lossy(&desc);
        // eprintln!("desc = {:?}", desc);

        let mut ft = NNFeatureTrans::new();
        ft.read_parameters(&mut rdr)?;

        let mut layers: Vec<Layer3> = {
            let layer0 = Layer0::new();
            let layer1 = Layer1::new(NNAffine::new(layer0));
            let layer2 = Layer2::new(NNAffine::new(layer1));
            let layer3 = Layer3::new(layer2);

            vec![layer3; 8]
            // vec![NNFeatureTrans::new(layer3); 8]
        };

        for (n,mut layer) in layers.iter_mut().enumerate() {

            let hash = rdr.read_u32::<LittleEndian>()?;
            assert_eq!(hash, Layer3::HASH);

            // eprintln!("layer = {:?}", n);
            layer.read_parameters(&mut rdr)?;
        }

        // let mut r = rdr.get_mut();
        let mut xs = vec![];
        let end = rdr.read_to_end(&mut xs)?;
        // eprintln!("end = {:?}", end);
        assert!(end == 0);

        Ok((ft,layers))
    }

}

/// Init
impl NNUE4 {
    // pub fn init_inputs(&mut self, )


}

/// Evaluate
impl NNUE4 {

    pub fn trace_eval(&mut self, g: &Game, adjusted: bool) -> ([Score; 8], [Score; 8],usize) {

        let mut out_psqt       = [0; 8];
        let mut out_positional = [0; 8];
        let correct_bucket = (g.state.material.count() as usize - 1) / 4;

        for bucket in 0..8 {

            let mut transformed = [0; HALF_DIMS * 2]; // 2048
            // let psqt = self.ft.transform(&g, &mut transformed, bucket, true);
            let psqt = self.ft.transform(g, &mut transformed, bucket);

            // let mut pos_buf = [0; Layer3::BUFFER_SIZE]; // XXX: 320 ??
            // // eprintln!("pos_buf.len() = {:?}", pos_buf.len());
            // self.layers[bucket].propagate(&transformed, &mut pos_buf);

            // let pos_buf: Vec<i32> = self.layers[bucket].propagate(&transformed);
            self.layers[bucket].propagate(&transformed);
            let pos_buf = self.layers[bucket].get_buf();
            let positional = pos_buf[0] as Score;

            // for (n,p) in pos_buf.iter().enumerate() {
            //     if *p != 0 {
            //         eprintln!("{} = {:?}", n, p);
            //     }
            // }

            out_psqt[bucket] = psqt / OUTPUT_SCALE;
            out_positional[bucket] = positional as i32 / OUTPUT_SCALE;

            // out_psqt[bucket] = psqt;
            // out_positional[bucket] = positional as i32;

        }

        (out_psqt,out_positional,correct_bucket)
    }

    // TODO: check for correctness with trace_eval
    pub fn evaluate(&mut self, g: &Game, adjusted: bool) -> Score {

        let c = g.state.material.count();
        let bucket = (c as usize - 1) / 4;
        // let bucket = if c == 0 { 0 } else { (c as usize - 1) / 4 };
        // eprintln!("bucket = {:?}", bucket);

        // self.ft.reset_accum(g);

        let mut transformed: Aligned<A64,_> = Aligned([0; HALF_DIMS * 2]);
        // let psqt = self.ft.transform(g, &mut transformed, bucket, refresh);
        let psqt = self.ft.transform(g, transformed.as_mut(), bucket);

        // // let mut pos_buf = [0; Layer3::BUFFER_SIZE]; // ?? 384
        // let mut pos_buf = [0; Layer3::SIZE_OUTPUT]; // 1
        // // eprintln!("pos_buf.len() = {:?}", pos_buf.len());
        // self.layers[bucket].propagate(&transformed, &mut pos_buf);

        // let pos_buf = self.layers[bucket].propagate(&transformed);
        self.layers[bucket].propagate(&transformed.as_ref());
        let pos_buf = self.layers[bucket].get_buf();
        let positional = pos_buf[0] as Score;

        // for (n,p) in pos_buf.iter().enumerate() {
        //     eprintln!("{:>5} = {:?}", n, p);
        // }

        // eprintln!("psqt       = {:?}", psqt);
        // eprintln!("positional = {:?}", positional);

        const RMB: Score = Rook.score() - Bishop.score();
        const DELTA: Score = 7;

        // TODO: if adjusted
        if adjusted
            && (g.state.material.non_pawn_value(White) - g.state.material.non_pawn_value(Black)) <= RMB {
            ((128 - DELTA) * psqt + (128 + DELTA) * positional) / 128 / OUTPUT_SCALE
        } else {
            (psqt + positional) / OUTPUT_SCALE
            // unimplemented!()
        }

    }

}

/// Index
impl NNUE4 {

    pub fn orient(king_sq: Coord, persp: Color, sq: Coord) -> Coord {
        let p = persp.fold(0, 1);

        const A8: u64 = 56;
        const H1: u64 = 7;

        let x = if king_sq.file() < 4 { 1 } else { 0 };

        let out = sq.inner() as u64 ^ (p * A8) ^ (x * H1);

        Coord::new_int(out)
    }

    pub fn make_index_half_ka_v2(king_sq: Coord, persp: Color, pc: Piece, side: Color, sq: Coord) -> NNIndex {
        let o_king_sq = Self::orient(king_sq, persp, king_sq);

        // let pidx = PIECE_SQ_INDEX[side][persp][pc.index() + 1];
        let pidx = PIECE_SQ_INDEX[persp][side][pc.index() + 1];

        let pc_nb = KING_BUCKETS[o_king_sq.inner() as usize];
        // assert!(pc_nb > 0);
        let pc_nb = PS_NB * pc_nb as usize;

        NNIndex(Self::orient(king_sq, persp, sq).inner() as usize
                + pidx
                + pc_nb)
    }

    // pub fn make_index_2((ksq1,ksq2): (Coord,Coord), pc: Piece, side: Color, sq: Coord) -> (NNIndex,NNIndex) {
    pub fn make_index_2(ksqs: [Coord; 2], pc: Piece, side: Color, sq: Coord) -> [NNIndex; 2] {
        let w = Self::make_index_half_ka_v2(ksqs[White], White, pc, side, sq);
        let b = Self::make_index_half_ka_v2(ksqs[Black], Black, pc, side, sq);
        [w,b]
    }

}




