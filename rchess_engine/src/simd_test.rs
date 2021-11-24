







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


