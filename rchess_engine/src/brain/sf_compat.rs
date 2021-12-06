
use crate::types::*;
pub use self::types::*;

use std::io::{self, Read};
use std::path::Path;

use byteorder::{ReadBytesExt, LittleEndian};

const HALF_DIMS: usize = 1024;

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

mod types {
    use super::*;
    pub use self::nn_input::NNInput;
    pub use self::nn_affine::NNAffine;
    pub use self::nn_relu::NNClippedRelu;
    pub use self::feature_trans::NNFeatureTrans;
    pub use self::accumulator::NNAccum;

    use std::{marker::PhantomData, io::BufReader, fs::File};
    use num_traits::{Num,PrimInt,NumCast};

    pub type Layer0 = NNInput<{HALF_DIMS * 2}>;
    pub type Layer1 = NNClippedRelu<NNAffine<Layer0, 8>>;
    pub type Layer2 = NNClippedRelu<NNAffine<Layer1, 32>>;
    pub type Layer3 = NNAffine<Layer2, 1>;

    const CACHE_LINE_SIZE: usize = 64;
    const WEIGHT_SCALE_BITS: usize = 6;

    pub trait NNLayer {
        // type InputType: Num + Copy + std::ops::Shr;
        // type OutputType: Num + Copy + std::ops::Shr;
        type InputType: PrimInt + NumCast;
        type OutputType: PrimInt + NumCast;

        const SIZE_OUTPUT: usize;
        const SIZE_INPUT: usize;

        const SELF_BUFFER_SIZE: usize;

        const HASH: u32;

        // fn propogate(&self, input: &[u8]) -> Vec<Self::OutputType>;
        fn propogate(&self, input: &[u8], output: &mut [Self::OutputType]);

        fn size(&self) -> usize;

        fn read_parameters(&mut self, rdr: &mut BufReader<File>) -> io::Result<()> {
            Ok(())
        }

    }

    const fn ceil_to_multiple(n: usize, base: usize) -> usize {
        (n + base - 1) / base * base
    }

    mod accumulator {
        use super::*;

        #[derive(Debug,PartialEq,Clone)]
        pub struct NNAccum {
        }

    }

    mod feature_trans {
        use super::*;

        #[derive(Debug,PartialEq,Clone)]
        pub struct NNFeatureTrans {
            biases:         Vec<i16>,
            weights:        Vec<i16>,
            psqt_weights:   Vec<i32>,

        }

        /// Consts, Init
        impl NNFeatureTrans {
            const DIMS_HALF: usize = 1024;

            const DIMS_IN: usize = 64 * 11 * 64 / 2;
            const DIMS_OUT: usize = Self::DIMS_HALF * 2;

            const PSQT_BUCKETS: usize = 8;
            const LAYER_STACKS: usize = 8;

            pub fn new() -> Self {
                Self {
                    // nn,
                    biases:         vec![0; Self::DIMS_HALF],
                    weights:        vec![0; Self::DIMS_HALF * Self::DIMS_IN],
                    psqt_weights:   vec![0; Self::DIMS_IN * Self::PSQT_BUCKETS],
                }
            }

            pub fn read_parameters(&mut self, mut rdr: &mut BufReader<File>) -> io::Result<()> {
                // println!("wat NNFeatureTrans");

                for mut x in self.biases.iter_mut() {
                    *x = rdr.read_i16::<LittleEndian>()?;
                }

                for mut x in self.weights.iter_mut() {
                    *x = rdr.read_i16::<LittleEndian>()?;
                }

                for mut x in self.psqt_weights.iter_mut() {
                    *x = rdr.read_i32::<LittleEndian>()?;
                }

                Ok(())
            }

        }

        /// Update Accum
        impl NNFeatureTrans {

            pub fn transform(&self, g: &Game, output: &mut [u8], bucket: usize) -> i32 {
                self.update_accum(g, White);
                self.update_accum(g, Black);

                let persps: [Color; 2] = [g.state.side_to_move, !g.state.side_to_move];

                // let accum = 

                unimplemented!()
            }

            pub fn evaluate(&mut self, g: &Game) {
                let bucket = (g.state.material.count() - 1) / 4;

                // let psqt = self.f

            }

            pub fn update_accum(&mut self, g: &Game, persp: Color) {
                unimplemented!()
            }
        }

    }

