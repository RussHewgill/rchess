
use crate::{trans_table::Node, explore::PackedSearchInfo};
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

        let mut num_buckets: usize = (megabytes * MEGABYTE) / std::mem::size_of::<Bucket>();
        num_buckets = num_buckets.next_power_of_two() / 2;

        // let tot_entries = num_buckets * ENTRIES_PER_BUCKET;

        let mut buf = vec![];
        for _ in 0..num_buckets {
            buf.push(Bucket::new());
        }

        Self {
            buf,
            num_buckets,
            used_entries:  AtomicUsize::new(0),
            cycles:        AtomicU8::new(0),
        }
    }

    pub fn insert(&self, zb: Zobrist, si: SearchInfo) {
        let (idx,ver) = self.calc_index(zb);

        if let Some(bucket) = self.buf.get(idx) {
            // bucket.store(si, &self.used_entries, &self.cycles);
            bucket.store(ver, si, &self.used_entries, &self.cycles);
        } else {
            panic!("TT insert, bad bucket idx: {:?}, zb = {:?}", idx, zb);
        }
    }

    pub fn probe(&self, zb: Zobrist) -> Option<(bool,SearchInfo)> {
        let (idx,ver) = self.calc_index(zb);

        if let Some(bucket) = self.buf.get(idx) {
            bucket.find(ver, &self.cycles)
        } else {
            panic!("TT probe, bad bucket idx: {:?}, zb = {:?}", idx, zb);
        }
    }

}

/// Calc index
impl TransTable {

    // #[cfg(feature = "nope")]
    /// https://en.wikipedia.org/wiki/Hash_function#Multiplicative_hashing
    pub fn calc_index(&self, zb: Zobrist) -> (usize, u32) {
        let key = (zb.0 as u128 * self.num_buckets as u128).overflowing_shr(64).0;

        let ver = (zb.0 & LOW_FOUR_BYTES) as u32;

        (key as usize, ver)
    }

    #[cfg(feature = "nope")]
    pub fn calc_index(&self, zb: Zobrist) -> (usize, u32) {
        let key = (zb.0 & HIGH_FOUR_BYTES) >> SHIFT_TO_LOWER;
        let total = self.num_buckets as u64;
        (key % total) as usize
    }

    #[cfg(feature = "nope")]
    pub fn calc_index(&self, zb: Zobrist) -> (usize, u32) {
        let key = zb.0 % self.num_buckets as u64;
        key as usize
    }

}

/// clear, increment
impl TransTable {

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
        let (idx,_) = self.calc_index(zb);
        unsafe {
            // let ptr = (*self.vec.get()).as_ptr().add(idx);
            let ptr = self.buf.as_ptr().add(idx);
            crate::prefetch::prefetch_write(ptr);
        }
    }
}

#[derive(Debug,Default,Clone,Copy,new)]
pub struct TTEntry {
    age:                u8,
    // zb:                 Option<u32>,
    entry:              Option<(u32,SearchInfo)>,
    // entry:              SearchInfo,
    // entry:              Option<PackedSearchInfo>,
}

// #[derive(Debug)]
// // #[repr(align(64))]
// pub struct Bucket {
//     bucket: [RwLock<TTEntry>; ENTRIES_PER_BUCKET],
// }

#[derive(Debug)]
#[repr(align(64))]
pub struct Bucket {
    // bucket: [RwLock<TTEntry>; ENTRIES_PER_BUCKET],
    bucket: RwLock<[TTEntry; ENTRIES_PER_BUCKET]>,
}

/// New, clear
impl Bucket {
    pub fn new() -> Self {
        Self {
            // bucket: array_init::array_init(|_| RwLock::new(TTEntry::default())),
            bucket: RwLock::new(array_init::array_init(|_| TTEntry::default())),
        }
    }

    pub fn clear(&self) {

        for e in self.bucket.write().iter_mut() {
            *e = TTEntry::default();
        }

        // for e in self.bucket.iter() {
        //     let mut w = e.write();
        //     *w = TTEntry::default();
        // }

    }
}

/// store, find
impl Bucket {

    pub fn store(&self, ver: u32, si: SearchInfo, used_entries: &AtomicUsize, age: &AtomicU8) {

        let mut idx_lowest_depth = None;

        // for (entry_idx,entry) in self.bucket.iter().enumerate() {
        //     let e = entry.read();
        //     // if let Some(e_si) = e.entry {
        //     //     if e_si.depth_searched < si.depth_searched {
        //     //         idx_lowest_depth = Some(entry_idx);
        //     //     }
        //     // }
        // }

        for (entry_idx,e) in self.bucket.read().iter().enumerate() {
            if let Some((ver,e_si)) = e.entry {
                if e_si.depth_searched < si.depth_searched {
                    idx_lowest_depth = Some(entry_idx);
                }
            }
        }

        let idx = if let Some(idx) = idx_lowest_depth {
            // used_entries.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            used_entries.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            idx
        } else { 0 };

        let age = age.load(std::sync::atomic::Ordering::Relaxed);

        // let mut w = self.bucket[idx_lowest_depth].write();
        // *w = TTEntry::new(age, Some(si));

        let mut w = self.bucket.write();
        let mut w = w.get_mut(idx).unwrap();
        *w = TTEntry::new(age, Some((ver,si)));

    }

    pub fn find(&self, ver: u32, age: &AtomicU8) -> Option<(bool,SearchInfo)> {

        for (entry_idx,e) in self.bucket.read().iter().enumerate() {
            if let Some((ver2,ee)) = e.entry {
                if ver2 == ver {
                    let age = age.load(std::sync::atomic::Ordering::Relaxed);
                    return Some((age == e.age, ee));
                }
            }
        }
        None

        // if let Some(e) = self.bucket.read().iter()
        //     .filter(|e| e.entry.is_some())
        //     .max_by_key(|e| e.entry.unwrap().depth_searched) {
        //         let age = age.load(std::sync::atomic::Ordering::Relaxed);
        //         return Some((age == e.age, e.entry.unwrap()));
        //     } else { None }

    }

}

