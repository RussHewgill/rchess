
use crate::types::*;
use crate::trans_table::Node;
use crate::explore::SearchInfo;
use crate::hashing::Zobrist;

use derive_new::new;
use parking_lot::RwLock;

// use std::alloc::{Layout, handle_alloc_error, self};
// use std::ptr::NonNull;
// use std::cell::{UnsafeCell, Cell};

// pub const CLUSTER_SIZE: usize = 3;

// #[derive(Debug,Eq,PartialEq,PartialOrd,Hash,Clone,Copy)]
// pub struct TTEntry {
//     pub verification:       u32,
//     pub entry:              SearchInfo,
// }

// impl TTEntry {
//     pub fn new() -> Self {
//         Self {
//             verification:  0,
//             entry:         SearchInfo::empty(),
//         }
//     }
// }

// pub struct Cluster {
//     pub entry:   [TTEntry; CLUSTER_SIZE],
//     pub padding: [u8; 2],
// }

// pub struct TransTable {}

use std::cell::{UnsafeCell, Cell};
use std::ptr::NonNull;
use std::mem;
use std::alloc::{Layout, handle_alloc_error, self};
use std::sync::atomic::{AtomicUsize, AtomicU8};
// use std::sync::atomic::AtomicUsize;

use super::*;

/// https://www.chess2u.com/t1820-setting-correct-hashtable-size?highlight=hash+size
/// HT[KB] = 2.0 * PFreq[MHz] * t[s]

// pub const DEFAULT_TT_SIZE_MB: usize = 1024;
// pub const DEFAULT_TT_SIZE_MB: usize = 256;
// pub const DEFAULT_TT_SIZE_MB: usize = 128;
// pub const DEFAULT_TT_SIZE_MB: usize = 64;
pub const DEFAULT_TT_SIZE_MB: usize = 32;
// pub const DEFAULT_TT_SIZE_MB: usize = 16;

// const ENTRIES_PER_BUCKET: usize = 3;
const ENTRIES_PER_BUCKET: usize = 2;

const KILOBYTE: usize = 1024;
const MEGABYTE: usize = 1024 * KILOBYTE;

const HIGH_FOUR_BYTES: u64 = 0xFF_FF_FF_FF_00_00_00_00;
const LOW_FOUR_BYTES: u64 = 0x00_00_00_00_FF_FF_FF_FF;
const SHIFT_TO_LOWER: u64 = 32;

#[derive(Debug,Eq,PartialEq,PartialOrd,Hash,Clone,Copy,new)]
pub struct TTEntry {
    // verification:       u32,
    age:                u8,
    entry:              Option<SearchInfo>,
    // pub partial_key:        u16,
    // pub best_move:          [u8; 2],
    // pub depth_searched:     Depth, // u8
    // pub node_type:          Node, // 1
    // pub score:              Score, // 4
    // pub entry:              SearchInfo,
}

/// Empty
impl TTEntry {
    pub fn empty() -> Self {
        Self {
            // lock:          AtomicBool::new(false),
            // verification:  0,
            age:           0,
            // entry:         SearchInfo::empty(),
            entry:         None,
        }
    }
}

// #[derive(Debug,Eq,PartialEq,PartialOrd,Hash,Clone,Copy)]
// // #[repr(align(64))]
// // #[repr(align(32))]
// pub struct Bucket {
//     // pub x:  usize,
//     bucket: [TTEntry; ENTRIES_PER_BUCKET],
// }

#[derive(Debug)]
// #[repr(align(64))]
pub struct Bucket {
    bucket: [RwLock<TTEntry>; ENTRIES_PER_BUCKET],
}

/// New, clear
impl Bucket {
    pub fn new() -> Self {
        use rand::Rng;
        Self {
            // x:      rand::thread_rng().gen(),
            // bucket: [TTEntry::empty(); ENTRIES_PER_BUCKET],
            bucket: array_init::array_init(|_| RwLock::new(TTEntry::empty())),
        }
    }

    pub fn clear(&self) {
        for e in self.bucket.iter() {
            let mut w = e.write();
            *w = TTEntry::empty();
        }
    }
}

/// store, find
// #[cfg(feature = "nope")]
impl Bucket {

