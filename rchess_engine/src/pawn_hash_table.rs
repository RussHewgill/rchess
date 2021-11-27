
use crate::types::*;
use crate::tables::*;

use derive_new::new;

use serde::{Serialize,Deserialize};
use evmap_derive::ShallowCopy;

#[derive(Debug,Eq,PartialEq,Hash,ShallowCopy,Clone,Copy,Serialize,Deserialize,new)]
pub struct PHEntry {

    pub connected:            BitBoard,
    pub supported_1:          BitBoard,
    pub supported_2:          BitBoard,
    pub passed:               BitBoard,
    pub candidate:            BitBoard,

    pub blocked:              BitBoard,
    pub doubled:              BitBoard,
    pub isolated:             BitBoard,
    pub doubled_isolated:     BitBoard,
    pub backward:             BitBoard,

}



