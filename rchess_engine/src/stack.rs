
use std::collections::HashSet;

use crate::{types::*, tables::HISTORY_MAX};

use arrayvec::ArrayVec;

#[derive(Debug,Clone)]
pub struct ABStack {
    // pub history:            [[[Score; 64]; 64]; 2],
    pub history:            crate::heuristics::ButterflyHistory,
    // pub killers:            crate::heuristics::KillerMoves,
    pub counter_moves:      crate::heuristics::CounterMoves,

    pub capture_history:    crate::heuristics::CaptureHistory,

    pub inside_null:        bool,

    // pub stacks:             ArrayVec<ABStackPly,128>,
    pub stacks:             Vec<ABStackPly>,

    // pub move_history:       ArrayVec<(Zobrist, Move), 128>,
    pub move_history:       Vec<(Zobrist, Move)>,
    // pub move_history:       HashSet<Zobrist>,

    pub pvs:                [Move; 128],
}

/// Get, push, with
impl ABStack {

    // pub fn get_or_push(&mut self, ply: Depth) -> &mut ABStackPly {
    //     unimplemented!()
    // }

    pub fn get(&self, ply: Depth) -> Option<&ABStackPly> {
        self.stacks.get(ply as usize)
    }

    pub fn push_if_empty(&mut self, g: &Game, ply: Depth) {
        if self.stacks.get(ply as usize).is_none() {
            self.stacks.push(ABStackPly::new(g, ply));
        }
    }

    pub fn with<F>(&mut self, ply: Depth, mut f: F)
        where F: FnMut(&mut ABStackPly)
    {
        if let Some(st) = self.stacks.get_mut(ply as usize) {
            f(st);
        }
    }

    pub fn get_with<F,T>(&self, ply: Depth, mut f: F) -> T
        where F: FnMut(&ABStackPly) -> T
    {
        if let Some(st) = self.stacks.get(ply as usize) {
            unimplemented!()
        } else {
            panic!("ABStack get_with bad ply");
        }
    }

}

/// Update stats
impl ABStack {

    // pub fn update_stats_fail_high(
    //     &mut self,
    //     g:          &Game,
    //     beta:       Score,
    //     ply:        Depth,
    //     depth:      Depth,
    // ) {
    //     unimplemented!()
    // }

    pub fn update_stats(
        &mut self,
        g:          &Game,
        best_move:  Move,
        best_score: Score,
        beta:       Score,
        ply:        Depth,
        depth:      Depth,
    ) {

        let bonus = Self::stat_bonus(depth);

        if best_move.filter_capture_or_promotion() {
            self.capture_history.increment(best_move, bonus);
        }

        unimplemented!()
    }

    pub fn update_quiet_stats(
        &mut self,
        g:          &Game,
        mv:         Move,
        depth:      Depth,
    ) {
        unimplemented!()
    }

    fn stat_bonus(depth: Depth) -> Score {
        let depth = depth as Score;
        Score::min(HISTORY_MAX, depth * depth)
    }

    // /// Stockfish magic
    // fn stat_bonus(depth: Depth) -> Score {
    //     let d = depth as Score;
    //     Score::min((6 * d + 200) * d - 215, 2000)
    // }

    // /// from Ethereal
    // fn history_bonus_decay(current: Score, delta: Score) -> Score {
    //     let mult = 32;
    //     let div = 512;
    //     current + mult * delta - current * delta.abs() / div
    // }

}

/// Killers
impl ABStack {
    pub fn killer_get(&self, ply: Depth) -> (Option<Move>,Option<Move>) {
        if let Some(st) = self.stacks.get(ply as usize) {
            st.killer_get()
        } else {
            (None,None)
        }
    }

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
    pub static_eval:      Option<Score>,
    pub material:         Material,
    pub in_check:         bool,
    pub forbidden_move:   Option<Move>,
}

/// New
impl ABStackPly {
    pub fn new(g: &Game, ply: Depth) -> Self {
        Self {
            ply,
            moves_searched: 0,
            // killers:        ArrayVec::default(),
            killers:        [None; 2],
            static_eval:    None,
            material:       g.state.material,
            in_check:       false,
            forbidden_move: None,
        }
    }
}

/// Killers
impl ABStackPly {

    pub fn killer_get(&self) -> (Option<Move>,Option<Move>) {
        (self.killers[0],self.killers[1])
    }

    pub fn killer_store(&mut self, mv: Move) {
        if self.killers[0] != Some(mv) {
            self.killers[1] = self.killers[0];
            self.killers[0] = Some(mv);
        }
    }
}

/// New
impl ABStack {
    pub fn new_with_moves(moves: &Vec<(Zobrist, Move)>) -> Self {
        let mut out = Self::new();
        out.move_history = moves.clone();
        // for (zb,_) in moves.iter() {
        //     out.move_history.insert(*zb);
        // }
        // out.move_history.try_extend_from_slice(&moves).unwrap();
        out
    }
    pub fn new() -> Self {
        Self {
            history:            crate::heuristics::ButterflyHistory::default(),
            counter_moves:      crate::heuristics::CounterMoves::default(),
            capture_history:    crate::heuristics::CaptureHistory::default(),

            inside_null:        false,

            stacks:             Vec::with_capacity(64),
            move_history:       Vec::with_capacity(64),

            pvs:                [Move::NullMove; 128],
        }
    }
}