    // pub fn store(&self, ver: u32, si: SearchInfo, used_entries: &mut usize) {
    pub fn store(&self, ver: u32, si: SearchInfo, used_entries: &AtomicUsize, age: &AtomicU8) {

        // let age = age.load(std::sync::atomic::Ordering::Relaxed);
        let mut idx_lowest_depth = 0;

        for entry_idx in 1..ENTRIES_PER_BUCKET {

            let e = self.bucket[entry_idx].read();
            if let Some(entry) = e.entry {
            // if let Some(entry) = self.bucket[entry_idx].entry {
                if entry.depth_searched < si.depth_searched {
                    idx_lowest_depth = entry_idx;
                }
            }

            // if self.bucket[entry_idx].entry.depth_searched < si.depth_searched {
            //     idx_lowest_depth = entry_idx;
            // }

        }

        if self.bucket[idx_lowest_depth].read().verification == 0 {
        // if self.bucket[idx_lowest_depth].verification == 0 {
            // *used_entries += 1;
            used_entries.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }

        // // self.bucket[idx_lowest_depth] = TTEntry::new(ver, si);
        // self.bucket[idx_lowest_depth] = TTEntry::new(ver, Some(si));

        let age = age.load(std::sync::atomic::Ordering::Relaxed);
        let mut w = self.bucket[idx_lowest_depth].write();
        *w = TTEntry::new(ver, age, Some(si));

    }

    // pub fn find(&self, ver: u32) -> Option<&SearchInfo> {
    // pub fn find(&self, ver: u32, age: &AtomicU8) -> Option<SearchInfo> {
    pub fn find(&self, ver: u32, age: &AtomicU8) -> Option<(bool,SearchInfo)> {

        // for e in self.bucket.iter() {
        //     let e = e.read();
        //     if let Some(ee) = e.entry {
        //         let x = ee.depth_searched - 4 * (age - e.age);
        //         // idx_best = idx_best.max(x);
        //         if x > best {
        //             best = 
        //             idx_best = x;
        //         }
        //     }
        //     // let x = e.
        //     // if e.read().age 
        // }


        for e in self.bucket.iter(){

            // if let Some(si) = e.entry {
            //     /// e.key ^ e.data
            //     let ver2 = TransTable::verify(&si, e.verification);
            //     if ver2 == ver {
            //         return e.entry.as_ref();
            //     } else {
            //         panic!("wat: si = {:?}", si);
            //         // return None;
            //     }
            // }

            // } else if e.verification == ver {
            //     // return e.entry.as_ref();
            //     return None;
            // }

            let e = e.read();
            if e.verification == ver {

                if let Some(ee) = e.entry {
                    let age = age.load(std::sync::atomic::Ordering::Relaxed);
                    return Some((age == e.age, ee));
                }

                // // return e.entry.as_ref();

                // if let Some(si) = e.entry {
                //     return Some(si);
                // }
                // return Some(&e.entry);
            }

        }
        None
    }
}

#[cfg(feature = "nope")]
impl Bucket {

    pub fn store(&self, ver: u32, si: SearchInfo, used_entries: &AtomicUsize, age: &AtomicU8) {
        unimplemented!()
    }

    pub fn find(&self, ver: u32, age: &AtomicU8) -> Option<(bool,SearchInfo)> {
        unimplemented!()
    }

}

// #[derive(Clone)]
/// XXX: TT
pub struct TransTable {
    vec:           Vec<Bucket>,
    megabytes:     usize,
    used_entries:  AtomicUsize,
    tot_buckets:   usize,
    tot_entries:   usize,
    cycles:        AtomicU8,
    hashmask:      u64,
}

/// Not actually safe, but at least it's a bit faster
unsafe impl Send for TransTable {}
unsafe impl Sync for TransTable {}

/// New
impl TransTable {

