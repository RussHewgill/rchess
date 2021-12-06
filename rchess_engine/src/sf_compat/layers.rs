
pub use self::nn_input::NNInput;
pub use self::nn_affine::NNAffine;
pub use self::nn_relu::NNClippedRelu;

use std::io::{self, Read,BufReader};
use std::fs::File;
use std::path::Path;

use byteorder::{ReadBytesExt, LittleEndian};

use num_traits::{Num,PrimInt,NumCast};

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
    const BUFFER_SIZE: usize;

    const HASH: u32;

    // fn propogate(&self, input: &[u8]) -> Vec<Self::OutputType>;
    fn propagate(&self, input: &[u8], output: &mut [Self::OutputType]);

    fn size(&self) -> usize;

    fn read_parameters(&mut self, rdr: &mut BufReader<File>) -> io::Result<()> {
        Ok(())
    }

}

const fn ceil_to_multiple(n: usize, base: usize) -> usize {
    (n + base - 1) / base * base
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
        const BUFFER_SIZE: usize = 0;

        const HASH: u32 = 0xEC42E90D ^ Self::SIZE_OUTPUT as u32;

        fn size(&self) -> usize { self.buf.len() }

        // fn propogate(&self, input: &[u8]) -> Vec<Self::OutputType> {
        fn propagate(&self, input: &[u8], output: &mut [Self::OutputType]) {
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

        const BUFFER_SIZE: usize = Prev::BUFFER_SIZE + Self::SELF_BUFFER_SIZE;

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

        fn propagate(&self, input: &[u8], output: &mut [Self::OutputType]) {

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

        const BUFFER_SIZE: usize = Prev::BUFFER_SIZE + Self::SELF_BUFFER_SIZE;

        const HASH: u32 = 0x538D24C7 + Prev::HASH;

        fn size(&self) -> usize { self.prev.size() }

        // fn propogate(&self, input: &[u8]) -> Vec<Self::OutputType> {
        fn propagate(&self, input: &[u8], output: &mut [Self::OutputType]) {

            assert_eq!(input.len(), Self::SELF_BUFFER_SIZE);
            // assert_eq!(output.len(), Self::SELF_BUFFER_SIZE);
            assert_eq!(output.len(), Self::SIZE_OUTPUT_PADDED);

            let mut input2 = vec![Self::InputType::zero(); Self::SIZE_INPUT];
            let mut input2: &mut [Self::InputType] = &mut input2[..];
            self.prev.propagate(&input, input2);

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





