
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
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
    pub fn get_move(&self, from: Coord, to: Coord, side: Color) -> i16 {
        self.buf[side][from][to]
    }

    pub fn increment(&mut self, mv: Move, ply: Depth, side: Color) {
    }

    pub fn _increment(&mut self, from: Coord, to: Coord, side: Color, add: i16) {
        self.buf[side][from][to] += add;
    }

}