    pub fn new_mb(megabytes: usize) -> Self {
        assert!(megabytes > 0);

        let mut tot_buckets: usize = (megabytes * MEGABYTE) / mem::size_of::<Bucket>();
        tot_buckets = tot_buckets.next_power_of_two() / 2;

        let e_size = std::mem::size_of::<TTEntry>();
        let b_size = e_size * ENTRIES_PER_BUCKET;
        let tot_entries = tot_buckets * ENTRIES_PER_BUCKET;

        let mut v = vec![];
        for _ in 0..tot_buckets {
            v.push(Bucket::new());
        }

        let vec = v;

        eprintln!("tot_buckets = {:b}", tot_buckets);
        eprintln!("tot_entries = {:b}", tot_entries);

        let hashmask = 1;

        Self {
            vec,
            megabytes:    megabytes,
            used_entries: AtomicUsize::new(0),
            tot_buckets:  tot_buckets,
            tot_entries:  tot_entries,
            cycles:       AtomicU8::new(0),
            hashmask,
        }
    }

}

/// Clear, increment cycle counter
impl TransTable {

    pub fn clear_table(&self) {
        for bucket in self.vec.iter() {
            bucket.clear();
        }
        self.used_entries.store(0, std::sync::atomic::Ordering::SeqCst);
        self.cycles.store(0, std::sync::atomic::Ordering::SeqCst);
    }

    // pub unsafe fn clear_table(&self) {
    //     let mut vec = &mut *self.vec.get();
    //     for b in vec.iter_mut() {
    //         *b = Bucket::new();
    //     }
    // }

    pub fn increment_cycle(&self) {
        self.cycles.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

}

/// Resize
impl TransTable {
    pub fn resize_mb(&mut self, size: usize) {
        let new = Self::new_mb(size);
        *self = new;
    }
}

/// Verify
impl TransTable {

    pub unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
        ::std::slice::from_raw_parts(
            (p as *const T) as *const u8,
            ::std::mem::size_of::<T>(),
        )
    }

    pub fn get_ver2(si: &SearchInfo) -> &[u32] {
        const D: usize = std::mem::size_of::<SearchInfo>();
        use std::convert::TryInto;

        let data: &[u8] = unsafe {
            // std::slice::from_raw_parts(si as *const u8, D)
            Self::any_as_u8_slice(si)
        };

        // let data: Vec<u8> = bincode::serialize(&si).unwrap();
        // eprintln!("data.len() = {:?}", data.len());
        let data: &[u32] = bytemuck::cast_slice(&data);

        data
    }

    pub fn verify(si: &SearchInfo, mut ver: u32) -> u32 {
        let data = Self::get_ver2(si);

        Self::_verify(data, ver)
    }

    pub fn _verify(si: &[u32], mut ver: u32) -> u32 {
        for d in si {
            ver ^= d;
        }
        ver
    }

}

/// Insert
impl TransTable {
    pub fn insert(&self, zb: Zobrist, si: SearchInfo) {
        let idx: usize = self.calc_index(zb);
        let ver: u32   = self.calc_verification(zb);

        // let ver = Self::verify(&si, ver);

        let ptr = self.bucket(idx).unwrap();
        ptr.store(ver, si, &self.used_entries, &self.cycles);

        // unsafe {
        //     let ptr = self.bucket(idx).unwrap();
        //     // let ptr = self.bucket(idx);
        //     // let mut used_entries: &mut usize = &mut (*self.used_entries.get());
        //     // let mut used_entries: &mut usize = &mut (*self.used_entries.get());
        //     // (*ptr).store(ver, si, used_entries);
        //     ptr.store(ver, si, &self.used_entries);
        // }

    }
}

/// Probe
impl TransTable {

    // // unsafe fn bucket(&self, idx: usize) -> *mut Bucket {
    // unsafe fn bucket(&self, idx: usize) -> Option<&mut Bucket> {
    //     // (*self.ptr.get()).as_ptr()
    //     // .add(idx)
    //     (*self.vec.get()).get_mut(idx)
    // }

    fn bucket(&self, idx: usize) -> Option<&Bucket> {
        self.vec.get(idx)
    }

    // pub fn probe(&self, zb: Zobrist) -> Option<&SearchInfo> {
    // pub fn probe(&self, zb: Zobrist) -> Option<SearchInfo> {
    pub fn probe(&self, zb: Zobrist) -> Option<(bool,SearchInfo)> {
        let idx = self.calc_index(zb);
        let ver = self.calc_verification(zb);

        // unsafe {
        //     let ptr = self.bucket(idx)?;
        //     let si = ptr.find(ver)?;
        //     Some(si)
        //     // Some(&si)
        //     // let ptr = self.bucket(idx);
        //     // (*ptr).find(ver)
        // }

        let ptr = self.bucket(idx)?;
        let (same_age, si) = ptr.find(ver, &self.cycles)?;
        Some((same_age, si))

    }

}

