
use std::convert::TryInto;

use core::arch::x86_64::*;


pub fn m256i_from_slice(s: &[u8]) -> __m256i {
    assert!(s.len() >= 32);
    let x0: i64 = i64::from_ne_bytes(s[0..8].try_into().unwrap());
    let x1: i64 = i64::from_ne_bytes(s[8..16].try_into().unwrap());
    let x2: i64 = i64::from_ne_bytes(s[16..24].try_into().unwrap());
    let x3: i64 = i64::from_ne_bytes(s[24..32].try_into().unwrap());
    unsafe { _mm256_set_epi64x(x0,x1,x2,x3) }
}

pub fn m128i_from_slice(s: &[u8]) -> __m256i {
    assert!(s.len() >= 32);
    let x0: i64 = i64::from_ne_bytes(s[0..8].try_into().unwrap());
    let x1: i64 = i64::from_ne_bytes(s[8..16].try_into().unwrap());
    let x2: i64 = i64::from_ne_bytes(s[16..24].try_into().unwrap());
    let x3: i64 = i64::from_ne_bytes(s[24..32].try_into().unwrap());
    unsafe { _mm256_set_epi64x(x0,x1,x2,x3) }
}







