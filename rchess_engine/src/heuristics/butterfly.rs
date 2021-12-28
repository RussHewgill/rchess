
use crate::tuning::depth_stat_bonus;
use crate::types::*;
use crate::tables::*;
use crate::heuristics::KillerMoves;

use crate::heuristics::ButterflyHistory;

impl Default for ButterflyHistory {
    fn default() -> Self {
        Self {
            buf:      [[[0; 64]; 64]; 2],
        }
    }
}

impl ButterflyHistory {

    pub fn get_move(&self, mv: Move, side: Color) -> Option<Score> {
        // assert!(mv.filter_quiet() || mv.filter_pawndouble());
        self._get_move(mv.sq_from(), mv.sq_to(), side)
    }

    pub fn _get_move(&self, from: Coord, to: Coord, side: Color) -> Option<Score> {
        let x = self.buf[side][from][to];
        if x == 0 { None } else { Some(x) }
    }

    pub fn increment(&mut self, mv: Move, ply: Depth, side: Color) {
        // assert!(mv.filter_quiet() || mv.filter_pawndouble());
        if !(mv.filter_quiet() || mv.filter_pawndouble()) {
            return;
        }
        self._increment(mv.sq_from(), mv.sq_to(), side, depth_stat_bonus(ply));
    }

    fn _increment(&mut self, from: Coord, to: Coord, side: Color, add: Score) {
        self.buf[side][from][to] += add;

        // /// XXX: ??? stockfish magic
        // const D: Score = 14_000;
        // assert!(add.abs() <= D);
        // let x = &mut self.buf[side][from][to];
        // *x += add - *x * add.abs() / D;

    }

}

