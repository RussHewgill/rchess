







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


pub fn simd_mm_0<const IS: usize, const OS: usize>(
    input:             &[u8],
    weights:           &[i8],
    biases:            &[i32],
    mut output:        &mut [i32]
) {

    let input      = &input[0..IS];
    let weights    = &weights[0..IS * OS];
    let biases     = &biases[0..OS];
    let mut output = &mut output[0..OS];

    for i in 0..OS {
        let offset = i * IS;
        let mut sum = biases[i];
        for j in 0..IS {
            let x = input[j] as i32;
            sum += weights[offset + j] as i32 * x;
        }
        output[i] = sum;
    }
}

#[allow(non_camel_case_types)]
pub struct SIMD_01<const IS: usize, const OS: usize>;

impl<const IS: usize, const OS: usize> SIMD_01<IS,OS> {

    const INPUT_SIMD_WIDTH: usize = 16; // SSE3
    const MAX_NUM_OUTPUT_REGS: usize = 8;

    const NUM_OUTPUT_REGS: usize  = if OS > Self::MAX_NUM_OUTPUT_REGS {
        Self::MAX_NUM_OUTPUT_REGS } else { OS };
    const SMALL_BLOCK_SIZE: usize = Self::INPUT_SIMD_WIDTH;
    const BIG_BLOCK_SIZE: usize   = Self::NUM_OUTPUT_REGS * IS;

    const NUM_SMALL_BLOCKS_PER_BIG_BLOCK: usize = Self::BIG_BLOCK_SIZE / Self::SMALL_BLOCK_SIZE;
    const NUM_SMALL_BLOCKS_PER_OUTPUT: usize = IS / Self::SMALL_BLOCK_SIZE;

    const NUM_BIG_BLOCKS: usize = OS / Self::NUM_OUTPUT_REGS;

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

impl<const IS: usize, const OS: usize> SIMD_01<IS,OS> {

    pub fn simd_mm(
        input:             &[u8],
        weights:           &[i8],
        biases:            &[i32],
        mut output:        &mut [i32]
    ) {
        let input      = &input[0..IS];
        let weights    = &weights[0..IS * OS];
        let biases     = &biases[0..OS];
        let mut output = &mut output[0..OS];

    }

}

use ndarray as nd;
use nd::{Array2,ArrayView2,ArrayViewMut2,ShapeBuilder};

pub fn simd_nd_mm_0<const IS: usize, const OS: usize>(
    input:             &[u8],
    weights:           &[i8],
    biases:            &[i32],
    mut output:        &mut [i32]
) {
    let input: nd::Array2<u8> = nd::Array2::from_shape_vec((IS, 1), input.to_vec()).unwrap();
    let weights: nd::Array2<i8> = nd::Array2::from_shape_vec((IS,OS).f(), weights.to_vec()).unwrap();
    let weights = weights.reversed_axes();
    let biases: nd::Array2<i32> = nd::Array2::from_shape_vec((OS, 1), biases.to_vec()).unwrap();
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
