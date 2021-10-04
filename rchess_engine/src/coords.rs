
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

impl D {
    pub fn shift(&self) -> i8 {
        match *self {
            D::N  => 8,
            D::NE => 9,
            D::E  => 1,
            D::SE => -7,
            D::S  => -8,
            D::SW => -9,
            D::W  => -1,
            D::NW => 7,
        }
    }
}