/// Query
impl TransTable {

    pub fn total_entries(&self) -> usize {
        // self.tot_entries.get()
        self.tot_entries
    }

    pub fn used_entries(&self) -> usize {

        self.used_entries.load(std::sync::atomic::Ordering::Relaxed)

        // unsafe {
        //     *self.used_entries.get()
        // }

    }

    // pub fn bucket_count(&self) -> usize {
    // }
}

/// Calc index, ver
impl TransTable {

    pub fn calc_index2(&self, zb: Zobrist) -> usize {

        // const SLOT_NB = 1024;
        // const BLOCK_SIZE = 4;

        // let address = zb.0 & (1024 - 1) ^ 

        // slotNb == 1024(powerOf2), blocksize== 4
        // address = key & (1024-1) ^ 0, 1, 2, 3;

        unimplemented!()
    }

    // /// Stockfish ??
    // pub fn calc_index2(&self, zb: Zobrist) -> usize {
    //     let key = (zb.0 as u128 * self.tot_buckets.get() as u128) >> 64;
    //     key as usize
    // }

    pub fn calc_index(&self, zb: Zobrist) -> usize {
        let key = (zb.0 & HIGH_FOUR_BYTES) >> SHIFT_TO_LOWER;
        // let total = self.tot_buckets.get() as u64;
        let total = self.tot_buckets as u64;
        (key % total) as usize
    }
    pub fn calc_verification(&self, zb: Zobrist) -> u32 {
        (zb.0 & LOW_FOUR_BYTES) as u32
    }
}

/// Prefetch
impl TransTable {

    // pub fn prefetch(&self, zb: Zobrist) {
    //     let idx = self.calc_index(zb);
    //     unsafe {
    //         let ptr = (*self.vec.get()).as_ptr().add(idx);
    //         crate::prefetch::prefetch_write(ptr);
    //     }
    // }

    pub fn prefetch(&self, zb: Zobrist) {
        let idx = self.calc_index(zb);

        unsafe {
            // let ptr = (*self.vec.get()).as_ptr().add(idx);
            let ptr = self.vec.as_ptr().add(idx);
            crate::prefetch::prefetch_write(ptr);
        }

    }

}

impl std::fmt::Debug for TransTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("TransTable"))?;
        Ok(())
    }
}


#[cfg(feature = "nope")]
mod prev_pleco {
    use super::*;

    const TT_ALLOC_SIZE: usize = std::mem::size_of::<TransTable>();
    type DummyTT = [u8; TT_ALLOC_SIZE];
    static mut TT_TABLE: DummyTT = [0; TT_ALLOC_SIZE];


    pub const CLUSTER_SIZE: usize = 3;

    #[derive(Debug,Eq,PartialEq,Hash,Clone,Copy)]
    pub struct Entry {
        pub partial_key:        u16,
        // pub best_move:          Move,
        // pub best_move:          PackedMove,
        pub best_move:          [u8; 2],
        pub depth_searched:     Depth,
        pub node_type:          Node,
        pub score:              Score,
    }

    impl Entry {

        pub fn is_empty(&self) -> bool {
            self.partial_key == 0
        }

        pub fn place(
            &mut self,
            zb:              Zobrist,
            // best_move:       Move,
            best_move:       [u8; 2],
            depth_searched:  Depth,
            node_type:       Node,
            score:           Score,
        ) {
            let partial_key = zb.0.wrapping_shr(48) as u16;

            if partial_key != self.partial_key {
                self.best_move = best_move;
            }

            if partial_key != self.partial_key
                && depth_searched > self.depth_searched
            {
                self.partial_key    = partial_key;
                self.score          = score;
                self.node_type      = node_type;
                self.depth_searched = depth_searched;
            }
        }

    }

