
use crate::tables::*;
use crate::types::*;
use crate::evaluate::*;
use crate::brain::types::*;
use crate::brain::types::nnue::*;
use crate::brain::accumulator::*;
use crate::brain::matrix::*;

use ndarray as nd;
use nd::{Array2};

/// Increment
impl NNUE {
    pub fn update_move(&mut self, g: &Game) -> i32 {
        unimplemented!()
    }
}

/// Run
impl NNUE {
    pub fn run_fresh(&self, g: &Game) -> i32 {
        unimplemented!()
    }
}

/// Backprop
impl NNUE {
}

/// Init
impl NNUE {

    /// Reset inputs and activations, and refill from board
    pub fn init_inputs(&mut self, g: &Game) {
        self.accum.init_inputs(g);
    }

}

