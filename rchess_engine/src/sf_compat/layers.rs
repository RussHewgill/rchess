
pub use self::nn_input::NNInput;
pub use self::nn_affine::NNAffine;
pub use self::nn_relu::NNClippedRelu;

use crate::eprint_self;

use std::io::{self, Read,BufReader, BufWriter};
use std::fs::File;
use std::path::Path;

// use ndarray as nd;
// use nd::{Array2,ShapeBuilder};

use arrayvec::ArrayVec;
use byteorder::{ReadBytesExt, LittleEndian};

use num_traits::{Num,PrimInt,NumCast,AsPrimitive};

use aligned::{Aligned,A64};

const CACHE_LINE_SIZE: usize = 64;
const WEIGHT_SCALE_BITS: u32 = 6;

const MAX_SIMD_WIDTH: usize = 32;

/// AVX2
pub const SIMD_WIDTH: usize = 32;

pub trait NNLayer {
    type InputType: PrimInt + NumCast + AsPrimitive<i32> + std::fmt::Debug;
    type OutputType: PrimInt + NumCast + AsPrimitive<i32> + std::fmt::Debug;

    const SIZE_OUTPUT: usize;
    const SIZE_INPUT: usize;

    const SELF_BUFFER_SIZE: usize;
    const BUFFER_SIZE: usize;

    const HASH: u32;

    // fn propagate(&mut self, trans_features: &[u8], output: &mut [Self::OutputType]);
    // fn propagate(&self, trans_features: &[u8]) -> Vec<Self::OutputType>;
    // fn propagate(&self, trans_features: &[u8]) -> ArrayVec<Self::OutputType, {Self::BUFFER_SIZE}>;
    // fn propagate(&mut self, trans_features: &[u8]);
    // fn propagate(&mut self, trans_features: &[u8]) -> &[Self::OutputType];
    fn propagate<'a>(&'a mut self, trans_features: &'a [u8]) -> &'a [Self::OutputType];

    // fn size(&self) -> usize;

    fn get_buf(&self) -> &[Self::OutputType];
    fn get_buf_mut(&mut self) -> &mut [Self::OutputType];

    fn read_parameters(&mut self, rdr: &mut BufReader<File>) -> io::Result<()> {
        Ok(())
    }

    fn write_parameters(&self, w: &mut BufWriter<File>) -> io::Result<()> {
        Ok(())
    }

}

pub const fn ceil_to_multiple(n: usize, base: usize) -> usize {
    (n + base - 1) / base * base
}

/// Input
mod nn_input {
    use super::*;
    use num_traits::Num;

    #[derive(Debug,Eq,PartialEq,PartialOrd,Ord,Clone)]
    pub struct NNInput<const OS: usize> {
        // buf:   [u8; OS],
        // buf:   Aligned<A64,[u8; OS]>,
    }

    impl<const OS: usize> NNLayer for NNInput<OS> {
        type InputType = u8;
        type OutputType = u8;
        const SIZE_OUTPUT: usize = OS;
        const SIZE_INPUT: usize = OS;

        const SELF_BUFFER_SIZE: usize = OS;
        const BUFFER_SIZE: usize = 0;

        const HASH: u32 = 0xEC42E90D ^ Self::SIZE_OUTPUT as u32;

        // fn size(&self) -> usize { self.buf.len() }

        fn get_buf(&self) -> &[Self::OutputType] {
            // self.buf.as_ref()
            unimplemented!()
        }
        fn get_buf_mut(&mut self) -> &mut [Self::OutputType] {
            // self.buf.as_mut()
            unimplemented!()
        }

        // fn propagate(&mut self, trans_features: &[u8], output: &mut [Self::OutputType]) {
        // fn propagate(&mut self, trans_features: &[u8]) {
        // fn propagate(&self, trans_features: &[u8]) -> Vec<Self::OutputType> {
        fn propagate<'a>(&'a mut self, trans_features: &'a [u8]) -> &'a [Self::OutputType] {
            // assert!(input.len() == output.len());
            // assert_eq!(trans_features.len(), Self::SELF_BUFFER_SIZE);
            // self.buf.copy_from_slice(trans_features);
            trans_features
            // self.buf.as_ref()
        }

        fn read_parameters(&mut self, rdr: &mut BufReader<File>) -> io::Result<()> {
            Ok(())
        }
    }

    impl<const OS: usize> NNInput<OS> {
        pub fn new() -> Self {
            Self {
                // buf:  Aligned([0; OS]),
            }
        }
    }

}

/// Affine
mod nn_affine {
    use crate::eprint_self;

    use super::*;
    use byteorder::WriteBytesExt;
    use num_traits::{Num,Zero};

    #[derive(Debug,Eq,PartialEq,Clone)]
    pub struct NNAffine<Prev: NNLayer, const OS: usize, const IS: usize> {
        pub prev:    Prev,

        // pub biases:  [i32; OS],
        pub biases:  Aligned<A64,[i32; OS]>,

        // pub weights: Vec<i8>, // IS * SIZE_INPUT
        pub weights: Aligned<A64,Vec<i8>>, // IS * SIZE_INPUT

        // pub buffer:  [<NNAffine<Prev,OS,IS> as NNLayer>::OutputType; OS],
        pub buffer:  Aligned<A64,[<NNAffine<Prev,OS,IS> as NNLayer>::OutputType; OS]>,
    }

    /// Consts, AVX2
    #[cfg(target_feature = "avx2")]
    impl<Prev: NNLayer, const OS: usize, const IS: usize> NNAffine<Prev,OS,IS> {
        const INPUT_SIMD_WIDTH: usize = 32;
        const MAX_NUM_OUTPUT_REGS: usize = 8;
    }

    #[cfg(all(not(target_feature = "avx2"), target_feature = "ssse3"))]
    impl<Prev: NNLayer, const OS: usize, const IS: usize> NNAffine<Prev,OS,IS> {
        const INPUT_SIMD_WIDTH: usize = 16;
        const MAX_NUM_OUTPUT_REGS: usize = 8;
    }

