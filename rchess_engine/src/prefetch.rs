



// pub trait Prefetch {
//     fn prefetch(&self);
// }


// use std::intrinsics::prefetch_write_data;
// unsafe {
//     prefetch_write_data::<T>(ptr, 3);
// }

/// https://github.com/sfleischman105/Pleco/blob/master/pleco/src/tools/mod.rs

/// Prefetch's `ptr` to all levels of the cache.
///
/// For some platforms this may compile down to nothing, and be optimized away.
/// To prevent compiling down into nothing, compilation must be done for a
/// `x86` or `x86_64` platform with SSE instructions available. An easy way to
/// do this is to add the environmental variable `RUSTFLAGS=-C target-cpu=native`.
#[inline(always)]
pub fn prefetch_write<T>(ptr: *const T) {
    __prefetch_write::<T>(ptr);
}


#[cfg(feature = "nightly")]
#[inline(always)]
fn __prefetch_write<T>(ptr: *const T) {
    use std::intrinsics::prefetch_write_data;
    unsafe {
        prefetch_write_data::<T>(ptr, 3);
    }
}

#[cfg(
    all(
        not(feature = "nightly"),
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse"
    )
)]
#[inline(always)]
fn __prefetch_write<T>(ptr: *const T) {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::_mm_prefetch;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::_mm_prefetch;
    unsafe {
        _mm_prefetch(ptr as *const i8, 3);
    }
}

#[cfg(
    all(
        not(feature = "nightly"),
        any(
            all(
                any(target_arch = "x86", target_arch = "x86_64"),
                not(target_feature = "sse")
            ),
            not(any(target_arch = "x86", target_arch = "x86_64"))
        )
    )
)]
#[inline(always)]
fn __prefetch_write<T>(ptr: *const T) {
    // Do nothing
}

