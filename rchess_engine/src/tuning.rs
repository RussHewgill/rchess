
use crate::evaluate::TaperedScore;
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;

use serde::{Serialize,Deserialize};

/// Passed bonus = passed * ranks past 2nd
// #[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
#[derive(Serialize,Deserialize,Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct EvPawn {
    pub backward: TaperedScore,
    pub doubled:  TaperedScore,
    pub isolated: TaperedScore,
    pub passed:   TaperedScore,
}

// #[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
#[derive(Serialize,Deserialize,Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct EvRook {
    pub rank7: TaperedScore,
}

// #[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
#[derive(Serialize,Deserialize,Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct EvKnight {
}

// #[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
#[derive(Serialize,Deserialize,Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct EvBishop {
}

// #[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
#[derive(Serialize,Deserialize,Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct EvQueen {
}

// #[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
#[derive(Serialize,Deserialize,Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct EvKing {
}

impl EvPawn {
    pub fn new() -> Self {
        Self {
            backward: TaperedScore::new(-10, -10),
            doubled:  TaperedScore::new(-15, -15),
            isolated: TaperedScore::new(-20, -20),
            passed:   TaperedScore::new(5,   10),
        }
    }
}

impl EvRook {
    pub fn new() -> Self {
        Self {
            rank7: TaperedScore::new(20,40),
        }
    }
}

impl EvKnight {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl EvBishop {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl EvQueen {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl EvKing {
    pub fn new() -> Self {
        Self {
        }
    }
}


