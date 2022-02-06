
use crate::types::Score;
use crate::{trans_table::Node, explore::PackedSearchInfo};
use crate::explore::SearchInfo;
use crate::hashing::Zobrist;

use derive_new::new;
use parking_lot::RwLock;

use aligned::{Aligned,A64,A32};

use std::cell::UnsafeCell;
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
    // buf:           Aligned<A64,Vec<Bucket>>,
    // buf:           Aligned<A32,Vec<Bucket>>,
    // buf:           UnsafeCell<Aligned<A64,Vec<Bucket>>>,
    // buf:           UnsafeCell<Vec<Bucket>>,
    num_buckets:   usize,
    used_entries:  AtomicUsize,
    cycles:        AtomicU8,
}

unsafe impl Send for TransTable {}
unsafe impl Sync for TransTable {}

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

        // let buf = Aligned(buf);
        // let buf = UnsafeCell::new(buf);

        Self {
            buf,
            num_buckets,
            used_entries:  AtomicUsize::new(0),
            cycles:        AtomicU8::new(0),
        }
    }

    pub fn insert(&self, zb: Zobrist, meval: Option<Score>, msi: Option<SearchInfo>) {
        let (idx,ver) = self.calc_index(zb);

        if let Some(bucket) = self.buf.get(idx) {
            bucket.store(ver, meval, msi, &self.used_entries, &self.cycles);
        } else {
            panic!("TT insert, bad bucket idx: {:?}, zb = {:?}", idx, zb);
        }

        // if let Some(bucket) = unsafe { (*self.buf.get()).get_mut(idx) } {
        //     bucket.store(ver, si, &self.used_entries, &self.cycles);
        // } else {
        //     panic!("TT insert, bad bucket idx: {:?}, zb = {:?}", idx, zb);
        // }

    }

    // pub fn probe(&self, zb: Zobrist) -> Option<(bool,SearchInfo)> {
    pub fn probe(&self, zb: Zobrist) -> (Option<TTEval>, Option<SearchInfo>) {
        let (idx,ver) = self.calc_index(zb);

        if let Some(bucket) = self.buf.get(idx) {
            bucket.find(ver, &self.cycles)
        } else {
            panic!("TT probe, bad bucket idx: {:?}, zb = {:?}", idx, zb);
        }

        // if let Some(bucket) = unsafe { (*self.buf.get()).get(idx) } {
        //     bucket.find(ver, &self.cycles)
        // } else {
        //     panic!("TT probe, bad bucket idx: {:?}, zb = {:?}", idx, zb);
        // }

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
        let key = (key % total) as usize;
        let ver = (zb.0 & LOW_FOUR_BYTES) as u32;
        (key,ver)
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

        // unsafe {
        //     for bucket in (*self.buf.get()).iter_mut() {
        //         bucket.clear();
        //     }
        // }

        self.used_entries.store(0, std::sync::atomic::Ordering::SeqCst);
        self.cycles.store(0, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn increment_cycle(&self) {
        self.cycles.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

}

/// Queries
impl TransTable {

    pub fn used_entries_eval_si(&self) -> (usize,usize,usize) {

        let mut used_total = 0;
        let mut used_eval  = 0;
        let mut used_si    = 0;

        for bucket in self.buf.iter() {

            #[cfg(feature = "unsafe_tt")]
            let bucket: &[TTEntry; ENTRIES_PER_BUCKET] = unsafe { &*bucket.bucket.get() };

            #[cfg(not(feature = "unsafe_tt"))]
            let bucket = bucket.bucket.read();

            for e in bucket.iter() {
                if let Some(e) = e.entry {
                    used_total += 1;

                    match (e.get_eval(), e.get_searchinfo()) {
                        (TTEval::Check | TTEval::Some(_),None) => used_eval += 1,
                        (_,Some(si))                           => used_si += 1,
                    }
                }
            }
        }

        (used_total,used_eval,used_si)
    }

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
            // let ptr = (*self.buf.get()).as_ptr().add(idx);
            crate::prefetch::prefetch_write(ptr);
        }
    }
}

#[derive(Debug,Default,Clone,Copy,new)]
pub struct TTEntry {
    age:                u8,
    // entry:              Option<(u32,SearchInfo)>,
    entry:              Option<TTEntry2>,
}

