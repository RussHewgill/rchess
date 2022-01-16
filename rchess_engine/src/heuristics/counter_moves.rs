
use crate::types::*;
use crate::tables::*;
use crate::heuristics::CounterMoves;

impl Default for CounterMoves {
    fn default() -> Self {
        Self {
            // buf:      [[[0; 64]; 64]; 2],
            // buf:      [[[None; 64]; 64]; 2],
            buf:      [[[None; 64]; 6]; 2],
            // buf:      [[None; 64]; 6],
        }
    }
}


impl CounterMoves {

    // pub fn clear(&mut self) {
    //     for x in self.buf.iter_mut()
    //         .flat_map(|x| x.iter_mut())
    //         .flat_map(|x| x.iter_mut()) {
    //             *x = None;
    //         }
    // }

    pub fn insert_counter_move(&mut self, prev_mv: Move, mv: Move, side: Color) {
    // pub fn insert_counter_move(&mut self, prev_mv: Move, mv: Move) {
        assert!(mv != Move::NullMove);
        if let Some(pc) = prev_mv.piece() {
            self.buf[side][pc][prev_mv.sq_to()] = Some(mv);
            // self.buf[pc][prev_mv.sq_to()] = Some(mv);
        }
    }

    pub fn get_counter_move(&self, prev_mv: Move, side: Color) -> Option<Move> {
    // pub fn get_counter_move(&self, prev_mv: Move) -> Option<Move> {
        let pc = prev_mv.piece()?;
        self.buf[side][pc][prev_mv.sq_to()]
        // self.buf[pc][prev_mv.sq_to()]
    }

}


