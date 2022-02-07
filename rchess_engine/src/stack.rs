
use std::collections::HashSet;

use crate::{types::*, tables::{HISTORY_MAX, MAX_SEARCH_PLY}, searchstats::RunningAverage};

use arrayvec::ArrayVec;
use nom::InputIter;

#[derive(Debug,Clone)]
pub struct ABStack {
    pub history:                crate::heuristics::ButterflyHistory,
    pub counter_moves:          crate::heuristics::CounterMoves,
    pub capture_history:        crate::heuristics::CaptureHistory,
    // pub continuation_history:   crate::heuristics::ContinuationHistory,

    pub double_ext_avg:         [RunningAverage; 2],
    pub exploding:              bool,

    // pub pieces:                 [Option<Piece>; 64],

    pub inside_null:            bool,

    pub stacks:                 Vec<ABStackPly>,

    pub move_history:           Vec<(Zobrist, Move)>,

    pub pvs:                    [Move; 128],
}

/// Get, push, with
impl ABStack {

    // pub fn get_or_push(&mut self, ply: Depth) -> &mut ABStackPly {
    //     unimplemented!()
    // }

    pub fn get(&self, ply: Depth) -> Option<&ABStackPly> {
        self.stacks.get(ply as usize)
    }

    // pub fn push_if_empty(&mut self, g: &Game, ply: Depth) {
    //     if self.stacks.get(ply as usize).is_none() {
    //         self.stacks.push(ABStackPly::new(g, ply));
    //     }
    // }

    pub fn with<F>(&mut self, ply: Depth, mut f: F)
        where F: FnMut(&mut ABStackPly)
    {
        if let Some(st) = self.stacks.get_mut(ply as usize) {
            f(st);
        } else {
            panic!("stack with, but missing ply: {}", ply);
        }
    }

    pub fn get_with<F,T>(&self, ply: Depth, mut f: F) -> Option<T>
        where F: FnMut(&ABStackPly) -> T
    {
        if let Some(st) = self.stacks.get(ply as usize) {
            Some(f(st))
        } else {
            // panic!("ABStack get_with bad ply");
            None
        }
    }

}

