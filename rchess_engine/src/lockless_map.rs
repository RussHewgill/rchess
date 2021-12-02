
use crate::types::*;
use crate::trans_table::Node;
use crate::evaluate::Score;
use crate::explore::SearchInfo;
use crate::hashing::Zobrist;

use std::alloc::{Layout, handle_alloc_error, self};
use std::ptr::NonNull;
use std::cell::UnsafeCell;


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

