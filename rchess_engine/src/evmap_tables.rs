
use crate::types::*;

use std::hash::Hash;
use std::sync::Arc;

use evmap::ShallowCopy;
use evmap::{ReadHandle,ReadHandleFactory,WriteHandle};
use evmap_derive::ShallowCopy;
use parking_lot::Mutex;


pub type FxBuildHasher = core::hash::BuildHasherDefault<rustc_hash::FxHasher>;

pub type EVReadFactory<T> = ReadHandleFactory<Zobrist, T, (), FxBuildHasher>;
pub type EVRead<T>        = ReadHandle<Zobrist, T, (), FxBuildHasher>;
pub type EVWrite<T>       = Arc<Mutex<WriteHandle<Zobrist, T, (), FxBuildHasher>>>;


pub fn new_hash_table<T: Eq + Hash + ShallowCopy>() -> (EVReadFactory<T>, EVWrite<T>) {
    let (r, w) = evmap::Options::default()
        .with_hasher(FxBuildHasher::default())
        .construct();
    let r = w.factory();
    let w = Arc::new(Mutex::new(w));
    (r,w)
}

