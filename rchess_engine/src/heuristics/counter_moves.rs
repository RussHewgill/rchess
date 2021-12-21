
use crate::types::*;
use crate::tables::*;
use crate::heuristics::CounterMoves;

impl Default for CounterMoves {
    fn default() -> Self {
        Self {
            // buf:      [[[0; 64]; 64]; 2],
            buf:      [[[None; 64]; 64]; 2],
        }
    }
}


impl CounterMoves {

    // pub fn _get_move(&self, from: Coord, to: Coord, side: Color) -> Option<Score> {
    //     let x = self.buf[side][from][to];
    //     if x == 0 { None } else { Some(x) }
    // }

    pub fn insert_counter_move(&mut self, prev_mv: Move, mv: Move, side: Color) {
        if mv.filter_quiet() || mv.filter_pawndouble() {
            self.buf[side][prev_mv.sq_from()][prev_mv.sq_to()] = Some(mv);
        }
    }

    pub fn get_counter_move(&self, ts: &'static Tables, g: &Game, mv: Move) -> Option<Move> {
        unimplemented!()
    }

}


