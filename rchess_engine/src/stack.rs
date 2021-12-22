
use crate::types::*;

use arrayvec::ArrayVec;

#[derive(Debug,Clone)]
pub struct ABStack {
    // pub history:        [[[Score; 64]; 64]; 2],
    pub history:        crate::heuristics::ButterflyHistory,
    // pub killers:        crate::heuristics::KillerMoves,
    pub counter_moves:  crate::heuristics::CounterMoves,

    pub stacks:         Vec<ABStackPly>,

    pub move_history:   Vec<(Zobrist, Move)>,
    pub pvs:            Vec<Move>,
}

impl ABStack {
    pub fn get_or_push(&mut self, ply: Depth) -> &mut ABStackPly {
        unimplemented!()
    }
}

#[derive(Debug,Clone)]
pub struct ABStackPly {
    pub ply:              Depth,
    pub moves_searched:   u8,
    pub killers:          ArrayVec<Move, 2>,
}

impl ABStackPly {
    pub fn new(ply: Depth) -> Self {
        Self {
            ply,
            moves_searched: 0,
            killers:        ArrayVec::default(),
        }
    }
}

impl ABStack {
    pub fn new_with_moves(moves: &Vec<(Zobrist, Move)>) -> Self {
        let mut out = Self::new();
        out.move_history = moves.clone();
        out
    }
    pub fn new() -> Self {
        Self {
            history:        crate::heuristics::ButterflyHistory::default(),
            // killers:        crate::heuristics::KillerMoves::default(),
            counter_moves:  crate::heuristics::CounterMoves::default(),
            stacks:         Vec::with_capacity(64),
            move_history:   Vec::with_capacity(64),
            // pvs:            Vec::with_capacity(64),
            pvs:            vec![Move::NullMove; 64],
        }
    }
}