    /// Input
    mod nn_input {
        use super::*;
        use num_traits::Num;

        #[derive(Debug,PartialEq,Clone,Copy)]
        pub struct NNInput<const OS: usize> {
            buf:   [u8; OS],
        }

        impl<const OS: usize> NNLayer for NNInput<OS> {
            type InputType = u8;
            type OutputType = u8;
            const SIZE_OUTPUT: usize = OS;
            const SIZE_INPUT: usize = OS;

            const SELF_BUFFER_SIZE: usize = OS;

            const HASH: u32 = 0xEC42E90D ^ Self::SIZE_OUTPUT as u32;

            fn size(&self) -> usize { self.buf.len() }

            // fn propogate(&self, input: &[u8]) -> Vec<Self::OutputType> {
            fn propogate(&self, input: &[u8], output: &mut [Self::OutputType]) {
                // self.buf.to_vec()
                // assert!(input.len() == output.len());
                assert_eq!(input.len(), Self::SELF_BUFFER_SIZE);
                assert_eq!(output.len(), Self::SELF_BUFFER_SIZE);
                output.copy_from_slice(&input);
            }

            fn read_parameters(&mut self, rdr: &mut BufReader<File>) -> io::Result<()> {
                // println!("wat NNInput");
                Ok(())
            }
        }

        impl<const OS: usize> NNInput<OS> {
            pub fn new() -> Self {
                Self {
                    buf:  [0; OS],
                }
            }
        }

    }

    /// Affine
    mod nn_affine {
        use super::*;
        use num_traits::Num;

        #[derive(Debug,PartialEq,Clone)]
        pub struct NNAffine<Prev: NNLayer, const OS: usize> {
            prev:    Prev,
            // biases:  Array2<u8>,
            // weights: Array2<u8>,
            biases:  [i32; OS],
            // biases:  [u8; OS],
            weights: Vec<u8>,
        }

        /// Consts
        impl<Prev: NNLayer, const OS: usize> NNAffine<Prev, OS> {

            /// AVX2
            const INPUT_SIMD_WIDTH: usize = 32;
            const MAX_NUM_OUTPUT_REGS: usize = 8;

            // /// AVX
            // const INPUT_SIMD_WIDTH: usize = 16;
            // const MAX_NUM_OUTPUT_REGS: usize = 8;

            const NUM_OUTPUT_REGS: usize  = if OS > Self::MAX_NUM_OUTPUT_REGS {
                Self::MAX_NUM_OUTPUT_REGS } else { OS };
            const SMALL_BLOCK_SIZE: usize = Self::INPUT_SIMD_WIDTH;
            const BIG_BLOCK_SIZE: usize   = Self::NUM_OUTPUT_REGS * Self::SIZE_INPUT_PADDED;

            const NUM_SMALL_BLOCKS_PER_BIG_BLOCK: usize = Self::BIG_BLOCK_SIZE / Self::SMALL_BLOCK_SIZE;
            const NUM_SMALL_BLOCKS_PER_OUTPUT: usize = Self::SIZE_INPUT_PADDED / Self::SMALL_BLOCK_SIZE;

            const SIZE_INPUT_PADDED: usize = ceil_to_multiple(Self::SIZE_INPUT, 32);

        }

        impl<Prev: NNLayer, const OS: usize> NNAffine<Prev, OS> {

            fn get_weight_index(idx: usize) -> usize {

                // const IndexType smallBlock = (i / SmallBlockSize) % NumSmallBlocksInBigBlock;

                let small_block = (idx / Self::SMALL_BLOCK_SIZE)
                    % Self::NUM_SMALL_BLOCKS_PER_BIG_BLOCK;

                // const IndexType smallBlockCol = smallBlock / NumSmallBlocksPerOutput;
                let small_block_col = small_block / Self::NUM_SMALL_BLOCKS_PER_OUTPUT;

                // const IndexType smallBlockRow = smallBlock % NumSmallBlocksPerOutput;
                let small_block_row = small_block % Self::NUM_SMALL_BLOCKS_PER_OUTPUT;

                // const IndexType bigBlock   = i / BigBlockSize;
                let big_block = idx / Self::BIG_BLOCK_SIZE;

                // const IndexType rest       = i % SmallBlockSize;
                let rest      = idx % Self::SMALL_BLOCK_SIZE;

                big_block * Self::BIG_BLOCK_SIZE
                    + small_block_row * Self::SMALL_BLOCK_SIZE * Self::NUM_OUTPUT_REGS
                    + small_block_col * Self::SMALL_BLOCK_SIZE
                    + rest
            }

