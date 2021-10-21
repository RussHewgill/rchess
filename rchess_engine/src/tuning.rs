
use crate::evaluate::TaperedScore;
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;


// pub static ROOK_7TH_RANK: TaperedScore = TaperedScore::new(20,40);

/// Passed bonus = passed * ranks past 2nd
#[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct EvPawn {
    pub backward: Score,
    pub doubled:  Score,
    pub isolated: Score,
    pub passed:   Score,
}

impl EvPawn {
    pub fn new() -> Self {
        Self {
            backward: -10,
            doubled:  -15,
            isolated: -20,
            passed:   5,
        }
    }
}

#[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct EvRook {
}

impl EvRook {
    pub fn new() -> Self {
        Self {
        }
    }
}

#[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct EvKnight {
}

impl EvKnight {
    pub fn new() -> Self {
        Self {
        }
    }
}

#[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct EvBishop {
}

impl EvBishop {
    pub fn new() -> Self {
        Self {
        }
    }
}

#[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct EvQueen {
}

impl EvQueen {
    pub fn new() -> Self {
        Self {
        }
    }
}

#[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct EvKing {
}

impl EvKing {
    pub fn new() -> Self {
        Self {
        }
    }
}


