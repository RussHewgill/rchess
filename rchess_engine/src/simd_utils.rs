

#[cfg(feature = "nope")]
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

#[cfg(feature = "nope")]
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

#[cfg(all(not(target_feature = "avx2"), target_feature = "ssse3"))]
pub mod safe_arch {
    use safe_arch::*;

    pub unsafe fn cast_slice_to_m128i<T: Sized>(xs: &[T]) -> &[m128i] {
        let size = std::mem::size_of::<T>();
        assert_eq!(xs.len() % (16 / size), 0);
        let ptr = xs.as_ptr() as *const m128i;
        let len = xs.len() / (16 / size);
        std::slice::from_raw_parts(ptr, len)
    }

    pub unsafe fn cast_slice_to_m128i_mut<T: Sized>(xs: &mut [T]) -> &mut [m128i] {
        let size = std::mem::size_of::<T>();
        assert_eq!(xs.len() % (16 / size), 0);
        let ptr = xs.as_ptr() as *mut m128i;
        let len = xs.len() / (16 / size);
        std::slice::from_raw_parts_mut(ptr, len)
    }

    pub fn m128_add_dpbusd_epi32x2(
        mut acc: &mut safe_arch::m128i,
        a0: safe_arch::m128i, b0: safe_arch::m128i,
        a1: safe_arch::m128i, b1: safe_arch::m128i,
    ) {
        use safe_arch::*;
        let mut prod0 = mul_u8i8_add_horizontal_saturating_m128i(a0, b0);
        let prod1     = mul_u8i8_add_horizontal_saturating_m128i(a1, b1);
        prod0 = add_saturating_i16_m128i(prod0, prod1);
        prod0 = mul_i16_horizontal_add_m128i(prod0, set_splat_i16_m128i(1));
        *acc  = add_i32_m128i(*acc, prod0);
    }

    pub fn m128_haddx4(
        mut sum0: safe_arch::m128i,
        sum1:     safe_arch::m128i,
        mut sum2: safe_arch::m128i,
        sum3:     safe_arch::m128i,
        bias:     safe_arch::m128i
    ) -> safe_arch::m128i {
        use safe_arch::*;

        sum0 = add_horizontal_i32_m128i(sum0, sum1);
        sum2 = add_horizontal_i32_m128i(sum2, sum3);

        sum0 = add_horizontal_i32_m128i(sum0, sum2);

        add_i32_m128i(sum0, bias)
    }

}

#[cfg(target_feature = "avx2")]
// #[cfg(feature = "nope")]
pub mod safe_arch {
    use safe_arch::*;

    pub unsafe fn cast_slice_to_m256i_mut_unchecked<T: Sized>(xs: &mut [T]) -> &mut [m256i] {
        let size = std::mem::size_of::<T>();
        let ptr = xs.as_mut_ptr() as *mut m256i;
        let len = xs.len() / (32 / size);
        std::slice::from_raw_parts_mut(ptr, len)
    }

    pub unsafe fn cast_slice_to_m256i_unchecked<T: Sized>(xs: &[T]) -> &[m256i] {
        let size = std::mem::size_of::<T>();
        let ptr = xs.as_ptr() as *const m256i;
        let len = xs.len() / (32 / size);
        std::slice::from_raw_parts(ptr, len)
    }

    pub unsafe fn cast_slice_to_m256i_mut<T: Sized>(xs: &mut [T]) -> &mut [m256i] {
        let size = std::mem::size_of::<T>();
        assert_eq!(xs.len() % (32 / size), 0);
        let ptr = xs.as_mut_ptr() as *mut m256i;
        let len = xs.len() / (32 / size);
        std::slice::from_raw_parts_mut(ptr, len)
    }

    pub unsafe fn cast_slice_to_m256i<T: Sized>(xs: &[T]) -> &[m256i] {
        let size = std::mem::size_of::<T>();
        assert_eq!(xs.len() % (32 / size), 0);
        let ptr = xs.as_ptr() as *const m256i;
        let len = xs.len() / (32 / size);
        std::slice::from_raw_parts(ptr, len)
    }

    pub unsafe fn cast_slice_to_m128i<T: Sized>(xs: &[T]) -> &[m128i] {
        let size = std::mem::size_of::<T>();
        assert_eq!(xs.len() % (16 / size), 0);
        let ptr = xs.as_ptr() as *const m128i;
        let len = xs.len() / (16 / size);
        std::slice::from_raw_parts(ptr, len)
    }

    pub fn m256_add_dpbusd_epi32x2(
        mut acc: &mut safe_arch::m256i,
        a0: safe_arch::m256i, b0: safe_arch::m256i,
        a1: safe_arch::m256i, b1: safe_arch::m256i,
    ) {
        use safe_arch::*;
        let mut product0 = mul_u8i8_add_horizontal_saturating_m256i(a0, b0);
        let product1     = mul_u8i8_add_horizontal_saturating_m256i(a1, b1);
        product0         = add_saturating_i16_m256i(product0, product1);
        product0         = mul_i16_horizontal_add_m256i(product0, set_splat_i16_m256i(1));
        *acc             = add_i32_m256i(*acc, product0);
    }

    pub fn m256_add_dpbusd_epi32(
        mut acc: &mut safe_arch::m256i, a: safe_arch::m256i, b: safe_arch::m256i
    ) {
        let mut prod0 = mul_u8i8_add_horizontal_saturating_m256i(a, b);
        prod0         = mul_i16_horizontal_add_m256i(prod0, set_splat_i16_m256i(1));
        *acc          = add_i32_m256i(*acc, prod0);
    }

    pub fn m256_hadd(
        sum:     safe_arch::m256i,
        bias:    i32
    ) -> i32 {

        const _MM_PERM_BADC: i32 = 0x4E;
        const _MM_PERM_CDAB: i32 = 0xB1;

        let sum128lo = cast_to_m128i_from_m256i(sum);
        let sum128hi = extract_m128i_m256i::<1>(sum);
        let mut sum128 = add_i32_m128i(sum128lo,sum128hi);
        sum128 = add_i32_m128i(sum128, shuffle_ai_f32_all_m128i::<_MM_PERM_BADC>(sum128));
        sum128 = add_i32_m128i(sum128, shuffle_ai_f32_all_m128i::<_MM_PERM_CDAB>(sum128));

        get_i32_from_m128i_s(sum128) + bias
    }

    pub fn m256_haddx4(
        mut sum0: safe_arch::m256i,
        sum1:     safe_arch::m256i,
        mut sum2: safe_arch::m256i,
        sum3:     safe_arch::m256i,
        bias:     safe_arch::m128i
    ) -> safe_arch::m128i {
        use safe_arch::*;

        sum0 = add_horizontal_i32_m256i(sum0, sum1);
        sum2 = add_horizontal_i32_m256i(sum2, sum3);

        sum0 = add_horizontal_i32_m256i(sum0, sum2);

        let sum128lo = cast_to_m128i_from_m256i(sum0);
        let sum128hi = extract_m128i_m256i::<1>(sum0);

        add_i32_m128i(add_i32_m128i(sum128lo, sum128hi), bias)
    }

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


