
use crate::types::*;
use crate::tables::*;
use crate::heuristics::ContinuationHistory;
use crate::stack::*;

impl Default for ContinuationHistory {
    fn default() -> Self {
        Self {
            buf:    [[[[0; 64]; 6]; 64]; 6],
        }
    }
}

impl ABStack {

    pub fn update_continuation_history(
        &mut self,
        mv:           Move,
        bonus:        Score,
    ) {

        // for i in [1,2,4,6]

        unimplemented!()
    }

}