#[derive(Debug,Clone,Copy)]
pub enum TTEntry2 {
    Eval { ver: u32, eval: TTEval },
    // SI   { ver: u32, si: SearchInfo },
    Both { ver: u32, eval: TTEval, si: SearchInfo },
}

#[derive(Debug,Clone,Copy)]
pub enum TTEval {
    // None,
    Check,
    Some(Score),
}

impl TTEval {
    pub fn to_option(self) -> Option<Score> {
        if let Self::Some(score) = self {
            Some(score)
        } else { None }
    }
}

/// New
impl TTEntry2 {
    pub fn new(ver: u32, meval: Option<Score>, msi: Option<SearchInfo>) -> Option<Self> {
        match (meval,msi) {
            (Some(eval),None)     => Some(Self::Eval { ver, eval: TTEval::Some(eval) }),
            (None,Some(si))       => Some(Self::Both { ver, eval: TTEval::Check, si }),
            (Some(eval),Some(si)) => Some(Self::Both { ver, eval: TTEval::Some(eval), si }),
            (None,None)           => None,
        }
    }
}

/// Getters
impl TTEntry {
    pub fn get_searchinfo(self) -> Option<SearchInfo> {
        // self.entry.map(|x| x.get_searchinfo()).flatten()
        self.entry.and_then(|x| x.get_searchinfo())
    }

    pub fn get_eval(self) -> Option<TTEval> {
        self.entry.map(|x| x.get_eval())
    }

    pub fn get_ver(&self) -> Option<u32> {
        self.entry.map(|x| x.get_ver())
    }
}

/// Getters
impl TTEntry2 {

    pub fn get_searchinfo(self) -> Option<SearchInfo> {
        match self {
            Self::Eval { .. }     => None,
            Self::Both { si, .. } => Some(si),
        }
    }

    pub fn get_eval(self) -> TTEval {
        match self {
            Self::Eval { eval, .. } => eval,
            Self::Both { eval, .. } => eval,
        }
    }

    pub fn get_ver(self) -> u32 {
        match self {
            Self::Eval { ver, .. } => ver,
            Self::Both { ver, .. } => ver,
        }
    }
}

// #[derive(Debug)]
// // #[repr(align(64))]
// pub struct Bucket {
//     bucket: [RwLock<TTEntry>; ENTRIES_PER_BUCKET],
// }

#[derive(Debug)]
// #[repr(align(64))]
// #[repr(align(32))] // XXX: this is faster than 64 for some reason ??
pub struct Bucket {

    #[cfg(feature = "unsafe_tt")]
    bucket: UnsafeCell<[TTEntry; ENTRIES_PER_BUCKET]>,
    #[cfg(not(feature = "unsafe_tt"))]
    bucket: RwLock<[TTEntry; ENTRIES_PER_BUCKET]>,

    // bucket: [RwLock<TTEntry>; ENTRIES_PER_BUCKET],
    // bucket: RwLock<Aligned<A64,[TTEntry; ENTRIES_PER_BUCKET]>>, // XXX: much slower
    // bucket: [TTEntry; ENTRIES_PER_BUCKET],
}

/// New, clear
impl Bucket {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "unsafe_tt")]
            bucket: UnsafeCell::new(array_init::array_init(|_| TTEntry::default())),
            #[cfg(not(feature = "unsafe_tt"))]
            bucket: RwLock::new(array_init::array_init(|_| TTEntry::default())),
        }
    }

    pub fn clear(&self) {

        #[cfg(feature = "unsafe_tt")]
        let mut bucket: &mut [TTEntry; ENTRIES_PER_BUCKET] = unsafe { &mut *self.bucket.get() };
        #[cfg(feature = "unsafe_tt")]
        for e in bucket.iter_mut() {
            *e = TTEntry::default();
        }

        #[cfg(not(feature = "unsafe_tt"))]
        for e in self.bucket.write().iter_mut() {
            *e = TTEntry::default();
        }

        // for e in self.bucket.write().iter_mut() {
        //     *e = TTEntry::default();
        // }

        // for e in self.bucket.iter() {
        //     let mut w = e.write();
        //     *w = TTEntry::default();
        // }

    }
}

