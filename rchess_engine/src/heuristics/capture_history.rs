
use crate::types::*;

use super::CaptureHistory;

impl Default for CaptureHistory {
    fn default() -> Self {
        Self { buf: [[[0; 5]; 64]; 6] }
    }
}

impl CaptureHistory {

    /// [Piece][To][CapturedPiece]
    pub fn update(&mut self, mv: Move, bonus: Score) {
        if let (Some(pc),Some(victim)) = (mv.piece(),mv.victim()) {
            self._update(pc, mv.sq_to(), victim, bonus);
        }
    }

    pub fn _update(&mut self, pc: Piece, to: Coord, victim: Piece, bonus: Score) {
        // assert!(pc != King);
        self.buf[pc][to][victim] += bonus;
    }

    // pub fn get(&self, mv: Move) -> Option<Score> {
    pub fn get(&self, mv: Move) -> Score {
        if let (Some(pc),Some(victim)) = (mv.piece(),mv.victim()) {
            self._get(pc, mv.sq_to(), victim)
        } else {
            // unimplemented!()
            panic!("CaptureHistory: get {:?}", mv);
        }
        // let pc = mv.piece()?;
        // let victim = mv.victim()?;
    }

    // pub fn _get(&self, pc: Piece, to: Coord, victim: Piece) -> Option<Score> {
    pub fn _get(&self, pc: Piece, to: Coord, victim: Piece) -> Score {
        // assert!(pc != King);
        // let x = self.buf[pc][to][victim];
        // if x == 0 { None } else { Some(x) }
        self.buf[pc][to][victim]
    }

}

