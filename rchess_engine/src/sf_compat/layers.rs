
pub use self::nn_input::NNInput;
pub use self::nn_affine::NNAffine;
pub use self::nn_relu::NNClippedRelu;

use std::io::{self, Read,BufReader, BufWriter};
use std::fs::File;
use std::path::Path;

use arrayvec::ArrayVec;
use byteorder::{ReadBytesExt, LittleEndian};

use num_traits::{Num,PrimInt,NumCast,AsPrimitive};

const CACHE_LINE_SIZE: usize = 64;
const WEIGHT_SCALE_BITS: u32 = 6;

pub trait NNLayer {
    // type InputType: Num + Copy + std::ops::Shr;
    // type OutputType: Num + Copy + std::ops::Shr;
    // type InputType: PrimInt + NumCast;
    // type OutputType: PrimInt + NumCast;
    type InputType: PrimInt + NumCast + AsPrimitive<i32>;
    type OutputType: PrimInt + NumCast + AsPrimitive<i32>;

    const SIZE_OUTPUT: usize;
    const SIZE_INPUT: usize;

    const SELF_BUFFER_SIZE: usize;
    const BUFFER_SIZE: usize;

    const HASH: u32;

    // fn propagate(&mut self, trans_features: &[u8], output: &mut [Self::OutputType]);
    fn propagate(&mut self, trans_features: &[u8]);
    // fn propagate(&self, trans_features: &[u8]) -> Vec<Self::OutputType>;
    // fn propagate(&self, trans_features: &[u8]) -> ArrayVec<Self::OutputType, {Self::BUFFER_SIZE}>;

    fn size(&self) -> usize;

    fn get_buf(&self) -> &[Self::OutputType];
    fn get_buf_mut(&mut self) -> &mut [Self::OutputType];

    fn read_parameters(&mut self, rdr: &mut BufReader<File>) -> io::Result<()> {
        Ok(())
    }

    fn write_parameters(&self, w: &mut BufWriter<File>) -> io::Result<()> {
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

    // #[derive(Debug,PartialEq,Clone,Copy)]
    #[derive(Debug,Eq,PartialEq,PartialOrd,Ord,Clone)]
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

        fn get_buf(&self) -> &[Self::OutputType] {
            &self.buf
        }
        fn get_buf_mut(&mut self) -> &mut [Self::OutputType] {
            &mut self.buf
        }

        // fn propagate(&mut self, trans_features: &[u8], output: &mut [Self::OutputType]) {
        fn propagate(&mut self, trans_features: &[u8]) {
        // fn propagate(&self, trans_features: &[u8]) -> Vec<Self::OutputType> {
            // assert!(input.len() == output.len());
            assert_eq!(trans_features.len(), Self::SELF_BUFFER_SIZE);

            // trans_features.to_vec()

            // assert_eq!(output.len(), Self::SELF_BUFFER_SIZE);
            // output.copy_from_slice(&trans_features);
            self.buf.copy_from_slice(&trans_features);
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
    use byteorder::WriteBytesExt;
    use num_traits::{Num,Zero};

    // #[derive(Debug,PartialEq,Clone)]
    #[derive(Debug,Eq,PartialEq,PartialOrd,Ord,Clone)]
    pub struct NNAffine<Prev: NNLayer, const OS: usize> {
        pub prev:    Prev,
        // biases:  Array2<u8>,
        // weights: Array2<u8>,
        pub biases:  [i32; OS],
        // biases:  [u8; OS],
        pub weights: Vec<i8>,

        // pub buffer:  [i32; OS],
        pub buffer:  [<NNAffine<Prev,OS> as NNLayer>::OutputType; OS],
    }

    /// Consts
    impl<Prev: NNLayer, const OS: usize> NNAffine<Prev, OS> {

        // /// AVX2
        // const INPUT_SIMD_WIDTH: usize = 32;
        // const MAX_NUM_OUTPUT_REGS: usize = 8;

        // /// AVX
        // const INPUT_SIMD_WIDTH: usize = 16;
        // const MAX_NUM_OUTPUT_REGS: usize = 8;

        const INPUT_SIMD_WIDTH: usize = 1;
        const MAX_NUM_OUTPUT_REGS: usize = 1;

        const NUM_OUTPUT_REGS: usize  = if OS > Self::MAX_NUM_OUTPUT_REGS {
            Self::MAX_NUM_OUTPUT_REGS } else { OS };
        const SMALL_BLOCK_SIZE: usize = Self::INPUT_SIMD_WIDTH;
        const BIG_BLOCK_SIZE: usize   = Self::NUM_OUTPUT_REGS * Self::SIZE_INPUT_PADDED;

        const NUM_SMALL_BLOCKS_PER_BIG_BLOCK: usize = Self::BIG_BLOCK_SIZE / Self::SMALL_BLOCK_SIZE;
        const NUM_SMALL_BLOCKS_PER_OUTPUT: usize = Self::SIZE_INPUT_PADDED / Self::SMALL_BLOCK_SIZE;

        const SIZE_INPUT_PADDED: usize = ceil_to_multiple(Self::SIZE_INPUT, 32);

    }

    impl<Prev: NNLayer, const OS: usize> NNAffine<Prev, OS> {

        fn _get_weight_index(idx: usize) -> usize {
            (idx / 4) % (Self::SIZE_INPUT_PADDED / 4) * (Self::SIZE_OUTPUT * 4)
                + idx / Self::SIZE_INPUT_PADDED * 4
                + idx % 4
        }

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
                // buffer:  [Self::OutputType::zero(); OS]
                buffer:  [0; OS]
            }
        }

    }

