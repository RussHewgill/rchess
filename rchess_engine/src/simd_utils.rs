

pub mod x86_64 {
    use std::convert::TryInto;
    use core::arch::x86_64::*;

    pub fn build_m256i_from_slice(s: &[u8]) -> __m256i {
        assert!(s.len() >= 32);
        let x0: i64 = i64::from_ne_bytes(s[0..8].try_into().unwrap());
        let x1: i64 = i64::from_ne_bytes(s[8..16].try_into().unwrap());
        let x2: i64 = i64::from_ne_bytes(s[16..24].try_into().unwrap());
        let x3: i64 = i64::from_ne_bytes(s[24..32].try_into().unwrap());
        unsafe { _mm256_set_epi64x(x0,x1,x2,x3) }
    }

    pub fn build_m128i_from_slice(s: &[u8]) -> __m128i {
        assert!(s.len() >= 16);
        let x0: i64 = i64::from_ne_bytes(s[0..8].try_into().unwrap());
        let x1: i64 = i64::from_ne_bytes(s[8..16].try_into().unwrap());
        unsafe { _mm_set_epi64x(x0,x1) }
    }

    pub mod conversions {
        use core::arch::x86_64::*;
        use std::simd::*;

        use super::build_m256i_from_slice;

        // pub trait FromSIMD<const WIDTH: usize> {

        // macro_rules! impl_simd_from {
        //     ($t0:ty, $t1:ty) => {
        //         impl ConvertSIMD<$t0> for $t1 {
        //             fn from(sq: $t0) -> Self {
        //                 unimplemented!()
        //             }
        //         }
        //         impl ConvertSIMD<$t1> for $t0 {
        //             // impl std::convert::From<Coord> for usize {
        //             fn from(c: $t1) -> Self {
        //                 unimplemented!()
        //             }
        //         }
        //     };
        // }

        pub trait ConvertSIMD<T> {
            fn simd_from(a: T) -> Self;
        }

        impl ConvertSIMD<__m256i> for u8x32 {
            fn simd_from(a: __m256i) -> Self {
                let mut xs = [0u8; 32];
                unsafe {
                    let mut ptr: *mut u8 = xs.as_mut_ptr();
                    let mut ptr = ptr as *mut __m256i;
                    _mm256_storeu_si256(ptr, a);
                }
                Self::from_array(xs)
            }
        }

        impl ConvertSIMD<u8x32> for __m256i {
            fn simd_from(a: u8x32) -> Self {
                unsafe {
                    let ptr = a.as_array();
                    let ptr = ptr.as_ptr() as *const __m256i;
                    _mm256_loadu_si256(ptr)
                }
            }
        }

        impl ConvertSIMD<i8x32> for __m256i {
            fn simd_from(a: i8x32) -> Self {
                unsafe {
                    let ptr = a.as_array();
                    let ptr = ptr.as_ptr() as *const __m256i;
                    _mm256_loadu_si256(ptr)
                }
            }
        }

        // impl std::convert::From<__m256i> for i8x32 {
        //     fn from(a: __m256i) -> Self {
        //         unimplemented!()
        //     }
        // }

    }

}

pub mod std_simd {
    use std::simd::*;

    pub fn cast_slice_to_i8x32(xs: &[i8]) -> &[i8x32] {
        assert!(xs.len() >= 32);
        // assert!(xs.len() % 32 == 0);
        unsafe {
            let ptr = xs.as_ptr();
            let ptr = ptr as *const i8x32;
            std::slice::from_raw_parts(ptr, xs.len() / 32)
        }
    }

}

pub mod safe_arch {
    use safe_arch::*;

    /// Overflows to negative
    pub fn mul_u8_m256i(a: m256i, b: m256i) -> m256i {
        let even = mul_i16_keep_low_m256i(a, b);
        let odd  = mul_i16_keep_low_m256i(shr_imm_u16_m256i::<8>(a), shr_imm_u16_m256i::<8>(b));
        let result = bitand_m256i(even, set_splat_i16_m256i(0xFF));
        let result = bitor_m256i(shl_imm_u16_m256i::<8>(odd), result);
        result
    }

    pub fn slice_to_m256i_u8(xs: &[u8]) -> m256i {
        m256i::from(slice_to_array_u8_32(xs))
    }

    pub fn slice_to_m256i_i8(xs: &[i8]) -> m256i {
        m256i::from(slice_to_array_i8_32(xs))
    }

    pub fn slice_to_array_u8_32(xs: &[u8]) -> [u8; 32] {
        let xs = &xs[0..32];
        let mut out = [0; 32];
        out.copy_from_slice(xs);
        out
    }

    pub fn slice_to_array_i8_32(xs: &[i8]) -> [i8; 32] {
        let xs = &xs[0..32];
        let mut out = [0; 32];
        out.copy_from_slice(xs);
        out
    }

}


