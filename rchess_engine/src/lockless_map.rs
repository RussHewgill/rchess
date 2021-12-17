
use crate::types::*;
use crate::trans_table::Node;
use crate::evaluate::Score;
use crate::explore::SearchInfo;
use crate::hashing::Zobrist;

pub use prev_rustic_nothread::*;

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

// #[cfg(feature = "nope")]
mod prev_rustic_nothread {
    use super::*;

    const ENTRIES_PER_BUCKET: usize = 4;

    const MEGABYTE: usize = 1024 * 1024;
    const HIGH_FOUR_BYTES: u64 = 0xFF_FF_FF_FF_00_00_00_00;
    const LOW_FOUR_BYTES: u64 = 0x00_00_00_00_FF_FF_FF_FF;
    const SHIFT_TO_LOWER: u64 = 32;

    #[derive(Debug,Eq,PartialEq,PartialOrd,Hash,Clone,Copy)]
    pub struct TTEntry {
        pub verification:       u32,
        // pub partial_key:        u16,
        // pub best_move:          [u8; 2],
        // pub depth_searched:     Depth, // u8
        // pub node_type:          Node, // 1
        // pub score:              Score, // 4
        pub entry:              SearchInfo,
    }

    impl TTEntry {
        pub fn new() -> Self {
            Self {
                verification:  0,
                entry:         SearchInfo::empty(),
            }
        }
    }

    #[derive(Debug,Eq,PartialEq,PartialOrd,Hash,Clone,Copy)]
    struct Bucket {
        bucket: [TTEntry; ENTRIES_PER_BUCKET],
    }

    /// New
    impl Bucket {
        pub fn new() -> Self {
            Self {
                bucket: [TTEntry::new(); ENTRIES_PER_BUCKET],
            }
        }
    }

    impl Bucket {
        pub fn store(&mut self, ver: u32, si: SearchInfo, used_entries: &mut usize) {
            let mut idx_lowest_depth = 0;

            unimplemented!()
        }

        pub fn find(&self, ver: u32) -> Option<&SearchInfo> {
            unimplemented!()
        }

    }

    pub struct TransTable {
        tt:            Vec<Bucket>,
        megabytes:     usize,
        used_entries:  usize,
        tot_buckets:   usize,
        tot_entries:   usize,
    }

    /// New
    impl TransTable {
        pub fn new(megabytes: usize) -> Self {
            let e_size = std::mem::size_of::<TTEntry>();
            let b_size = e_size * ENTRIES_PER_BUCKET;
            let tot_buckets = MEGABYTE / b_size * megabytes;
            let tot_entries = tot_buckets * ENTRIES_PER_BUCKET;
            Self {
                tt: vec![Bucket::new(); tot_buckets],
                megabytes,
                used_entries: 0,
                tot_buckets,
                tot_entries,
            }
        }
    }

    impl TransTable {
        pub fn insert(&mut self, zb: Zobrist, si: SearchInfo) {
            let idx = self.calc_index(zb);
            let ver = self.calc_verification(zb);
            self.tt[idx].store(ver, si, &mut self.used_entries)
        }
    }

    impl TransTable {
        fn calc_index(&self, zb: Zobrist) -> usize {
            let key = (zb.0 & HIGH_FOUR_BYTES) >> SHIFT_TO_LOWER;
            let total = self.tot_buckets as u64;
            (key % total) as usize
        }
        fn calc_verification(&self, zb: Zobrist) -> u32 {
            (zb.0 & LOW_FOUR_BYTES) as u32
        }
    }

}