    #[cfg(all(not(target_feature = "avx2"), not(target_feature = "ssse3")))]
    // #[cfg(not(target_feature = "avx2"))]
    impl<Prev: NNLayer, const OS: usize, const IS: usize> NNAffine<Prev,OS,IS> {
        const INPUT_SIMD_WIDTH: usize = 1;
        const MAX_NUM_OUTPUT_REGS: usize = 1;
    }

    impl<Prev: NNLayer, const OS: usize, const IS: usize> NNAffine<Prev,OS,IS> {
        const NUM_OUTPUT_REGS: usize  = if OS > Self::MAX_NUM_OUTPUT_REGS {
            Self::MAX_NUM_OUTPUT_REGS } else { OS };
        const SMALL_BLOCK_SIZE: usize = Self::INPUT_SIMD_WIDTH;
        const BIG_BLOCK_SIZE: usize   = Self::NUM_OUTPUT_REGS * Self::SIZE_INPUT_PADDED;

        const NUM_SMALL_BLOCKS_PER_BIG_BLOCK: usize = Self::BIG_BLOCK_SIZE / Self::SMALL_BLOCK_SIZE;
        const NUM_SMALL_BLOCKS_PER_OUTPUT: usize = Self::SIZE_INPUT_PADDED / Self::SMALL_BLOCK_SIZE;

        const NUM_BIG_BLOCKS: usize = Self::SIZE_OUTPUT / Self::NUM_OUTPUT_REGS;

        const SIZE_INPUT_PADDED: usize = ceil_to_multiple(Self::SIZE_INPUT, 32);

    }

    impl<Prev: NNLayer, const OS: usize, const IS: usize> NNAffine<Prev,OS,IS> {

        #[cfg(all(not(target_feature = "avx2"), not(target_feature = "ssse3")))]
        pub fn get_weight_index(idx: usize) -> usize {
            idx
        }

        // XXX: fix
        #[cfg(any(target_feature = "avx2", target_feature = "ssse3"))]
        pub fn get_weight_index(idx: usize) -> usize {
            if Self::INPUT_SIMD_WIDTH == 1 {
                idx
            } else if Self::SIZE_INPUT_PADDED >= 128 {
                // #[cfg(target_feature = "ssse3")]
                // // return Self::_get_weight_index_scrambled(idx);
                // return idx;
                // #[cfg(not(target_feature = "ssse3"))]
                return Self::_get_weight_index(idx);
                // idx
            } else {
                // Self::_get_weight_index_scrambled(idx)
                idx
            }
        }

        pub fn _get_weight_index_scrambled(idx: usize) -> usize {
            (idx / 4) % (Self::SIZE_INPUT_PADDED / 4) * (Self::SIZE_OUTPUT * 4)
                + idx / Self::SIZE_INPUT_PADDED * 4
                + idx % 4
        }

