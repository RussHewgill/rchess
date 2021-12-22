
use crate::types::*;

use arrayvec::ArrayVec;

#[derive(Debug,Clone)]
pub struct ABStack {
    // pub history:        [[[Score; 64]; 64]; 2],
    pub history:        crate::heuristics::ButterflyHistory,
    // pub killers:        crate::heuristics::KillerMoves,
    pub counter_moves:  crate::heuristics::CounterMoves,

    pub stacks:         ArrayVec<ABStackPly,128>,

    pub move_history:   ArrayVec<(Zobrist, Move), 128>,
    pub pvs:            [Move; 128],
}

impl ABStack {
    pub fn get_or_push(&mut self, ply: Depth) -> &mut ABStackPly {
        unimplemented!()
    }

    pub fn push_if_empty(&mut self, ply: Depth) {
        if self.stacks.get(ply as usize).is_none() {
            self.stacks.push(ABStackPly::new(ply));
        }
    }
}

impl ABStack {
    pub fn killer_store(&mut self, ply: Depth, mv: Move) {
        if let Some(st) = self.stacks.get_mut(ply as usize) {
            st.killer_store(mv);
        }
    }
}

#[derive(Debug,Clone)]
pub struct ABStackPly {
    pub ply:              Depth,
    pub moves_searched:   u8,
    pub killers:          [Option<Move>; 2],
}

/// Killers
impl ABStackPly {
    pub fn killer_store(&mut self, mv: Move) {
        if self.killers[0] != Some(mv) {
            self.killers[1] = self.killers[0];
            self.killers[0] = Some(mv);
        }
    }
}

/// New
impl ABStackPly {
    pub fn new(ply: Depth) -> Self {
        Self {
            ply,
            moves_searched: 0,
            // killers:        ArrayVec::default(),
            killers:        [None; 2],
        }
    }
}

/// New
impl ABStack {
    pub fn new_with_moves(moves: &Vec<(Zobrist, Move)>) -> Self {
        let mut out = Self::new();
        // out.move_history = moves.clone();
        out.move_history.try_extend_from_slice(&moves).unwrap();
        out
    }
    pub fn new() -> Self {
        Self {
            history:        crate::heuristics::ButterflyHistory::default(),
            // killers:        crate::heuristics::KillerMoves::default(),
            counter_moves:  crate::heuristics::CounterMoves::default(),
            // stacks:         Vec::with_capacity(64),
            // move_history:   Vec::with_capacity(64),
            stacks:         ArrayVec::new(),
            move_history:   ArrayVec::new(),
            // pvs:            Vec::with_capacity(64),
            // pvs:            vec![Move::NullMove; 64],
            pvs:            [Move::NullMove; 128],
        }
    }
}




