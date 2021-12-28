
use crate::types::*;

use super::CaptureHistory;

impl Default for CaptureHistory {
    fn default() -> Self {
        Self { buf: [[[0; 5]; 64]; 6] }
    }
}

impl CaptureHistory {

    /// [Piece][To][CapturedPiece]
    pub fn increment(&mut self, mv: Move, bonus: Score) {
        if let (Some(pc),Some(victim)) = (mv.piece(),mv.victim()) {
            self._increment(pc, mv.sq_to(), victim, bonus);
        }
    }

    pub fn _increment(&mut self, pc: Piece, to: Coord, victim: Piece, bonus: Score) {
        assert!(pc != King);
        self.buf[pc][to][victim] += bonus;
    }

    pub fn get(&self, pc: Piece, to: Coord, victim: Piece) -> Option<Score> {
        assert!(pc != King);
        let x = self.buf[pc][to][victim];
        if x == 0 { None } else { Some(x) }
    }

}

