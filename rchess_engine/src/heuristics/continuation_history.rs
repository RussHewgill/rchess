
use crate::types::*;
use crate::tables::*;
use crate::heuristics::ContinuationHistory;
use crate::stack::*;

impl Default for ContinuationHistory {
    fn default() -> Self {
        Self {
            buf:            [[[[0; 64]; 6]; 64]; 6],
            buf_in_check:   [[[[0; 64]; 6]; 64]; 6],
        }
    }
}

impl ABStack {

    pub fn update_continuation_history(
        &mut self,
        mv:           Move,
        ply:          Depth,
        bonus:        Score,
    ) {
        let in_check = self.stacks.get(ply as usize).map(|st| st.in_check).unwrap();

        let pc = mv.piece().unwrap();
        let to = mv.sq_to();

        for i in [1,2,4,6] {

            if in_check && i > 2 { break; }

            if let Some(st) = self.stacks.get(ply as usize - i) {
                if let Some(current_mv) = st.current_move {
                }
            }
        }
        unimplemented!()
    }

}

impl ContinuationHistory {
    fn update(&mut self, mv: Move, prev_mv: Move) {
        unimplemented!()
    }
}

