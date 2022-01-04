
use crate::trans_table::Node;
use crate::explore::SearchInfo;
use crate::hashing::Zobrist;

use derive_new::new;
use parking_lot::RwLock;

use std::sync::atomic::{AtomicUsize, AtomicU8};

/// https://www.chess2u.com/t1820-setting-correct-hashtable-size?highlight=hash+size
/// HT[KB] = 2.0 * PFreq[MHz] * t[s]

// pub const DEFAULT_TT_SIZE_MB: usize = 1024;
// pub const DEFAULT_TT_SIZE_MB: usize = 256;
// pub const DEFAULT_TT_SIZE_MB: usize = 128;
// pub const DEFAULT_TT_SIZE_MB: usize = 64;
pub const DEFAULT_TT_SIZE_MB: usize = 32;
// pub const DEFAULT_TT_SIZE_MB: usize = 16;

const ENTRIES_PER_BUCKET: usize = 3;
// const ENTRIES_PER_BUCKET: usize = 2;

const KILOBYTE: usize = 1024;
const MEGABYTE: usize = 1024 * KILOBYTE;

const HIGH_FOUR_BYTES: u64 = 0xFF_FF_FF_FF_00_00_00_00;
const LOW_FOUR_BYTES: u64 = 0x00_00_00_00_FF_FF_FF_FF;
const SHIFT_TO_LOWER: u64 = 32;

#[derive(Debug)]
pub struct TransTable {
    buf:           Vec<Bucket>,
    num_buckets:   usize,
    used_entries:  AtomicUsize,
    cycles:        AtomicU8,
}

/// New, Insert, Probe
impl TransTable {

    pub fn new_mb(megabytes: usize) -> Self {
        unimplemented!()
    }

    pub fn insert(&self, zb: Zobrist, si: SearchInfo) {
        unimplemented!()
    }

    pub fn probe(&self, zb: Zobrist) -> Option<(bool,SearchInfo)> {
        unimplemented!()
    }

}

/// Calc index, clear, increment
impl TransTable {

    fn calc_index(&self, zb: Zobrist) -> usize {

        // slotNb == 1024(powerOf2), blocksize== 4
        // address = key & (1024-1) ^ 0, 1, 2, 3;

        // return ((uint128)a * (uint128)b) >> 64;
        let key = (zb.0 as u128 * self.num_buckets as u128).overflowing_shr(64).0 as u64;

        unimplemented!()
    }

    pub fn clear_table(&self) {
        for bucket in self.buf.iter() {
            bucket.clear();
        }
        self.used_entries.store(0, std::sync::atomic::Ordering::SeqCst);
        self.cycles.store(0, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn increment_cycle(&self) {
        self.cycles.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

}

/// Queries
impl TransTable {

    pub fn total_entries(&self) -> usize {
        self.num_buckets * ENTRIES_PER_BUCKET
    }

    pub fn used_entries(&self) -> usize {
        self.used_entries.load(std::sync::atomic::Ordering::Relaxed)
    }

}

/// Prefetch
impl TransTable {
    pub fn prefetch(&self, zb: Zobrist) {
        let idx = self.calc_index(zb);
        unsafe {
            // let ptr = (*self.vec.get()).as_ptr().add(idx);
            let ptr = self.buf.as_ptr().add(idx);
            crate::prefetch::prefetch_write(ptr);
        }
    }
}

#[derive(Debug,Default,Eq,PartialEq,PartialOrd,Hash,Clone,Copy,new)]
pub struct TTEntry {
    age:                u8,
    entry:              Option<SearchInfo>,
}

#[derive(Debug)]
// #[repr(align(64))]
pub struct Bucket {
    bucket: [RwLock<TTEntry>; ENTRIES_PER_BUCKET],
}

/// New, clear
impl Bucket {
    pub fn new() -> Self {
        Self {
            bucket: array_init::array_init(|_| RwLock::new(TTEntry::default())),
        }
    }

    pub fn clear(&self) {
        for e in self.bucket.iter() {
            let mut w = e.write();
            *w = TTEntry::default();
        }
    }
}

/// store, find
impl Bucket {

    pub fn store(&self, ver: u32, si: SearchInfo, used_entries: &AtomicUsize, age: &AtomicU8) {
        unimplemented!()
    }

    pub fn find(&self, ver: u32, age: &AtomicU8) -> Option<(bool,SearchInfo)> {
        unimplemented!()
    }

}