    impl<Prev: NNLayer, const OS: usize> NNLayer for NNAffine<Prev, OS> {
        type InputType = Prev::OutputType;
        type OutputType = i32;
        const SIZE_OUTPUT: usize = OS;
        const SIZE_INPUT: usize = Prev::SIZE_OUTPUT;

        // static_assert(std::is_same<InputType, std::uint8_t>::value, "");
        // static_assert(std::is_same<InputType, std::uint8_t>::value, "");

        const SELF_BUFFER_SIZE: usize = // 64
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

        fn get_buf(&self) -> &[Self::OutputType] {
            &self.buffer
            // unimplemented!()
        }
        fn get_buf_mut(&mut self) -> &mut [Self::OutputType] {
            &mut self.buffer
            // unimplemented!()
        }

        // fn propagate(&mut self, trans_features: &[u8], mut output: &mut [Self::OutputType]) {
        fn propagate(&mut self, trans_features: &[u8]) {
        // fn propagate(&self, trans_features: &[u8]) -> Vec<Self::OutputType> {

            // eprintln!("affine propagate");
            // eprintln!("NNAffine InputType = {:?}", std::any::type_name::<Self::InputType>());

            // let mut input: [Self::InputType; Self::SIZE_INPUT_PADDED] =
            //     [Self::InputType::zero(); Self::SIZE_INPUT_PADDED];
            // let mut input = ArrayVec::new
            // let input2: ArrayVec<Self::InputType, {Self::SIZE_INPUT_PADDED}> = ArrayVec::new();

            // let mut input = [Self::InputType::zero(); Self::SIZE_INPUT_PADDED];
            // let mut input = [Self::InputType::zero(); OS];

            // eprintln!("OS = {:?}", OS);
            // eprintln!("Self::SIZE_INPUT_PADDED = {:?}", Self::SIZE_INPUT_PADDED);

            // let mut input: Vec<Self::InputType> = vec![Self::InputType::zero(); Self::SIZE_INPUT_PADDED];
            // self.prev.propagate(trans_features, &mut input);

            // self.prev.propagate(trans_features, &mut input);

            self.prev.propagate(trans_features);
            let input = self.prev.get_buf();

            // let input: Vec<Self::InputType> = self.prev.propagate(trans_features);
            // assert_eq!(input.len(), Self::SIZE_INPUT_PADDED);

            // let mut output = vec![0; Self::SIZE_OUTPUT];

            let x0 = self.weights[0];
            let x1 = self.weights[Self::SIZE_INPUT_PADDED * (Self::SIZE_OUTPUT - 1) + Self::SIZE_INPUT - 1];

            for i in 0..Self::SIZE_OUTPUT {

                let offset = i * Self::SIZE_INPUT_PADDED;

                let mut sum: i32 = self.biases[i];

                for j in 0..Self::SIZE_INPUT {
                    // let x: i32 = input[j].as_();
                    let x: i32 = input[j].as_();
                    let x0 = self.weights[offset + j] as i32 * x;
                    sum += x0;
                }

                // output[i] = sum as Self::OutputType;
                // self.buffer[i] = sum as Self::OutputType;
                self.buffer[i] = sum as i32;
            }

            // output
        }

        fn read_parameters(&mut self, mut rdr: &mut BufReader<File>) -> io::Result<()> {
            self.prev.read_parameters(&mut rdr)?;
            // println!("wat NNAffine, OS = {:?}", OS);

            // eprintln!("Affine Read");
            // eprintln!("Self::SIZE_INPUT = {:?}", Self::SIZE_INPUT);
            // eprintln!("Self::SIZE_OUTPUT = {:?}", Self::SIZE_OUTPUT);
            // eprintln!("Self::SIZE_INPUT_PADDED = {:?}", Self::SIZE_INPUT_PADDED);

            for i in 0..Self::SIZE_OUTPUT {
                let x = rdr.read_i32::<LittleEndian>()?;
                // let x = rdr.read_u8()?;
                self.biases[i] = x;
            }

            let size = Self::SIZE_INPUT_PADDED * Self::SIZE_OUTPUT;
            self.weights = vec![0; size];

            for i in 0..size {
                // eprintln!("i = {:?}", i);
                let x = rdr.read_i8()?;
                // self.weights[Self::get_weight_index(i)] = x;
                self.weights[i] = x;
            }

            Ok(())
        }

