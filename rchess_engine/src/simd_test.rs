
use std::arch::x86_64::{__m256i,__m128i};

use ndarray as nd;
use nd::{Array2,ArrayView2,ArrayViewMut2,ShapeBuilder};

pub fn simd_0(dst: &mut [f32], src: &[f32], gain_l: f32, gain_r: f32) {
    for i in 0..src.len() {
        dst[i * 2 + 0] = src[i] * gain_l;
        dst[i * 2 + 1] = src[i] * gain_r;
    }
}

pub fn simd_1(dst: &mut [f32], src: &[f32], gain_l: f32, gain_r: f32) {
    assert_eq!(src.len() % 4, 0);
    assert_eq!(dst.len(), src.len() * 2);

    use std::arch::x86_64::*;

    unsafe {
        let src_ptr = src.as_ptr();
        let dst_ptr = dst.as_mut_ptr();

        // create SIMD variables with each element set to the same value
        // mul_l = |  gain_l |  gain_l |  gain_l |  gain_l |
        // mul_r = |  gain_r |  gain_r |  gain_r |  gain_r |
        let mul_l = _mm_set1_ps(gain_l);
        let mul_r = _mm_set1_ps(gain_r);

        // process the source samples in blocks of four
        let mut i = 0;
        while i < src.len() {
            // load 4 of our source samples
            // input = | src(i + 0) | src(i + 1) | src(i + 2) | src(i + 3) |
            let input = _mm_loadu_ps(src_ptr.add(i));

            // multiply each of the four input values by the left and right volumes
            // we now have two variables containing four output values is each
            // out_l = | src(i + 0) * gain_l | src(i + 1) * gain_l | src(i + 2) * gain_l | src(i + 3) * gain_l |
            // out_r = | src(i + 0) * gain_r | src(i + 1) * gain_r | src(i + 2) * gain_r | src(i + 3) * gain_r |
            let out_l = _mm_mul_ps(input, mul_l);
            let out_r = _mm_mul_ps(input, mul_r);

            // re-arrange the output values so that each left-right pair is next to each other
            // out_lo = | src(i + 0) * gain_l | src(i + 0) * gain_r | src(i + 1) * gain_l | src(i + 1) * gain_r |
            // out_hi = | src(i + 2) * gain_l | src(i + 2) * gain_r | src(i + 3) * gain_l | src(i + 3) * gain_r |
            let out_lo = _mm_unpacklo_ps(out_l, out_r);
            let out_hi = _mm_unpackhi_ps(out_l, out_r);

            // write the four output samples (8 f32 values) to our destination memory
            _mm_storeu_ps(dst_ptr.add(2 * i + 0), out_lo);
            _mm_storeu_ps(dst_ptr.add(2 * i + 4), out_hi);

            i += 4;
        }
    }

}

pub use wat::*;
mod wat {
    use super::*;

    pub fn simd_mm_0<const IS: usize, const OS: usize>(
        // input:             &[i32],
        input:             &[u8],
        weights:           &[i8],
        biases:            &[i32],
        mut output:        &mut [i32],
    ) {

        let input      = &input[0..IS];
        let weights    = &weights[0..IS * OS];
        let biases     = &biases[0..OS];
        let mut output = &mut output[0..OS];

        for i in 0..OS {
            let offset = i * IS;
            let mut sum: i32 = biases[i];

            for j in 0..IS {
                let x: i32 = input[j] as i32;
                sum += weights[offset + j] as i32 * x;
                // sum += weights[i][k] as i32 * x;
            }

            // for (j,x) in input.iter().enumerate() {
            //     // let x: i32 = *x as i32;
            //     sum += weights[offset + j] as i32 * x;
            // }

            output[i] = sum;
        }

    }

