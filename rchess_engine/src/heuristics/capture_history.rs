
use crate::types::*;

use super::CaptureHistory;

impl Default for CaptureHistory {
    fn default() -> Self {
        Self { buf: [[[0; 5]; 64]; 6] }
    }
}

impl CaptureHistory {

    /// [Piece][To][CapturedPiece]
    pub fn increment(&mut self, pc: Piece, to: Coord, victim: Piece) {
        self.buf[pc][to][victim] += 1;
    }

    pub fn get(&self, pc: Piece, to: Coord, victim: Piece) -> Option<i16> {
        let x = self.buf[pc][to][victim];
        if x == 0 { None } else { Some(x) }
    }

}