/// init_node
impl ABStack {
    pub fn init_node(&mut self, ply: Depth, depth: Depth, g: &Game) {

        let in_check = g.state.in_check;

        /// use previous double extensions
        let d = self.get_with(ply - 1, |st| st.double_extensions).unwrap_or(0);

        self.with(ply, |st| {
            st.material          = g.state.material;
            st.in_check          = in_check;
            st.moves_searched    = 0;
            st.current_move      = None;
            st.double_extensions = d;
            st.depth             = depth;
        });

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

    #[cfg(not(feature = "history_heuristic"))]
    pub fn update_history(
        &mut self,
        g:                    &Game,
        best_mv:              Move,
        best_score:           Score,
        beta:                 Score,
        captures_searched:    ArrayVec<Move,64>,
        quiets_searched:      ArrayVec<Move,64>,
        ply:                  Depth,
        depth:                Depth,
    ) {
    }

    #[cfg(feature = "history_heuristic")]
    pub fn update_history(
        &mut self,
        g:                    &Game,
        best_mv:              Move,
        best_score:           Score,
        beta:                 Score,
        captures_searched:    ArrayVec<Move,64>,
        quiets_searched:      ArrayVec<Move,64>,
        ply:                  Depth,
        depth:                Depth,
    ) {

        let side        = g.state.side_to_move;
        let bonus       = Self::stat_bonus(depth);
        let bonus_quiet = Self::stat_bonus(depth - 1);

        if best_mv.filter_capture_or_promotion() {
            /// Increase Capture history for good capture
            self.capture_history.update(best_mv, bonus);
        } else {
            /// Increase history for good quiet move
            // self.history.update(best_mv, side, bonus_quiet);
            self.update_quiet_stats(g, ply, best_mv, side, bonus_quiet);

            /// penalty for all quiets that aren't the best move
            for mv in quiets_searched.into_iter() {
                if mv != best_mv {
                    self.history.update(mv, side, -bonus_quiet);
                    // self.update_continuation_history(mv, ply, -bonus_quiet);
                }
            }

        }

        /// decrease stats for all captures other than the best one
        for mv in captures_searched.into_iter() {
            if mv != best_mv {
                self.capture_history.update(mv, -bonus);
            }
        }

    }

    pub fn update_quiet_stats(
        &mut self,
        g:          &Game,
        ply:        Depth,
        mv:         Move,
        side:       Color,
        bonus:      Score,
    ) {

        #[cfg(feature = "killer_moves")]
        if !mv.filter_all_captures() {
            self.killers_store(ply, mv);
        }

        #[cfg(feature = "countermove_heuristic")]
        if let Some(prev_mv) = g.last_move {
            self.counter_moves.insert_counter_move(prev_mv, mv, g.state.side_to_move);
            // stack.counter_moves.insert_counter_move(prev_mv, mv);
        }

        self.history.update(mv, side, bonus);
        // self.update_continuation_history(mv, ply, bonus);

        // unimplemented!()
    }

    // #[cfg(feature = "nope")]
    pub fn stat_bonus(depth: Depth) -> Score {
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

/// Get History
impl ABStack {
    pub fn get_move_history(
        &self,
        mv:           Move,
        side:         Color,
        prev_mv:      Option<Move>,
    ) -> Score {
        if mv.filter_all_captures() {
            // self.capture_history.get(mv)
            // -self.capture_history.get(mv)
            0
        } else {
            self.history.get_move(mv, side)
        }

        // let cm: [[[[[Score; 2]; 6]; 64]; 6]; 64];
    }
}

/// Killers
impl ABStack {

    pub fn killers_clear(&mut self, ply: Depth) {
        self.with(ply, |st| st.killers = [None; 2]);
    }

    pub fn killers_get(&self, ply: Depth) -> (Option<Move>,Option<Move>) {
        if let Some(st) = self.stacks.get(ply as usize) {
            st.killer_get()
        } else {
            // debug!("stack killers_get, but missing ply: {}", ply);
            (None,None)
        }
    }

    pub fn killers_store(&mut self, ply: Depth, mv: Move) {

        self.with(ply, |st| st.killer_store(mv));

        // if let Some(st) = self.stacks.get_mut(ply as usize) {
        //     st.killer_store(mv);
        // }
    }
}

/// misc
impl ABStack {

    pub fn update_double_extension_avg(&mut self, ply: Depth, side: Color) {

        if ply == 0 { return; }

        let d0 = self.stacks[ply as usize].depth;
        let d1 = self.stacks[ply as usize - 1].depth;

        let x = if d0 > d1 {
            // panic!()
            1
        } else {
            0
        };

        self.double_ext_avg[side].update(x);

    }

    pub fn update_double_extension(&mut self, ply: Depth, extensions: Depth) {
        if extensions == 2 {
            let d = self.stacks[ply as usize - 1].double_extensions + 1;
            self.stacks[ply as usize].double_extensions = d;
        }
    }

}

/// Clear
impl ABStack {

    pub fn clear_history(&mut self) {
        self.history         = Default::default();
        self.counter_moves   = Default::default();
        self.capture_history = Default::default();
    }

}

#[derive(Debug,Clone,Default)]
pub struct ABStackPly {
    // pub zobrist:          Zobrist,
    pub ply:                  Depth,
    pub depth:                Depth,
    pub moves_searched:       u8,
    pub killers:              [Option<Move>; 2],
    pub static_eval:          Option<Score>,
    pub material:             Material,
    pub in_check:             bool,
    pub current_move:         Option<Move>,
    pub forbidden_move:       Option<Move>,
    pub double_extensions:    u32,
}

/// New
impl ABStackPly {

    pub fn new_empty(ply: Depth) -> Self {
        let mut out = Self::default();
        out.ply = ply;
        out
    }

    pub fn new(g: &Game, ply: Depth) -> Self {
        Self {
            // zobrist:            g.zobrist,
            ply,
            depth: 0,
            moves_searched:     0,
            // killers:            ArrayVec::default(),
            killers:            [None; 2],
            static_eval:        None,
            material:           g.state.material,
            in_check:           false,
            current_move:       None,
            forbidden_move:     None,
            double_extensions:  0,
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
        let mut out = Self::new_with_plies();
        out.move_history = moves.clone();
        // for (zb,_) in moves.iter() {
        //     out.move_history.insert(*zb);
        // }
        // out.move_history.try_extend_from_slice(&moves).unwrap();
        out
    }

    pub fn new() -> Self {
        Self {
            history:                crate::heuristics::ButterflyHistory::default(),
            counter_moves:          crate::heuristics::CounterMoves::default(),
            capture_history:        crate::heuristics::CaptureHistory::default(),
            // continuation_history:   crate::heuristics::ContinuationHistory::default(),

            double_ext_avg:         [RunningAverage::new(0,100); 2],
            exploding:              false,

            // pieces:                 [None; 64],

            inside_null:            false,

            // stacks,
            // move_history:           Vec::with_capacity(64),
            stacks:                 vec![],
            move_history:           vec![],

            pvs:                    [Move::NullMove; 128],
        }
    }

    pub fn new_with_plies() -> Self {

        let mut out = Self::new();

        // let mut stacks = vec![];
        let mut stacks = Vec::with_capacity(MAX_SEARCH_PLY as usize);
        for ply in 0..MAX_SEARCH_PLY {
            stacks.push(ABStackPly::new_empty(ply));
        }
        out.stacks = stacks;

        out
    }
}




