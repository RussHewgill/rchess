
use crate::types::*;
use crate::tables::*;



#[derive(Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct Evaluation {
    score:     Score,
    phase:     Phase,
}

impl Evaluation {

    pub fn score(&self) -> Score { self.score }

    pub fn phase(&self) -> Phase { self.phase }

}