/// store, find
impl Bucket {

    pub fn store(
        &self,
        ver:              u32,
        meval:            Option<Score>,
        msi:              Option<SearchInfo>,
        used_entries:     &AtomicUsize,
        age:              &AtomicU8,
    ) {

        assert!(meval.is_some() || msi.is_some());

        let mut idx_lowest_depth = None;


        #[cfg(feature = "unsafe_tt")]
        let bucket: &[TTEntry; ENTRIES_PER_BUCKET] = unsafe { &*self.bucket.get() };

        #[cfg(not(feature = "unsafe_tt"))]
        let bucket = self.bucket.read();

        for (entry_idx,e) in bucket.iter().enumerate() {
            if let Some(ver2) = e.get_ver() {
                match e.get_searchinfo() {
                    Some(e_si) => {
                        if let Some(si) = msi {
                            if e_si.depth_searched < si.depth_searched
                                // && e_si.node_type != Node::Exact
                            {
                                idx_lowest_depth = Some(entry_idx);
                            }
                        }
                    },
                    None => {
                        idx_lowest_depth = Some(entry_idx);
                        /// only eval is stored, so this slot is best to overwrite
                        break;
                    },
                }
            }
        }
        drop(bucket);

        let idx = if let Some(idx) = idx_lowest_depth {
            used_entries.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            idx
        } else { 0 };

        let age = age.load(std::sync::atomic::Ordering::Relaxed);

        #[cfg(feature = "unsafe_tt")]
        let mut bucket: &mut [TTEntry; ENTRIES_PER_BUCKET] = unsafe { &mut *self.bucket.get() };
        #[cfg(feature = "unsafe_tt")]
        let mut w = bucket.get_mut(idx).unwrap();

        #[cfg(not(feature = "unsafe_tt"))]
        let mut w = self.bucket.write();
        #[cfg(not(feature = "unsafe_tt"))]
        let mut w = w.get_mut(idx).unwrap();

        let new = TTEntry::new(age, TTEntry2::new(ver, meval, msi));
        *w = new;

    }

    #[cfg(feature = "nope")]
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
        // for (entry_idx,e) in self.bucket.iter().enumerate() {
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

        // self.bucket[idx] = TTEntry::new(age, Some((ver,si)));

        // unimplemented!()

    }

    pub fn find(&self, ver: u32, age: &AtomicU8) -> (Option<TTEval>,Option<SearchInfo>) {

        #[cfg(feature = "unsafe_tt")]
        let bucket: &[TTEntry; ENTRIES_PER_BUCKET] = unsafe { &*self.bucket.get() };
        #[cfg(feature = "unsafe_tt")]
        for (entry_idx,e) in bucket.iter().enumerate() {
            if let Some(ver2) = e.get_ver() {
                if ver2 == ver {
                    return (e.get_eval(),e.get_searchinfo());
                }
            }
        }

        #[cfg(not(feature = "unsafe_tt"))]
        for (entry_idx,e) in self.bucket.read().iter().enumerate() {
            if let Some(ver2) = e.get_ver() {
                if ver2 == ver {
                    return (e.get_eval(),e.get_searchinfo());
                }
            }
        }

        (None,None)
    }

    // pub fn find(&self, ver: u32, age: &AtomicU8) -> Option<(bool,SearchInfo)> {
    #[cfg(feature = "nope")]
    pub fn find(&self, ver: u32, age: &AtomicU8) -> (Option<Score>,Option<SearchInfo>) {

        for (entry_idx,e) in self.bucket.read().iter().enumerate() {
        // for (entry_idx,e) in self.bucket.iter().enumerate() {
            if let Some((ver2,ee)) = e.entry {
                if ver2 == ver {
                    let age = age.load(std::sync::atomic::Ordering::Relaxed);
                    // return Some((age == e.age, ee));
                    unimplemented!()
                }
            }
        }
        (None,None)

        // if let Some(e) = self.bucket.read().iter()
        //     .filter(|e| e.entry.is_some())
        //     .max_by_key(|e| e.entry.unwrap().depth_searched) {
        //         let age = age.load(std::sync::atomic::Ordering::Relaxed);
        //         return Some((age == e.age, e.entry.unwrap()));
        //     } else { None }

    }

}