            pub fn new(prev: Prev) -> Self {
                Self {
                    prev,
                    biases:  [0; OS],
                    weights: vec![0; OS * Self::SIZE_INPUT],
                }
            }

        }

        impl<Prev: NNLayer, const OS: usize> NNLayer for NNAffine<Prev, OS> {
            type InputType = Prev::OutputType;
            type OutputType = u8;
            const SIZE_OUTPUT: usize = OS;
            const SIZE_INPUT: usize = Prev::SIZE_OUTPUT;

            const SELF_BUFFER_SIZE: usize =
                ceil_to_multiple(Self::SIZE_OUTPUT * std::mem::size_of::<Self::OutputType>(), CACHE_LINE_SIZE);

            const HASH: u32 = {
                let mut hash = 0xCC03DAE4;
                hash += Self::SIZE_OUTPUT as u32;
                hash ^= Prev::HASH.overflowing_shr(1).0;
                hash ^= Prev::HASH.overflowing_shl(31).0;
                hash
            };

            fn size(&self) -> usize {
                self.prev.size()
                    + self.biases.len() * std::mem::size_of_val(&self.biases[0])
                    + self.weights.len() * std::mem::size_of_val(&self.weights[0])
            }

            fn propogate(&self, input: &[u8], output: &mut [Self::OutputType]) {

                // assert_eq!(input.len(), Self::SELF_BUFFER_SIZE);
                // assert_eq!(output.len(), Self::SELF_BUFFER_SIZE);
                // assert_eq!(output.len(), Self::SIZE_OUTPUT);

                for i in 0..Self::SIZE_OUTPUT {
                    let offset = i * Self::SIZE_INPUT_PADDED;

                    let mut sum = self.biases[i];

                    for j in 0..Self::SIZE_INPUT {
                        sum += self.weights[offset + j] as i32 * input[j] as i32;
                    }

                    output[i] = sum as u8;
                }
            }

            fn read_parameters(&mut self, mut rdr: &mut BufReader<File>) -> io::Result<()> {
                self.prev.read_parameters(&mut rdr)?;
                // println!("wat NNAffine, OS = {:?}", OS);

                for i in 0..Self::SIZE_OUTPUT {
                    let x = rdr.read_i32::<LittleEndian>()?;
                    // let x = rdr.read_u8()?;
                    self.biases[i] = x;
                }

                self.weights = vec![0; OS * Self::SIZE_INPUT_PADDED];

                for i in 0..Self::SIZE_INPUT_PADDED * Self::SIZE_OUTPUT {
                    // eprintln!("i = {:?}", i);
                    let x = rdr.read_u8()?;
                    self.weights[Self::get_weight_index(i)] = x;
                }

                Ok(())
            }

        }
    }

    /// Clipped Relu
    mod nn_relu {
        use super::*;
        use num_traits::{Num,Zero,NumCast,AsPrimitive};
        // use num_traits::{Num,Zero};

        #[derive(Debug,PartialEq,Clone,Copy)]
        pub struct NNClippedRelu<Prev: NNLayer> {
            prev:    Prev,
        }

        impl<Prev: NNLayer> NNClippedRelu<Prev> {

            const SIZE_OUTPUT_PADDED: usize = ceil_to_multiple(Self::SIZE_OUTPUT, 32);

            pub fn new(prev: Prev) -> Self {
                Self {
                    prev,
                }
            }
        }

        impl<Prev: NNLayer> NNLayer for NNClippedRelu<Prev> {
            type InputType = Prev::OutputType;
            type OutputType = u8;
            const SIZE_OUTPUT: usize = Prev::SIZE_OUTPUT;
            const SIZE_INPUT: usize  = Prev::SIZE_OUTPUT;

            const SELF_BUFFER_SIZE: usize =
                ceil_to_multiple(Self::SIZE_OUTPUT * std::mem::size_of::<Self::OutputType>(), CACHE_LINE_SIZE);

            const HASH: u32 = 0x538D24C7 + Prev::HASH;

            fn size(&self) -> usize { self.prev.size() }