    #[repr(C)]
    pub struct Cluster {
        // pub entry:   [SearchInfo; CLUSTER_SIZE],
        pub entry:   [Entry; CLUSTER_SIZE],
        pub padding: [u8; 2],
    }

    pub struct TransTable {
        clusters:   UnsafeCell<NonNull<Cluster>>,
        cap:        UnsafeCell<usize>,
        time_age:   UnsafeCell<u8>,
    }

    /// Construction
    impl TransTable {

        pub fn init_tt() {
            unsafe {
                let tt = &mut TT_TABLE as *mut DummyTT as *mut TransTable;
                std::ptr::write(tt, TransTable::new(256));
            }
        }

        #[inline(always)]
        pub fn global() -> &'static TransTable {
            unsafe {
                &*(&TT_TABLE as *const DummyTT as *const TransTable)
            }
        }

        pub fn new(mb_size: usize) -> Self {
            let mut num_clusters: usize = (mb_size * 1024 * 1024) / std::mem::size_of::<Cluster>();
            Self::new_num_clusters(num_clusters)
        }

        pub fn new_num_clusters(num_clusters: usize) -> Self {
            Self::create(num_clusters.next_power_of_two())
        }

        fn create(size: usize) -> Self {
            assert_eq!(size.count_ones(), 1);
            TransTable {
                clusters:   UnsafeCell::new(alloc_clusters(size)),
                cap:        UnsafeCell::new(size),
                time_age:   UnsafeCell::new(0),
            }
        }
    }

    /// Probe
    impl TransTable {

        pub fn probe(&self, zb: &Zobrist) -> (bool, &mut Entry) {
            let partial_key = zb.0.wrapping_shr(48) as u16;

            let cluster: *mut Cluster = self.cluster(zb);
            unsafe {
                let init_entry: *mut Entry = cluster_first_entry(cluster);

                for i in 0..CLUSTER_SIZE {
                    let entry_ptr: *mut Entry = init_entry.offset(i as isize);

                    let entry: &mut Entry = &mut (*entry_ptr);

                    if entry.partial_key == 0 || entry.partial_key == partial_key {
                        return (entry.partial_key != 0, entry)
                    }

                }

                let mut replacement: *mut Entry = init_entry;
                // let mut replacement_score: Score = (&*replacement).time_value(self.time_age());

            }

            panic!("duplicate zobrist, no room");
            // unimplemented!()
        }

        /// Returns the cluster of a given key.
        #[inline]
        fn cluster(&self, zb: &Zobrist) -> *mut Cluster {
            let index: usize = ((self.num_clusters() - 1) as u64 & zb.0) as usize;
            unsafe {
                (*self.clusters.get()).as_ptr().offset(index as isize)
            }
        }

    }

    /// Modify
    impl TransTable {
    }

    /// Misc
    impl TransTable {
        /// Returns the number of clusters the Transposition Table holds.
        #[inline(always)]
        pub fn num_clusters(&self) -> usize {
            unsafe {
                *self.cap.get()
            }
        }

        /// Returns the number of Entries the Transposition Table holds.
        #[inline(always)]
        pub fn num_entries(&self) -> usize {
            self.num_clusters() * CLUSTER_SIZE
        }

        // Called each time a new position is searched.
        #[inline]
        pub fn new_search(&self) {
            unsafe {
                let c = self.time_age.get();
                *c = (*c).wrapping_add(4);
            }
        }

        /// Returns the current time age of a TT.
        #[inline]
        pub fn time_age(&self) -> u8 {
            unsafe {
                *self.time_age.get()
            }
        }
    }

    fn alloc_clusters(size: usize) -> NonNull<Cluster> {
        let size = size * std::mem::size_of::<Cluster>();

        let layout = Layout::from_size_align(size, 2).unwrap();

        unsafe {
            let ptr: *mut u8 = alloc::alloc_zeroed(layout);
            let ptr2: NonNull<Cluster> = match NonNull::new(ptr) {
                Some(p) => p.cast(),
                _       => handle_alloc_error(layout),
            };
            ptr2
        }
    }

    unsafe fn cluster_first_entry(cluster: *mut Cluster) -> *mut Entry {
        (*cluster).entry.get_unchecked_mut(0) as *mut Entry
    }

}