        pub fn _get_weight_index(idx: usize) -> usize {

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

                biases:  Aligned([0; OS]),
                weights: Aligned(vec![0; OS * Self::SIZE_INPUT]),

                // buffer:  [Self::OutputType::zero(); OS]
                buffer:  Aligned([0; OS])
            }
        }

    }

    /// Approach 1:
    #[cfg(feature = "nope")]
    impl<Prev: NNLayer, const OS: usize, const IS: usize> NNAffine<Prev,OS,IS> {

        // const INPUT_SIMD_WIDTH: usize = 32; // AVX2
        // const MAX_NUM_OUTPUT_REGS: usize = 8; // AVX2

        /// Approach 1:
        ///   - used when the PaddedInputDimensions >= 128
        ///   - uses AVX512 if possible
        ///   - processes inputs in batches of 2*InputSimdWidth
        ///   - so in batches of 128 for AVX512
        ///   - the weight blocks of size InputSimdWidth are transposed such that
        ///     access is sequential
        ///   - N columns of the weight matrix are processed a time, where N
        ///     depends on the architecture (the amount of registers)
        ///   - accumulate + hadd is used
        #[cfg(feature = "nope")]
        pub fn _propagate_avx2(&mut self, trans_features: &[u8]) {
            use crate::simd_utils::x86_64::*;
            use crate::simd_utils::std_simd::*;

            // use std::simd::*;
            use core::arch::x86_64::*;

            // using acc_vec_t = __m256i;
            // using bias_vec_t = __m128i;
            // using weight_vec_t = __m256i;
            // using in_vec_t = __m256i;
            // #define vec_zero _mm256_setzero_si256()
            // #define vec_add_dpbusd_32x2 Simd::m256_add_dpbusd_epi32x2
            // #define vec_hadd Simd::m256_hadd
            // #define vec_haddx4 Simd::m256_haddx4


            self.prev.propagate(trans_features);
            let input: &[<NNAffine<Prev,OS> as NNLayer>::InputType] = self.prev.get_buf();
            // let input: &[u8] = self.prev.get_buf();

            assert!(Self::SIZE_OUTPUT % Self::NUM_OUTPUT_REGS == 0);
            assert!(input.len() % 32 == 0);

            // XXX: Safe until I change the Prev::InputType
            let input: &[u8] = unsafe {
                let ptr = input.as_ptr();
                let ptr2 = ptr as *const u8;
                std::slice::from_raw_parts(ptr2, input.len())
            };

            // type InVec = u8x32;

            use std::convert::TryInto;

            let ins: Vec<__m256i> = input.chunks_exact(32).map(|s| {
                build_m256i_from_slice(s)
            }).collect();

            for big_block in 0..Self::NUM_BIG_BLOCKS {
                let mut acc = vec![unsafe { _mm256_setzero_si256() }; Self::NUM_OUTPUT_REGS];

                let mut small_block = 0;
                while small_block < Self::NUM_SMALL_BLOCKS_PER_OUTPUT {

                    let in0: __m256i = ins[small_block + 0];
                    let in1: __m256i = ins[small_block + 1];

                    let offset = big_block * Self::BIG_BLOCK_SIZE
                        + small_block * Self::SMALL_BLOCK_SIZE * Self::NUM_OUTPUT_REGS;

                    for k in 0..Self::NUM_OUTPUT_REGS {

                        let s = &self.weights[offset + k..];
                        let s: &[u8] = unsafe { &*(s as *const _ as *const [u8]) };

                        let b0 = build_m256i_from_slice(s);
                        let b1 = build_m256i_from_slice(&s[Self::NUM_OUTPUT_REGS..]);

                        // let b0: i8x32 = i8x32::from_slice(&self.weights[offset + k..]);
                        // let b1: i8x32 = i8x32::from_slice(&self.weights[offset + k + Self::NUM_OUTPUT_REGS..]);

                        // let zero: __m256i = _mm256_setzero_si256();
                        // let offsets = _mm256_set_epi32(7, 3, 6, 2, 5, 1, 4, 0);

                        // __m256i product0 = _mm256_maddubs_epi16(a0, b0);
                        // let product0 = in0 * b0;

                        unsafe {
                            let mut product0 = _mm256_maddubs_epi16(in0, b0);
                            let product1 = _mm256_maddubs_epi16(in1, b1);

                            product0 = _mm256_adds_epi16(product0, product1);
                            product0 = _mm256_adds_epi16(product0, _mm256_set1_epi16(1));

                            acc[k] = _mm256_add_epi32(acc[k], product0);
                        };

                    }

                    small_block += 2;
                }

                if Self::NUM_OUTPUT_REGS % 4 == 0 {

                    // let ins: Vec<__m256i> = input.chunks_exact(32).map(|s| {
                    //     Self::_m256i_from_slice(s)
                    // }).collect();

                    // let output_vec: Vec<__m256i> = unimplemented!();
                    // let output_vec: Vec<__m256i> = output.

                    // let o_size = Self::SIZE_OUTPUT / 
                    // let output_vec: [__m128i; ]
                    let output_vec: Vec<__m128i> = vec![];

                    // let s: &[u8] = unsafe { &*(s as *const _ as *const [u8]) };

                    // let bias_vec: &[__m128] = unsafe {
                    //     &*(&self.weights as *const _ as *const [__m128])
                    // };

                    for k in 0..Self::NUM_OUTPUT_REGS {
                        let idx = (big_block * Self::NUM_OUTPUT_REGS + k) / 4;

                        let mut sum0 = acc[k+0];
                        let sum1 = acc[k+1];
                        let mut sum2 = acc[k+2];
                        let sum3 = acc[k+3];

                        unsafe {
                            sum0 = _mm256_hadd_epi32(sum0, sum1);
                            sum2 = _mm256_hadd_epi32(sum2, sum3);

                            sum0 = _mm256_hadd_epi32(sum0, sum2);

                            let sum128lo = _mm256_castsi256_si128(sum0);
                            let sum128hi = _mm256_extracti128_si256::<1>(sum0);

                            // let out = _mm_add_epi32(_mm_add_epi32(sum128lo, sum128hi), bias);
                            // output_vec[idx] = _mm_add_epi32(_mm_add_epi32(sum128lo, sum128hi), bias);
                            unimplemented!()
                        }

                        // m256_haddx4

                    }
                }

            }

            // let input = InVec::from_slice(&input);

            // let input

            unimplemented!()
        }

        #[cfg(feature = "nope")]
        pub fn _propagate_avx2_small(&mut self, trans_features: &[u8]) {
            self.prev.propagate(trans_features);
            let input = self.prev.get_buf();

            for i in 0..Self::SIZE_OUTPUT {
                let offset = i * Self::SIZE_INPUT_PADDED;
                let mut sum: i32 = self.biases[i];
                for (j,x) in input.iter().enumerate() {
                    let x: i32 = x.as_();
                    let x0 = self.weights[offset + j] as i32 * x;
                    sum += x0;
                }
                self.buffer[i] = sum as i32;
            }
        }

        #[cfg(feature = "nope")]
        pub fn _propagate_avx2_large(&mut self, trans_features: &[u8]) {
            self.prev.propagate(trans_features);
            let input = self.prev.get_buf();

            for i in 0..Self::SIZE_OUTPUT {
                let offset = i * Self::SIZE_INPUT_PADDED;
                let mut sum: i32 = self.biases[i];
                for (j,x) in input.iter().enumerate() {
                    let x: i32 = x.as_();
                    // let x0 = self.weights[offset + j] as i32 * x;
                    let x0 = self.weights[Self::get_weight_index(offset + j)] as i32 * x;
                    sum += x0;
                }
                self.buffer[i] = sum as i32;
            }
        }

        #[cfg(feature = "nope")]
        #[allow(unreachable_code)]
        pub fn _propagate_avx2_large(&mut self, trans_features: &[u8]) {
            self.prev.propagate(trans_features);
            let input = self.prev.get_buf();

            // assert!(input.len() == Self::SIZE_INPUT_PADDED);
            assert!(input.len() == Self::SIZE_INPUT);

            // Safe as long as InputType is u8
            let input2: &[u8] = unsafe {
                let ptr = input.as_ptr() as *const u8;
                std::slice::from_raw_parts(ptr, input.len())
            };

            use safe_arch::*;
            use crate::simd_utils::safe_arch::*;

            assert_eq!(input.len() % 32, 0);

            // let input_vec: Vec<m256i> = input2.array_chunks::<32>()
            //     .map(|&a| m256i::from(a))
            //     .collect();

            // XXX: Segfault?
            let input_vec: &[m256i] = unsafe {
                let ptr = input2.as_ptr();
                let ptr = ptr as *const i8 as *const m256i;
                std::slice::from_raw_parts(ptr, input.len() / 32)
            };

            // let k0 = input_vec[0];
            // eprintln!("k0 = {:?}", k0);

            // let in0 = input_vec[0];
            // let in1 = slice_to_m256i_u8(&input2[0..32]);

            // let k0 = extract_i8_as_i32_m256i::<0>(in0);
            // let k1 = extract_i8_as_i32_m256i::<1>(in0);
            // let k2 = extract_i8_as_i32_m256i::<2>(in0);
            // let k3 = extract_i8_as_i32_m256i::<3>(in0);

            // eprintln!("k0 = {:?}", k0);
            // eprintln!("k1 = {:?}", k1);
            // eprintln!("k2 = {:?}", k2);
            // eprintln!("k3 = {:?}", k3);

            // eprintln!("in0 = {:?}", bytemuck::cast::<m256i,[u8;32]>(in0));
            // eprintln!("in1 = {:?}", bytemuck::cast::<m256i,[u8;32]>(in1));

            let weight_vec: &[m256i] = unsafe {
                let ptr = self.weights.as_ptr() as *const m256i;
                std::slice::from_raw_parts(ptr, self.weights.len() / 32)
            };

            // eprint_self!(Self::NUM_BIG_BLOCKS); // 1
            // eprint_self!(Self::BIG_BLOCK_SIZE); // 16384
            // eprint_self!(Self::SMALL_BLOCK_SIZE); // 32
            // eprint_self!(Self::NUM_SMALL_BLOCKS_PER_BIG_BLOCK); // 512
            // eprint_self!(Self::NUM_SMALL_BLOCKS_PER_OUTPUT); // 64

            for big_block in 0..Self::NUM_BIG_BLOCKS {

                let mut acc = vec![m256i::default(); Self::NUM_OUTPUT_REGS];

                // for small_block in (0..Self::NUM_SMALL_BLOCKS_PER_OUTPUT/2).map(|x| x*2) {
                let mut small_block = 0;
                while small_block < Self::NUM_SMALL_BLOCKS_PER_OUTPUT {

                    let w_offset = big_block * Self::BIG_BLOCK_SIZE
                        + small_block * Self::SMALL_BLOCK_SIZE * Self::NUM_OUTPUT_REGS;
                    let w_offset = w_offset / 32;

                    let weight_vec = &weight_vec[w_offset..];

                    // eprintln!("w_offset = {:?}", w_offset);

                    let in0 = input_vec[small_block + 0];
                    let in1 = input_vec[small_block + 1];

                    // let wat = extract_m128i_m256i::<1>(in0);
                    // let wat = weight_vec[0];

                    // eprintln!("in0 = {:?}", bytemuck::cast::<m256i,[u8;32]>(wat));
                    // eprintln!("in0 = {:?}", bytemuck::cast::<m128i,[u8;16]>(wat));

                    for k in 0..Self::NUM_OUTPUT_REGS {
                        // let b0 = Self::slice_i8_to_m256i(&weight_vec[k..k+32]);
                        // let b1 = Self::slice_i8_to_m256i(
                        //     &weight_vec[k+Self::NUM_OUTPUT_REGS..k+Self::NUM_OUTPUT_REGS+32]);

                        let b0 = weight_vec[k];
                        let b1 = weight_vec[k + Self::NUM_OUTPUT_REGS];

                        m256_add_dpbusd_epi32x2(&mut acc[k], in0, b0, in1, b1)
                    }

                    small_block += 2
                }

                // let wat: &[i32] = unsafe {
                //     let ptr = acc.as_ptr();
                //     let ptr = ptr as *const i32;
                //     std::slice::from_raw_parts(ptr, acc.len() * 8)
                // };

                // for k in 0..64 {
                //     eprintln!("wat[{}] = {:?}", k, wat[k]);
                // }

                // for k in 0..8 {
                //     eprintln!("wat[{}] = {:?}", k, wat[k]);
                // }

                // let ks0: [i32; 8] = acc[0].into();
                // // let ks1: [i32; 8] = acc[1].into();
                // eprintln!("ks0 = {:?}", ks0);
                // // eprintln!("ks1 = {:?}", ks1);

                // let bias_vec: &[m128i] = unsafe {
                //     let ptr = self.biases.as_ptr();
                //     let ptr = ptr as *const m128i;
                //     std::slice::from_raw_parts(ptr, self.biases.len() / 4)
                // };

                // eprintln!("bias[0] = {:?}", bytemuck::cast::<m128i,[i32;4]>(bias_vec[0]));

                // let bias_vec: Vec<m128i> = self.biases.array_chunks::<4>()
                //     .map(|&x| m128i::from(x))
                //     .collect();

                // let output_vec: &mut [m128i] = unsafe {
                //     let ptr = self.buffer.as_mut_ptr();
                //     let ptr = ptr as *mut m128i;
                //     std::slice::from_raw_parts_mut(ptr, self.buffer.len() / 4)
                // };

                // for k in (0..Self::NUM_OUTPUT_REGS/4).map(|x| x * 4) {
                //     let idx = (big_block * Self::NUM_OUTPUT_REGS + k) / 4;
                //     output_vec[idx] = m256_haddx4(acc[k+0],acc[k+1],acc[k+2],acc[k+2],bias_vec[idx]);
                // }

                for k in 0..Self::NUM_OUTPUT_REGS {
                    let idx = big_block * Self::NUM_OUTPUT_REGS + k;
                    self.buffer[idx] = m256_hadd(acc[k], self.biases[idx]);
                }

            }

            // for k in 0..8 {
            //     eprintln!("self.buffer[{}] = {:?}", k, self.buffer[k]);
            // }

            // eprintln!("input.len() = {:?}", input.len());
            // eprintln!("input2.len() = {:?}", input2.len());

            // for i in 0..Self::SIZE_OUTPUT {
            //     let offset = i * Self::SIZE_INPUT_PADDED;
            //     let mut sum: i32 = self.biases[i];
            //     for (j,x) in input.iter().enumerate() {
            //         let x: i32 = x.as_();
            //         let x0 = self.weights[offset + j] as i32 * x;
            //         sum += x0;
            //     }
            //     self.buffer[i] = sum as i32;
            // }

        }

        #[cfg(feature = "nope")]
        pub fn _propagate_avx2_small<'a>(
            &'a mut self, trans_features: &'a [u8]
        ) -> &'a [<NNAffine<Prev,IS,OS> as NNLayer>::OutputType] {

            // eprintln!("affine propagate");
            // eprintln!("NNAffine InputType = {:?}", std::any::type_name::<Self::InputType>());

            let input = self.prev.propagate(trans_features);
            let input = &input[..Self::SIZE_INPUT];

            let input: &[u8] = unsafe {
                let ptr = input.as_ptr();
                let ptr2 = ptr as *const u8;
                std::slice::from_raw_parts(ptr2, input.len())
            };

            assert_eq!(self.biases.len(), Self::SIZE_OUTPUT);
            assert_eq!(self.buffer.len(), Self::SIZE_OUTPUT);
            // eprintln!("Self::SIZE_OUTPUT = {:?}", Self::SIZE_OUTPUT);
            // assert_eq!(Self::SIZE_OUTPUT % 4, 0);

            assert_eq!(IS, input.len());

            eprint_self!(Self::SIZE_INPUT_PADDED);
            eprint_self!(Self::SIZE_INPUT);
            eprint_self!(Self::SIZE_OUTPUT);

            eprintln!("self.weights.len() = {:?}", self.weights.len());

            for i in 0..Self::SIZE_OUTPUT {

                let offset = i * Self::SIZE_INPUT_PADDED;

                let weights = &self.weights[offset..offset + IS];
                // let weights = &self.weights[offset..];
                let mut sum: i32 = unsafe { *self.biases.get_unchecked(i) };

                for (k,x) in input.iter().enumerate() {
                    let x: i32 = x.as_();
                    let x0 = weights[k] as i32 * x;
                    sum += x0;
                }

                self.buffer[i] = sum as i32;
            }

            // if self.buffer.len() == 8 {
            //     eprintln!("self.buffer = {:?}", &self.buffer);
            // }

            self.buffer.as_ref()
        }

        #[cfg(feature = "nope")]
        pub fn _propagate_avx2_small<'a>(
            &'a mut self, trans_features: &'a [u8]
        ) -> &'a [<NNAffine<Prev,IS,OS> as NNLayer>::OutputType] {
            use safe_arch::*;
            use crate::simd_utils::safe_arch::*;

            let input = self.prev.propagate(trans_features);
            let input = &input[..Self::SIZE_INPUT]; // TODO: bench

            eprintln!("Self::SIZE_INPUT = {:?}", Self::SIZE_INPUT);
            eprintln!("Self::SIZE_OUTPUT = {:?}", Self::SIZE_OUTPUT);
            eprintln!("self.weights.len() = {:?}", self.weights.len());

            eprintln!("IS * OS = {:?}", IS * OS);

            for i in 0..Self::SIZE_OUTPUT {
                let offset = i * Self::SIZE_INPUT_PADDED;
                let mut sum: i32 = self.biases[i];
                for (j,x) in input.iter().enumerate() {
                    let x: i32 = x.as_();
                    let x0 = self.weights[offset + j] as i32 * x;
                    sum += x0;
                }
                self.buffer[i] = sum as i32;
            }

            self.buffer.as_ref()
        }

    }

    /// AVX2
    #[cfg(target_feature = "avx2")]
    impl<Prev: NNLayer, const OS: usize, const IS: usize> NNAffine<Prev,OS,IS> {

        // #[cfg(feature = "nope")]
        /// TODO: when SIZE_OUTPUT % OUTPUTSIMD_WIDTH == 0
        pub fn _propagate_avx2_small<'a>(
            &'a mut self, trans_features: &'a [u8]
        ) -> &'a [<NNAffine<Prev,IS,OS> as NNLayer>::OutputType] {
            use safe_arch::*;
            use crate::simd_utils::safe_arch::*;

            let input = self.prev.propagate(trans_features);
            let input = &input[..Self::SIZE_INPUT]; // TODO: bench

            const OUTPUT_SIMD_WIDTH: usize = 32 / 4;
            const INPUT_SIMD_WIDTH: usize = 32;

            assert_eq!(Self::SIZE_INPUT % 8, 0);
            assert!(Self::SIZE_OUTPUT % OUTPUT_SIMD_WIDTH == 0 || Self::SIZE_OUTPUT == 1);

            // let input: &[u8] = unsafe {
            //     let ptr = input.as_ptr() as *const u8;
            //     std::slice::from_raw_parts(ptr, input.len())
            // };

            // if Self::SIZE_OUTPUT % OUTPUT_SIMD_WIDTH == 0 {
            if false {
                let num_chunks: usize = IS / 4;
                let num_regs: usize = Self::SIZE_OUTPUT / OUTPUT_SIMD_WIDTH;

                let input32: &[i32] = unsafe {
                    let ptr = input.as_ptr() as *const i32;
                    std::slice::from_raw_parts(ptr, input.len() / 4)
                };

                let bias_vec: &[m256i] = unsafe {
                    let ptr = self.biases.as_ptr() as *const m256i;
                    std::slice::from_raw_parts(
                        ptr, self.biases.len() * std::mem::size_of::<i32>() / 32)
                };

                let mut acc = vec![m256i::default(); num_regs];

                // for k in 0..num_regs {
                //     acc[k] = bias_vec[k];
                // }
                acc[..num_regs].copy_from_slice(&bias_vec[..num_regs]);

                let mut i = 0;
                while i < num_chunks {
                    let in0 = set_splat_i32_m256i(input32[i + 0]);
                    let in1 = set_splat_i32_m256i(input32[i + 1]);

                    let col0 = &self.weights[(i+0) * Self::SIZE_OUTPUT * 4..];
                    let col1 = &self.weights[(i+1) * Self::SIZE_OUTPUT * 4..];

                    let col0: &[m256i] = unsafe {
                        let ptr = col0.as_ptr() as *const m256i;
                        std::slice::from_raw_parts(ptr, col0.len() / 32)
                    };
                    let col1: &[m256i] = unsafe {
                        let ptr = col1.as_ptr() as *const m256i;
                        std::slice::from_raw_parts(ptr, col1.len() / 32)
                    };

                    for k in 0..num_regs {
                        m256_add_dpbusd_epi32x2(&mut acc[k], in0, col0[k], in1, col1[k]);
                    }

                    let output_vec: &mut [m256i] = unsafe {
                        let ptr = self.buffer.as_mut_ptr() as *mut m256i;
                            std::slice::from_raw_parts_mut(
                                ptr, self.buffer.len() * std::mem::size_of::<i32>() / 32)
                    };

                    // for k in 0..num_regs {
                    //     output_vec[k] = acc[k];
                    // }
                    output_vec[..num_regs].copy_from_slice(&acc[..num_regs]);

                    i += 2;
                }

            } else if Self::SIZE_OUTPUT == 1 {
            // } else {
                let num_chunks: usize = Self::SIZE_INPUT_PADDED / 32;

                let input_vec: &[m256i] = unsafe {
                    let ptr = input.as_ptr() as *const m256i;
                    std::slice::from_raw_parts(ptr, input.len() / 32)
                };

                let row0: &[m256i] = unsafe {
                    let ptr = self.weights.as_ptr() as *const m256i;
                    std::slice::from_raw_parts(ptr, self.weights.len() / 32)
                };

                let mut sum = m256i::default();

                for k in 0..num_chunks {
                    let in0 = input_vec[k];
                    m256_add_dpbusd_epi32(&mut sum, in0, row0[k]);
                }

                self.buffer[0] = m256_hadd(sum, self.biases[0]);
            } else {
                for i in 0..Self::SIZE_OUTPUT {
                    let offset = i * Self::SIZE_INPUT_PADDED;
                    let mut sum: i32 = self.biases[i];
                    for (j,x) in input.iter().enumerate() {
                        let x: i32 = x.as_();
                        let x0 = self.weights[offset + j] as i32 * x;
                        sum += x0;
                    }
                    self.buffer[i] = sum as i32;
                }
                // panic!();
            }

            self.buffer.as_ref()
        }

        #[cfg(target_feature = "avx2")]
        pub fn _propagate_avx2_large<'a>(
            &'a mut self, trans_features: &'a [u8]
        ) -> &'a [<NNAffine<Prev,IS,OS> as NNLayer>::OutputType] {
            use safe_arch::*;
            use crate::simd_utils::safe_arch::*;

            let input = self.prev.propagate(trans_features);
            assert_eq!(input.len() % 32, 0);

            let input_vec: &[m256i] = unsafe {
                let ptr = input.as_ptr() as *const m256i;
                std::slice::from_raw_parts(ptr, input.len() / 32)
            };

            let weight_vec: &[m256i] = unsafe {
                let ptr = self.weights.as_ptr() as *const m256i;
                std::slice::from_raw_parts(ptr, self.weights.len() / 32)
            };

            for big_block in 0..Self::NUM_BIG_BLOCKS {

                // let mut acc = [m256i::default(); Self::NUM_OUTPUT_REGS];
                // let mut acc = vec![m256i::default(); Self::NUM_OUTPUT_REGS];
                let mut acc = [m256i::default(); OS];

                let mut small_block = 0;
                while small_block < Self::NUM_SMALL_BLOCKS_PER_OUTPUT {

                    let w_offset = big_block * Self::BIG_BLOCK_SIZE
                        + small_block * Self::SMALL_BLOCK_SIZE * Self::NUM_OUTPUT_REGS;
                    let w_offset = w_offset / 32;

                    let weight_vec = &weight_vec[w_offset..];

                    let in0 = input_vec[small_block + 0];
                    let in1 = input_vec[small_block + 1];

                    for k in 0..Self::NUM_OUTPUT_REGS {
                        let b0 = weight_vec[k];
                        let b1 = weight_vec[k + Self::NUM_OUTPUT_REGS];
                        m256_add_dpbusd_epi32x2(&mut acc[k], in0, b0, in1, b1)
                    }

                    small_block += 2
                }

                let bias_vec: &[m128i] = unsafe {
                    let ptr = self.biases.as_ptr() as *const m128i;
                    std::slice::from_raw_parts(ptr, self.biases.len() / 4)
                };

                let output_vec: &mut [m128i] = unsafe {
                    let ptr = self.buffer.as_mut_ptr() as *mut m128i;
                    std::slice::from_raw_parts_mut(ptr, self.buffer.len() / 4)
                };

                let mut k = 0;
                while k < Self::NUM_OUTPUT_REGS {
                    let idx = (big_block * Self::NUM_OUTPUT_REGS + k) / 4;
                    output_vec[idx] = m256_haddx4(acc[k+0],acc[k+1],acc[k+2],acc[k+3],bias_vec[idx]);
                    k += 4;
                }

                // for k in 0..Self::NUM_OUTPUT_REGS {
                //     let idx = big_block * Self::NUM_OUTPUT_REGS + k;
                //     self.buffer[idx] = m256_hadd(acc[k], self.biases[idx]);
                // }

            }

            self.buffer.as_ref()
        }

    }

    /// SSSE3
    #[cfg(all(not(target_feature = "avx2"), target_feature = "ssse3"))]
    impl<Prev: NNLayer, const OS: usize, const IS: usize> NNAffine<Prev,OS,IS> {

        pub fn _propagate_avx2_small<'a>(
            &'a mut self, trans_features: &'a [u8]
        ) -> &'a [<NNAffine<Prev,IS,OS> as NNLayer>::OutputType] {
            self._propagate_nosimd(trans_features)
        }

        pub fn _propagate_avx2_large<'a>(
            &'a mut self, trans_features: &'a [u8]
        ) -> &'a [<NNAffine<Prev,IS,OS> as NNLayer>::OutputType] {
            use safe_arch::*;
            use crate::simd_utils::safe_arch::*;

            let input = self.prev.propagate(trans_features);
            assert_eq!(input.len() % 32, 0);

            let input_vec: &[m128i] = unsafe {
                cast_slice_to_m128i(&input)
            };

            let weight_vec: &[m128i] = unsafe {
                cast_slice_to_m128i(&self.weights)
            };

            for big_block in 0..Self::NUM_BIG_BLOCKS {
                let mut acc = [m128i::default(); OS];

                let mut small_block = 0;
                while small_block < Self::NUM_SMALL_BLOCKS_PER_OUTPUT {

                    let w_offset = big_block * Self::BIG_BLOCK_SIZE
                        + small_block * Self::SMALL_BLOCK_SIZE * Self::NUM_OUTPUT_REGS;

                    /// XXX: 16 or 32 and why?
                    // let w_offset = w_offset / 32;
                    let w_offset = w_offset / 16;

                    let weight_vec = &weight_vec[w_offset..];

                    let in0 = input_vec[small_block + 0];
                    let in1 = input_vec[small_block + 1];

                    for k in 0..Self::NUM_OUTPUT_REGS {
                        let b0 = weight_vec[k];
                        let b1 = weight_vec[k + Self::NUM_OUTPUT_REGS];
                        m128_add_dpbusd_epi32x2(&mut acc[k], in0, b0, in1, b1)
                    }

                    small_block += 2
                }

                let output_vec: &mut [m128i] = unsafe {
                    cast_slice_to_m128i_mut(self.buffer.as_mut())
                };

                let bias_vec: &[m128i] = unsafe {
                    cast_slice_to_m128i(self.biases.as_ref())
                };

                let mut k = 0;
                while k < Self::NUM_OUTPUT_REGS {
                    let idx = (big_block * Self::NUM_OUTPUT_REGS + k) / 4;
                    output_vec[idx] = m128_haddx4(acc[k+0],acc[k+1],acc[k+2],acc[k+3],bias_vec[idx]);
                    k += 4;
                }

            }

            self.buffer.as_ref()
        }

    }

    // #[cfg(all(not(target_feature = "avx2"), not(target_feature = "ssse3")))]
    impl<Prev: NNLayer, const OS: usize, const IS: usize> NNAffine<Prev,OS,IS> {
        pub fn _propagate_nosimd<'a>(
            &'a mut self,
            trans_features: &'a [u8]
        ) -> &'a [<NNAffine<Prev,IS,OS> as NNLayer>::OutputType] {
            let input = self.prev.propagate(trans_features);
            let input = &input[..Self::SIZE_INPUT]; // TODO: bench
            for i in 0..Self::SIZE_OUTPUT {
                let offset = i * Self::SIZE_INPUT_PADDED;
                let mut sum: i32 = self.biases[i];
                for (j,x) in input.iter().enumerate() {
                    let x: i32 = x.as_();
                    let x0 = self.weights[offset + j] as i32 * x;
                    sum += x0;
                }
                self.buffer[i] = sum as i32;
            }
            self.buffer.as_ref()
        }
    }

    impl<Prev: NNLayer, const OS: usize, const IS: usize> NNLayer for NNAffine<Prev,OS,IS> {
        type InputType = Prev::OutputType;
        type OutputType = i32;
        const SIZE_OUTPUT: usize = OS;
        // const SIZE_INPUT: usize = Prev::SIZE_OUTPUT;
        const SIZE_INPUT: usize = IS;

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

        fn get_buf(&self) -> &[Self::OutputType] {
            self.buffer.as_ref()
        }
        fn get_buf_mut(&mut self) -> &mut [Self::OutputType] {
            // mut self.buffer.as_mut()
            self.buffer.as_mut()
        }

        #[cfg(feature = "nope")]
        fn propagate(&mut self, trans_features: &[u8]) { self._propagate_ndarray(trans_features); }

        // #[cfg(feature = "nope")]
        #[cfg(target_feature = "avx2")]
        fn propagate<'a>(&'a mut self, trans_features: &'a [u8]) -> &'a [Self::OutputType] {
            if Self::SIZE_INPUT_PADDED >= 128 {
                self._propagate_avx2_large(trans_features)
            } else {
                self._propagate_avx2_small(trans_features)
                // self._propagate_avx2_small_nosimd(trans_features)
            }
        }

        // fn propagate<'a>(&'a mut self, trans_features: &'a [u8]) -> &'a [Self::OutputType] {}

        #[cfg(all(not(target_feature = "avx2"), target_feature = "ssse3"))]
        fn propagate<'a>(&'a mut self, trans_features: &'a [u8]) -> &'a [Self::OutputType] {
            if Self::SIZE_INPUT_PADDED >= 128 {
                self._propagate_avx2_large(trans_features)
            } else {
                self._propagate_avx2_small(trans_features)
                // self._propagate_avx2_small_nosimd(trans_features)
            }
        }

        // #[cfg(feature = "nope")]
        #[cfg(all(not(target_feature = "avx2"), not(target_feature = "ssse3")))]
        // fn propagate(&mut self, trans_features: &[u8]) {
        fn propagate<'a>(&'a mut self, trans_features: &'a [u8]) -> &'a [Self::OutputType] {

            // eprintln!("affine propagate");
            // eprintln!("NNAffine InputType = {:?}", std::any::type_name::<Self::InputType>());

            let input = self.prev.propagate(trans_features);
            // let input = self.prev.get_buf();
            let input = &input[..Self::SIZE_INPUT];

            let input: &[u8] = unsafe {
                let ptr = input.as_ptr();
                let ptr2 = ptr as *const u8;
                std::slice::from_raw_parts(ptr2, input.len())
            };

            assert_eq!(self.biases.len(), Self::SIZE_OUTPUT);
            assert_eq!(self.buffer.len(), Self::SIZE_OUTPUT);
            // eprintln!("Self::SIZE_OUTPUT = {:?}", Self::SIZE_OUTPUT);
            // assert_eq!(Self::SIZE_OUTPUT % 4, 0);

            // assert_eq!(Self::SIZE_INPUT, Self::SIZE_INPUT_PADDED);

            // let x: u64;
            // unsafe {
            //     asm!(
            //         // "2:",
            //         "mov {}, 5",
            //         out(reg) x
            //     );
            // }
            // eprintln!("x = {:?}", x);

            // let mut acc = [0i32; IS];
            assert_eq!(IS, input.len());

            for i in 0..Self::SIZE_OUTPUT {

                let offset = i * Self::SIZE_INPUT_PADDED;

                let weights = &self.weights[offset..offset + IS];
                // assert_eq!(weights.len(), input.len());

                // let mut sum: i32 = self.biases[i];
                let mut sum: i32 = unsafe { *self.biases.get_unchecked(i) };

                for (k,x) in input.iter().enumerate() {
                    let x: i32 = x.as_();
                    // let x0 = self.weights[offset + k] as i32 * x;
                    let x0 = weights[k] as i32 * x;
                    // let x0 = self.weights[offset + j] as i32 * *x as i32; // no benefit
                    // let x0 = unsafe { *self.weights.get_unchecked(offset + j) } as i32 * x;
                    sum += x0;
                    // acc[k] = x0;
                }

                self.buffer[i] = sum as i32;
            }

            self.buffer.as_ref()
        }

        fn read_parameters(&mut self, mut rdr: &mut BufReader<File>) -> io::Result<()> {
            self.prev.read_parameters(rdr)?;
            // println!("wat NNAffine, OS = {:?}", OS);

            // eprintln!("Affine Read");
            // eprintln!("Self::SIZE_INPUT = {:?}", Self::SIZE_INPUT);
            // eprintln!("Self::SIZE_OUTPUT = {:?}", Self::SIZE_OUTPUT);
            // eprintln!("Self::SIZE_INPUT_PADDED = {:?}", Self::SIZE_INPUT_PADDED);

            for i in 0..Self::SIZE_OUTPUT {
                let x = rdr.read_i32::<LittleEndian>()?;
                self.biases[i] = x;
            }

            let size = Self::SIZE_INPUT_PADDED * Self::SIZE_OUTPUT;

            self.weights = Aligned(vec![0; size]);

            for i in 0..size {
                let x = rdr.read_i8()?;
                self.weights[Self::get_weight_index(i)] = x;
                // self.weights[i] = x;
            }

            Ok(())
        }

        fn write_parameters(&self, mut w: &mut BufWriter<File>) -> io::Result<()> {
            self.prev.write_parameters(w)?;
            for b in self.biases.iter() {
                w.write_i32::<LittleEndian>(*b)?;
            }
            // for wt in self.weights.iter() {
            //     w.write_u8(*wt)?;
            // }
            for i in 0..Self::SIZE_OUTPUT * Self::SIZE_INPUT_PADDED {
                // let wt = self.weights[Self::get_weight_index(i)];
                let wt = self.weights[i];
                w.write_i8(wt)?;
                // unimplemented!()
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
        pub prev:    Prev,
        // buf:         [u8; OS],
        buf:         Aligned<A64,[u8; OS]>,
    }

    impl<Prev: NNLayer, const OS: usize> NNClippedRelu<Prev, OS> {

        const SIZE_OUTPUT_PADDED: usize = ceil_to_multiple(Self::SIZE_OUTPUT, 32);

        const NUM_CHUNKS: usize = Prev::SIZE_OUTPUT / SIMD_WIDTH;

        pub fn new(prev: Prev) -> Self {
            Self {
                prev,
                buf:  Aligned([0; OS]),
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

        // fn size(&self) -> usize { self.prev.size() }

        fn get_buf(&self) -> &[Self::OutputType] {
            self.buf.as_ref()
        }
        fn get_buf_mut(&mut self) -> &mut [Self::OutputType] {
            self.buf.as_mut()
        }

        // fn propagate(&mut self, trans_features: &[u8]) {
        fn propagate<'a>(&'a mut self, trans_features: &'a [u8]) -> &'a [Self::OutputType] {

            // eprintln!("relu propagate");
            // eprintln!("NNRelu InputType = {:?}", std::any::type_name::<Self::InputType>());

            let input = self.prev.propagate(trans_features);
            // let input = self.prev.get_buf();

            // TODO: AVX2 magic

            // use std::simd::*;

            let start = 0;

            for (i,x) in input.iter().enumerate() {
                let x0: i32 = x.as_();
                let x1 = (x0.overflowing_shr(WEIGHT_SCALE_BITS).0).clamp(0, 127);
                self.buf[i] = x1.as_();
            }

            // for i in start..Self::SIZE_INPUT {
            //     let x0: i32 = input[i].as_();
            //     let x1 = (x0.overflowing_shr(WEIGHT_SCALE_BITS).0).clamp(0, 127);
            //     self.buf[i] = x1.as_();
            // }

            self.buf.as_ref()
        }

        fn read_parameters(&mut self, mut rdr: &mut BufReader<File>) -> io::Result<()> {
            let out = self.prev.read_parameters(rdr)?;
            // println!("wat NNRelu, Size = {:?}", Self::SIZE_INPUT);
            Ok(out)
        }

        fn write_parameters(&self, mut w: &mut BufWriter<File>) -> io::Result<()> {
            self.prev.write_parameters(w)?;
            Ok(())
        }

    }

}