    /// C_ij is the dot of A(row i) and B(col j)
    pub fn simd_mm_2<const IS: usize, const OS: usize>(
        input:             &[u8],
        // weights:           &[i32],
        weights:           &[[i8; IS]; OS],
        // weights:           &[[i8; OS]; IS], // slower?
        biases:            &[i32],
        mut output:        &mut [i32]
    ) {
        // use std::simd::*;
        use safe_arch::*;
        use crate::simd_utils::safe_arch::*;

        let input      = &input[0..IS];
        // let weights    = &weights[0..IS * OS];
        let biases     = &biases[0..OS];
        let mut output = &mut output[0..OS];

        // shape input   = 2048, 1
        // shape weights = 8, 2048

        // let bias_vec: &[m256i] = unsafe {
        //     let ptr = biases.as_ptr();
        //     let ptr = ptr as *const m128i;
        //     std::slice::from_raw_parts(ptr, self.biases.len() / 4)
        // };

        // let bias_vec: &[m128i] = unsafe {
        //     let ptr = self.biases.as_ptr();
        //     let ptr = ptr as *const m128i;
        //     std::slice::from_raw_parts(ptr, self.biases.len() / 4)
        // };

        // for i in 0..OS {
        if !true {
            let i = 0;
            let offset = i * OS;

            let mut sum_lo = m256i::from([biases[i],0,0,0,0,0,0,0]);
            let mut sum_hi = m256i::default();

            let row = &weights[i];

            for k in 0..IS/32 {
                let row_k = slice_to_m256i_i8(&row[k*32..]);
                let input_k = slice_to_m256i_u8(&input[k*32..]);

                let ex_row_lo = shr_imm_i16_m256i::<8>(unpack_low_i8_m256i(row_k, row_k));
                let ex_row_hi = shr_imm_i16_m256i::<8>(unpack_high_i8_m256i(row_k, row_k));

                let ex_input_lo = unpack_low_i8_m256i(input_k, zeroed_m256i());
                let ex_input_hi = unpack_high_i8_m256i(input_k, zeroed_m256i());

                let prod_lo = mul_i16_horizontal_add_m256i(ex_row_lo, ex_input_lo);
                let prod_hi = mul_i16_horizontal_add_m256i(ex_row_hi, ex_input_hi);

                sum_lo = add_i32_m256i(sum_lo, prod_lo);
                sum_hi = add_i32_m256i(sum_hi, prod_hi);
            }

            let sum = add_i32_m256i(sum_hi, sum_lo);

            let res: [i32; 8] = sum.into();
            eprintln!("res = {:?}", res);

            let sum_lo = cast_to_m128i_from_m256i(sum);
            let sum_hi = extract_m128i_m256i::<1>(sum);

            let sum2 = add_i32_m128i(sum_lo, sum_hi);

            let hi = unpack_high_i32_m128i(sum2, sum2);
            let sum = add_i32_m128i(hi, sum2);
            let sum = get_i32_from_m128i_s(sum);

            output[i] = sum;

            // let sum_hi_64 = shuffle_ai_f64_all_m128i::<0b01_00_11_10>(sum);

            // output[i] = extract_i32_from_m256i(a)

            // let sum = add_horizontal_i32_m256i(a, b)

            // output[i] = extract_i32_from_m256i::<0>(sum);

        }

        // for i in 0..OS {
        //     let mut sum = 0;
        //     for k in 0..IS {
        //         let x = input[k] as i32;
        //         // sum += weights[k][i] as i32 * x;
        //         sum += weights[i][k] as i32 * x;
        //     }
        //     // let sum = dot_product(input, &weights[i..i+IS]);
        //     // let sum = dot_product(input, &weights[i]);
        //     output[i] = biases[i] + sum;
        //     // output[i] = sum;
        // }

    }

    pub fn dot_product_basic(a: &[u8], b: &[i8]) -> i32 {
        assert_eq!(a.len(), b.len());
        assert!(a.len() % 32 == 0);
        let mut sum = 0;
        for i in 0..a.len() {
            sum += a[i] as i32 * b[i] as i32;
        }
        sum
    }

    pub fn dot_product0(a: &[u8], b: &[i8]) -> i32 {
        use safe_arch::*;
        use crate::simd_utils::safe_arch::*;

        assert_eq!(a.len(), b.len());
        assert!(a.len() % 32 == 0);

        let mut acc = vec![m256i::default(); a.len() / 32];

        for i in 0..a.len() / 32 {
            let offset = i * 32;

            let a0 = &a[offset..offset + 32];
            let b0 = &b[offset..offset + 32];

            let a0 = slice_to_m256i_u8(&a0);
            let b0 = slice_to_m256i_i8(&b0);

            let prod0 = mul_u8i8_add_horizontal_saturating_m256i(a0, b0);

            acc[i] = prod0;

        }

        let mut sum = m256i::default();
        for a in acc.into_iter() {
            sum = add_i16_m256i(sum, a);
        }

        let x = mul_i16_horizontal_add_m256i(sum, set_splat_i16_m256i(1));

        let x = add_horizontal_i32_m256i(x, m256i::default());

        let x: [i32; 8] = x.into();
        // let x: [i16; 16] = x.into();

        // eprintln!("x = {:?}", x);

        let out = x.iter().sum();

        // let mut out = 0;
        // out += x[0];
        // out += x[2];
        // out += x[4];
        // out += x[6];

        // let mut x = 0;
        // extract_i32_imm_m128i::<0>(sum);

        out
        // unimplemented!()
    }

