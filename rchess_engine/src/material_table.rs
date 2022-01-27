
use std::collections::HashMap;

use crate::endgame::*;
use crate::types::*;
use crate::tables::*;


#[derive(Debug,Default,Clone)]
pub struct MaterialTable {
    table:     HashMap<Zobrist, MatEval>
}

impl MaterialTable {

    pub fn get(&self, zb: Zobrist) -> Option<&MatEval> {
        self.table.get(&zb)
    }

    pub fn insert(&mut self, zb: Zobrist, v: MatEval) {
        self.table.insert(zb, v);
    }

}

#[derive(Debug,Clone)]
pub struct MatEval {
    pub score:     Score,
    pub phase:     Phase,
    // factor:    [ScaleFactor; 2],

    // eg_val:    Option<Box<dyn EndGame>>,
    // eg_scale:  Option<[Box<dyn EndGame>; 2]>,
    pub eg_val:    Option<EndGameType>,
    // eg_scale:  Option<[EndGameType; 2]>,

}

impl MatEval {
    pub fn new(g: &Game, score: Score) -> Self {
        Self {
            score,
            phase:     g.state.phase,
            // factor:    [ScaleFactor::Normal; 2],

            eg_val:    None,
            // eg_scale:  None,

        }
    }
}

// impl Evaluation {
//     pub fn score(&self) -> Score { self.score }
//     pub fn phase(&self) -> Phase { self.phase }
// }