            // fn propogate(&self, input: &[u8]) -> Vec<Self::OutputType> {
            fn propogate(&self, input: &[u8], output: &mut [Self::OutputType]) {

                assert_eq!(input.len(), Self::SELF_BUFFER_SIZE);
                // assert_eq!(output.len(), Self::SELF_BUFFER_SIZE);
                assert_eq!(output.len(), Self::SIZE_OUTPUT_PADDED);

                let mut input2 = vec![Self::InputType::zero(); Self::SIZE_INPUT];
                let mut input2: &mut [Self::InputType] = &mut input2[..];
                self.prev.propogate(&input, input2);

                // TODO: AVX2 magic

                let start = 0;

                for i in start..Self::SIZE_INPUT {
                    let x0 = input2[i] >> WEIGHT_SCALE_BITS;
                    let x1: Self::InputType = NumCast::from(127u8).unwrap();
                    let x2: Self::InputType = NumCast::from(0u8).unwrap();
                    let x = x2.max(x1.min(x0));
                    output[i] = NumCast::from(x).unwrap();
                }

                // Affine transform layers expect that there is at least
                // ceil_to_multiple(OutputDimensions, 32) initialized values.
                // We cannot do this in the affine transform because it requires
                // preallocating space here.
                for i in Self::SIZE_OUTPUT..Self::SIZE_OUTPUT_PADDED {
                    output[i] = Zero::zero();
                }
            }

            fn read_parameters(&mut self, mut rdr: &mut BufReader<File>) -> io::Result<()> {
                let out = self.prev.read_parameters(&mut rdr)?;
                // println!("wat NNRelu, Size = {:?}", Self::SIZE_INPUT);
                Ok(out)
            }
        }
    }

}

#[derive(Debug,PartialEq,Clone)]
pub struct NNUE4 {
    pub ft:      NNFeatureTrans,
    pub layers:  Vec<Layer3>,
}

/// Read from file
impl NNUE4 {

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
        eprintln!("version = {:#8x}", version);

        let hashvalue = rdr.read_u32::<LittleEndian>()?;
        eprintln!("hashvalue = {:#8x}", hashvalue);

        let size      = rdr.read_u32::<LittleEndian>()?;
        eprintln!("size = {:?}", size);

        let mut desc = vec![0u8; size as usize];
        rdr.read_exact(&mut desc)?;
        let desc = String::from_utf8_lossy(&desc);
        eprintln!("desc = {:?}", desc);

        let mut ft = NNFeatureTrans::new();
        ft.read_parameters(&mut rdr)?;

        let mut layers: Vec<Layer3> = {
            let layer0 = Layer0::new();
            let layer1 = Layer1::new(NNAffine::new(layer0));
            let layer2 = Layer2::new(NNAffine::new(layer1));
            let layer3 = Layer3::new(layer2);

            vec![layer3.clone(); 8]
            // vec![NNFeatureTrans::new(layer3); 8]
        };

        for (n,mut layer) in layers.iter_mut().enumerate() {
            // eprintln!("layer = {:?}", n);
            layer.read_parameters(&mut rdr)?;
        }

        // let mut r = rdr.get_mut();
        // let mut xs = vec![];
        // let end = r.read_to_end(&mut xs)?;
        // eprintln!("end = {:?}", end);

        Ok((ft,layers))
    }

}

/// Init
impl NNUE4 {
    // pub fn init_inputs(&mut self, )
}

/// Index
impl NNUE4 {

    pub fn orient(king_sq: u8, persp: Color, sq: u8) -> u8 {
        let p = persp.fold(0, 1);

        const A8: u64 = 56;
        const H1: u64 = 7;

        let x = if Coord::from(king_sq).file() < 4 { 1 } else { 0 };

        let out = sq as u64 ^ (p * A8) ^ (x * H1);

        out as u8
    }

    pub fn make_index_half_ka_v2(king_sq: u8, persp: Color, pc: Piece, side: Color, sq: u8) -> usize {
        let o_king_sq = orient(king_sq, persp, king_sq);
        let pidx = PIECE_SQ_INDEX[side][persp][pc.index() + 1];

        let pc_nb = KING_BUCKETS[o_king_sq as usize];
        // assert!(pc_nb > 0);
        let pc_nb = PS_NB * pc_nb as usize;

        orient(king_sq, persp, sq) as usize
            + pidx
            + pc_nb
    }

}