    pub fn dot_product1(a: &[i32], b: &[i32]) -> i32 {
        use std::simd::*;

        assert_eq!(a.len(), b.len());
        assert!(a.len() % 4 == 0);

        let mut sum = a.array_chunks::<4>()
            .map(|&x| i32x4::from_array(x))
            .zip(b.array_chunks::<4>().map(|&y| i32x4::from_array(y)))
            .map(|(a,b)| (a * b).horizontal_sum())
            .sum();

        sum
    }

    pub fn dot_product2(a: &[i32], b: &[i32]) -> i32 {
        use std::simd::*;

        assert_eq!(a.len(), b.len());
        assert!(a.len() % 4 == 0);

        let mut sum = a.array_chunks::<4>()
            .map(|&x| i32x4::from_array(x))
            .zip(b.array_chunks::<4>().map(|&y| i32x4::from_array(y)))
            .map(|(a,b)| a * b)
            .sum::<i32x4>()
            .horizontal_sum();

        sum
    }

}

// pub fn simd_mm_1<const IS: usize, const OS: usize>(
//     input:             &[u8],
//     weights:           &[i8],
//     biases:            &[i32],
//     mut output:        &mut [i32]
// ) {
//     use std::simd::*;
//     let input      = &input[0..IS];
//     let weights    = &weights[0..IS * OS];
//     let biases     = &biases[0..OS];
//     let mut output = &mut output[0..OS];
//     // for i in 0..OS/8 {
//     //     let offset = i * IS;
//     //     // let mut sum = biases[i];
//     //     // output[i] = sum;
//     // }
// }

#[allow(non_camel_case_types)]
pub struct SIMD_01<const IS: usize, const OS: usize>;

impl<const IS: usize, const OS: usize> SIMD_01<IS,OS> {

    const INPUT_SIMD_WIDTH: usize = 32; // AVX2
    // const INPUT_SIMD_WIDTH: usize = 16; // SSE3
    const MAX_NUM_OUTPUT_REGS: usize = 8;

    // const INPUT_SIMD_WIDTH: usize = 1;
    // const MAX_NUM_OUTPUT_REGS: usize = 1;

    const NUM_OUTPUT_REGS: usize  = if OS > Self::MAX_NUM_OUTPUT_REGS {
        Self::MAX_NUM_OUTPUT_REGS } else { OS }; // 8

    const SMALL_BLOCK_SIZE: usize = Self::INPUT_SIMD_WIDTH;     // 32
    const BIG_BLOCK_SIZE: usize   = Self::NUM_OUTPUT_REGS * IS; // 8192

    const NUM_SMALL_BLOCKS_PER_BIG_BLOCK: usize = Self::BIG_BLOCK_SIZE / Self::SMALL_BLOCK_SIZE; // 256
    const NUM_SMALL_BLOCKS_PER_OUTPUT: usize = IS / Self::SMALL_BLOCK_SIZE; // 32

    const NUM_BIG_BLOCKS: usize = OS / Self::NUM_OUTPUT_REGS; // 1

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

}

/// Approach 1:
///   - used when the PaddedInputDimensions >= 128
///   - uses AVX512 if possible
///   - processes inputs in batches of 2*InputSimdWidth
///   - so in batches of 128 for AVX512
///     and in batches of 64 for AVX2
///   - the weight blocks of size InputSimdWidth are transposed such that
///     access is sequential
///   - N columns of the weight matrix are processed a time, where N
///     depends on the architecture (the amount of registers)
///   - accumulate + hadd is used
#[allow(unused_doc_comments)]
impl<const IS: usize, const OS: usize> SIMD_01<IS,OS> {

