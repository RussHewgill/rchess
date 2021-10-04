
use crate::types::*;


#[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub enum D {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct Coord(pub u8, pub u8);

impl Coord {
    pub fn shift(&self, d: D) -> BitBoard {
        unimplemented!()
    }
}

