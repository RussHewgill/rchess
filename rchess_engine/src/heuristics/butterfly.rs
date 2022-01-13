
use crate::types::*;
use crate::tables::*;
use crate::heuristics::KillerMoves;

use crate::heuristics::ButterflyHistory;

impl Default for ButterflyHistory {
    fn default() -> Self {
        Self {
            buf:      [[[0; 64]; 64]; 2],
            // buf:      [[[(0,0); 64]; 64]; 2],
        }
    }
}

impl ButterflyHistory {

    // pub fn get_move(&self, mv: Move, side: Color) -> Option<Score> {
    pub fn get_move(&self, mv: Move, side: Color) -> Score {
        // assert!(mv.filter_quiet() || mv.filter_pawndouble());
        self._get_move(mv.sq_from(), mv.sq_to(), side)
    }

    // pub fn _get_move(&self, from: Coord, to: Coord, side: Color) -> Option<Score> {
    fn _get_move(&self, from: Coord, to: Coord, side: Color) -> Score {
        // let x = self.buf[side][from][to];
        // if x == 0 { None } else { Some(x) }

        self.buf[side][from][to]

        // let (good,all) = self.good[side][from][to];
        // good / all
    }

    // pub fn increment(&mut self, mv: Move, depth: Depth, side: Color) {
    //     // assert!(mv.filter_quiet() || mv.filter_pawndouble());
    //     if !(mv.filter_quiet() || mv.filter_pawndouble()) {
    //         return;
    //     }
    //     self._increment(mv.sq_from(), mv.sq_to(), side, depth_stat_bonus(depth));
    // }

    pub fn update(&mut self, mv: Move, side: Color, bonus: Score) {
        self._update(mv.sq_from(), mv.sq_to(), side, bonus);
    }

    fn _update(&mut self, from: Coord, to: Coord, side: Color, bonus: Score) {
        self.buf[side][from][to] += bonus;

        // /// XXX: ??? stockfish magic
        // const D: Score = 14_000;
        // assert!(add.abs() <= D);
        // let x = &mut self.buf[side][from][to];
        // *x += add - *x * add.abs() / D;

    }

}

