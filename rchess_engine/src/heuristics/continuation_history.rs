
use crate::types::*;
use crate::tables::*;
use crate::heuristics::ContinuationHistory;

impl Default for ContinuationHistory {
    fn default() -> Self {
        Self {
            buf:    [[[[0; 64]; 6]; 64]; 6],
        }
    }
}

impl ContinuationHistory {
}