        fn write_parameters(&self, mut w: &mut BufWriter<File>) -> io::Result<()> {
            self.prev.write_parameters(&mut w)?;
            for b in self.biases.iter() {
                w.write_i32::<LittleEndian>(*b)?;
            }
            // for wt in self.weights.iter() {
            //     w.write_u8(*wt)?;
            // }
            for i in 0..Self::SIZE_OUTPUT * Self::SIZE_INPUT_PADDED {
                let wt = self.weights[Self::get_weight_index(i)];
                w.write_i8(wt)?;
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

    // #[derive(Debug,PartialEq,Clone,Copy)]
    #[derive(Debug,Eq,PartialEq,PartialOrd,Ord,Clone)]
    pub struct NNClippedRelu<Prev: NNLayer, const OS: usize> {
        prev:    Prev,
        buf:     [u8; OS],
    }

    impl<Prev: NNLayer, const OS: usize> NNClippedRelu<Prev, OS> {

        const SIZE_OUTPUT_PADDED: usize = ceil_to_multiple(Self::SIZE_OUTPUT, 32);

        pub fn new(prev: Prev) -> Self {
            Self {
                prev,
                buf:  [0; OS],
            }
        }
    }

    impl<Prev: NNLayer, const OS: usize> NNLayer for NNClippedRelu<Prev, OS> {
        type InputType = Prev::OutputType;
        type OutputType = u8;
        const SIZE_OUTPUT: usize = Prev::SIZE_OUTPUT;
        const SIZE_INPUT: usize  = Prev::SIZE_OUTPUT;

        // static_assert(std::is_same<InputType, std::int32_t>::value, ""); 

        const SELF_BUFFER_SIZE: usize =
            ceil_to_multiple(Self::SIZE_OUTPUT * std::mem::size_of::<Self::OutputType>(), CACHE_LINE_SIZE);

        const BUFFER_SIZE: usize = Prev::BUFFER_SIZE + Self::SELF_BUFFER_SIZE;

        const HASH: u32 = 0x538D24C7 + Prev::HASH;

        fn size(&self) -> usize { self.prev.size() }

        fn get_buf(&self) -> &[Self::OutputType] {
            &self.buf
        }
        fn get_buf_mut(&mut self) -> &mut [Self::OutputType] {
            &mut self.buf
        }

        // fn propagate(&mut self, trans_features: &[u8], output: &mut [Self::OutputType]) {
        fn propagate(&mut self, trans_features: &[u8]) {
        // fn propagate(&self, trans_features: &[u8]) -> Vec<Self::OutputType> {

            // eprintln!("relu propagate");
            // eprintln!("NNRelu InputType = {:?}", std::any::type_name::<Self::InputType>());

            // let mut input: Vec<Self::InputType> = vec![Self::InputType::zero(); Self::SIZE_INPUT];
            // self.prev.propagate(trans_features, &mut input);

            // let input: Vec<Self::InputType> = self.prev.propagate(trans_features);
            // assert_eq!(input.len(), Self::SIZE_INPUT);

            self.prev.propagate(trans_features);
            // let mut output = vec![0; Self::SIZE_OUTPUT_PADDED];
            let input = self.prev.get_buf();

            // TODO: AVX2 magic

            let start = 0;

            for i in start..Self::SIZE_INPUT {
                // let x0: i32 = NumCast::from(input[i]).unwrap();

                let x0: i32 = input[i].as_();
                let x1 = (x0.overflowing_shr(WEIGHT_SCALE_BITS).0).clamp(0, 127);
                // output[i] = NumCast::from(x1).unwrap();
                // output[i] = x1.as_();
                self.buf[i] = x1.as_();
            }

            // // Affine transform layers expect that there is at least
            // // ceil_to_multiple(OutputDimensions, 32) initialized values.
            // // We cannot do this in the affine transform because it requires
            // // preallocating space here.
            // for i in Self::SIZE_OUTPUT..Self::SIZE_OUTPUT_PADDED {
            //     output[i] = Zero::zero();
            // }

            // output
        }

        fn read_parameters(&mut self, mut rdr: &mut BufReader<File>) -> io::Result<()> {
            let out = self.prev.read_parameters(&mut rdr)?;
            // println!("wat NNRelu, Size = {:?}", Self::SIZE_INPUT);
            Ok(out)
        }

        fn write_parameters(&self, mut w: &mut BufWriter<File>) -> io::Result<()> {
            self.prev.write_parameters(&mut w)?;
            Ok(())
        }

    }

}