    pub fn simd_mm(
        input:             &[i8],
        weights:           &[i8],
        biases:            &[i32],
        mut output:        &mut [i32]
    ) {
        let input      = &input[0..IS];
        let weights    = &weights[0..IS * OS];
        let biases     = &biases[0..OS];
        let mut output = &mut output[0..OS];

        use std::arch::x86_64::*;
        // use std::simd::*;
        use crate::simd_utils::std_simd::cast_slice_to_i8x32;
        use crate::simd_utils::x86_64::*;
        use crate::simd_utils::x86_64::conversions::*;

        // eprintln!("IS * OS = {:?}", IS * OS);
        // eprintln!("Self::SMALL_BLOCK_SIZE = {:?}", Self::SMALL_BLOCK_SIZE);
        // eprintln!("Self::BIG_BLOCK_SIZE = {:?}", Self::BIG_BLOCK_SIZE);
        // eprintln!("Self::NUM_BIG_BLOCKS = {:?}", Self::NUM_BIG_BLOCKS);
        // eprintln!("Self::NUM_SMALL_BLOCKS_PER_BIG_BLOCK = {:?}", Self::NUM_SMALL_BLOCKS_PER_BIG_BLOCK);
        // eprintln!("Self::NUM_SMALL_BLOCKS_PER_OUTPUT = {:?}", Self::NUM_SMALL_BLOCKS_PER_OUTPUT);

        // let input: &[i8x32] = cast_slice_to_i8x32(input);

        let input: &[__m256i] = unsafe {
            let ptr = input.as_ptr() as *const __m256i;
            std::slice::from_raw_parts(ptr, input.len() / 32)
        };

        let big_block = 0;

        let mut acc: Vec<__m256i> = vec![unsafe { _mm256_setzero_si256() }; Self::NUM_OUTPUT_REGS];

        /// Each big block has NumOutputRegs small blocks in each "row", one per register.
        /// We process two small blocks at a time to save on one addition without VNNI.
        let mut small_block = 0;
        while small_block < Self::NUM_SMALL_BLOCKS_PER_OUTPUT {

            let offset = big_block * Self::BIG_BLOCK_SIZE
                + small_block * Self::SMALL_BLOCK_SIZE * Self::NUM_OUTPUT_REGS;

            let weight_vec: &[i8] = &weights[offset..];
            let weight_vec: &[__m256i] = unsafe {
                let ptr = weight_vec.as_ptr() as *const __m256i;
                std::slice::from_raw_parts(ptr, weight_vec.len() / 32)
            };

            let in0 = input[small_block + 0];
            let in1 = input[small_block + 1];

            for k in 0..Self::NUM_OUTPUT_REGS {

                let b0 = weight_vec[k];
                let b1 = weight_vec[k + Self::NUM_OUTPUT_REGS];

                Self::m256_add_dpbusd_epi32x2(&mut acc[k], in0, b0,in1, b1);

            }

            small_block += 2;
        }

        let output_vec: &mut [__m128i] = unsafe {
            let ptr = output.as_mut_ptr();
            let ptr = ptr as *mut __m128i;
            std::slice::from_raw_parts_mut(ptr, output.len() / 4)
        };

        let bias_vec: &[__m128i] = unsafe {
            let ptr = biases.as_ptr();
            let ptr = ptr as *mut __m128i;
            std::slice::from_raw_parts(ptr, biases.len() / 4)
        };

        let mut k = 0;
        while k < Self::NUM_OUTPUT_REGS {
            let idx = (big_block * Self::NUM_OUTPUT_REGS + k) / 4;

            output_vec[idx] = Self::m256_haddx4(acc[k+0],acc[k+1],acc[k+2],acc[k+3],bias_vec[idx]);

            k += 4;
        }

    }

    fn m256_add_dpbusd_epi32x2(
        mut acc: &mut __m256i,
        a0: __m256i, b0: __m256i,
        a1: __m256i, b1: __m256i,
    ) {
        use std::arch::x86_64::*;
        unsafe {
            let mut product0 = _mm256_maddubs_epi16(a0, b0);
            let product1 = _mm256_maddubs_epi16(a1, b1);
            product0 = _mm256_adds_epi16(product0, product1);
            product0 = _mm256_madd_epi16(product0, _mm256_set1_epi16(1));
            *acc = _mm256_add_epi32(*acc, product0);
        }
    }

    fn m256_haddx4(
        mut sum0: __m256i,
        sum1: __m256i,
        mut sum2: __m256i,
        sum3: __m256i,
        bias: __m128i,
    ) -> __m128i {
        use std::arch::x86_64::*;
        unsafe {

            // sum0 = _mm256_hadd_epi32(sum0, sum1);
            // sum2 = _mm256_hadd_epi32(sum2, sum3);
            // sum0 = _mm256_hadd_epi32(sum0, sum2);
            // let sum128lo = _mm256_castsi256_si128(sum0);
            // let sum128hi = _mm256_extracti128_si256::<1>(sum0);
            // _mm_add_epi32(_mm_add_epi32(sum128lo, sum128hi), bias)


            sum0 = _mm256_hadd_epi32(sum0, sum1);
            sum2 = _mm256_hadd_epi32(sum2, sum3);

            sum0 = _mm256_hadd_epi32(sum0, sum2);

            let sum128lo = _mm256_castsi256_si128(sum0);
            let sum128hi = _mm256_extracti128_si256(sum0, 1);

            _mm_add_epi32(_mm_add_epi32(sum128lo, sum128hi), bias)

            // [[maybe_unused]] static __m128i m256_haddx4(
            //     __m256i sum0, __m256i sum1, __m256i sum2, __m256i sum3,
            //     __m128i bias) {
            //     sum0 = _mm256_hadd_epi32(sum0, sum1);
            //     sum2 = _mm256_hadd_epi32(sum2, sum3);
            //     sum0 = _mm256_hadd_epi32(sum0, sum2);
            //     __m128i sum128lo = _mm256_castsi256_si128(sum0);
            //     __m128i sum128hi = _mm256_extracti128_si256(sum0, 1);
            //     return _mm_add_epi32(_mm_add_epi32(sum128lo, sum128hi), bias);
            // }

            // unimplemented!()

        }
    }

}

pub fn simd_nd_mm_0<const IS: usize, const OS: usize>(
    input:             &[i8],
    weights:           &[i8],
    biases:            &[i32],
    mut output:        &mut [i32]
) {
    let input: nd::Array2<i8> = nd::Array2::from_shape_vec((IS, 1), input.to_vec()).unwrap();
    let weights: nd::Array2<i8> = nd::Array2::from_shape_vec((IS,OS).f(), weights.to_vec()).unwrap();
    let weights = weights.reversed_axes();
    let biases: nd::Array2<i32> = nd::Array2::from_shape_vec((OS, 1), biases.to_vec()).unwrap();

    eprintln!("input.shape() = {:?}", input.shape());     // 1024, 1
    eprintln!("weights.shape() = {:?}", weights.shape()); // 8, 1024
    eprintln!("biases.shape() = {:?}", biases.shape());   // 8, 1

    let input   = input.map(|x| *x as i32);
    let weights = weights.map(|x| *x as i32);
    let biases  = biases.map(|x| *x as i32);

    let result = weights.dot(&input) + &biases;
    output.copy_from_slice(result.as_slice().unwrap());
}

pub fn simd_nd_mm_1<const IS: usize, const OS: usize>(
    input:             ArrayView2<u8>,
    weights:           ArrayView2<i8>,
    biases:            ArrayView2<i32>,
    mut result:        &mut Array2<i32>,
) {
    let input   = input.map(|x| *x as i32);
    let weights = weights.map(|x| *x as i32);
    let biases  = biases.map(|x| *x as i32);
    *result = weights.dot(&input) + &biases;
}

pub fn simd_nd_mm_3<const IS: usize, const OS: usize>(
    input:             &[i32],
    weights:           &[i32],
    biases:            &[i32],
    mut output:        &mut [i32]
) {
    let input: nd::Array2<i32> = nd::Array2::from_shape_vec((IS, 1), input.to_vec()).unwrap();
    let weights: nd::Array2<i32> = nd::Array2::from_shape_vec((IS,OS).f(), weights.to_vec()).unwrap();
    let weights = weights.reversed_axes();
    let biases: nd::Array2<i32> = nd::Array2::from_shape_vec((OS, 1), biases.to_vec()).unwrap();

    // eprintln!("input.shape() = {:?}", input.shape());     // 1024, 1
    // eprintln!("weights.shape() = {:?}", weights.shape()); // 8, 1024
    // eprintln!("biases.shape() = {:?}", biases.shape());   // 8, 1

    let input   = input.map(|x| *x as i32);
    let weights = weights.map(|x| *x as i32);
    let biases  = biases.map(|x| *x as i32);

    for i in 0..OS {
        let row = weights.row(i);
        let col = input.column(0);
        // eprintln!("row.shape() = {:?}", row.shape());
        // eprintln!("col.shape() = {:?}", col.shape());
        let sum = row.dot(&col);
        output[i] = sum;
    }


    // let result = weights.dot(&input) + &biases;
    // output.copy_from_slice(result.as_slice().unwrap());
}


