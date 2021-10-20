
use std::time::Duration;

use crate::types::*;
use crate::tables::*;

#[derive(Debug,Default,PartialEq,PartialOrd,Clone,Copy)]
pub struct SearchStats {
    pub nodes:          u32,
    pub leaves:         u32,
    pub checkmates:     u32,
    pub stalemates:     u32,
    pub tt_hits:        u32,
    pub tt_misses:      u32,
    pub alpha:          i32,
    pub beta:           i32,
}

impl SearchStats {

    pub fn print(&self, dt: Duration) {
        print!("\n");
        println!("time       = {:.3}s", dt.as_secs_f64());
        println!("nodes      = {:?}", self.nodes);
        println!("rate       = {:.2} nodes/s", (self.nodes as f64 / 1000.) / dt.as_secs_f64());
        println!("leaves     = {:?}", self.leaves);
        println!("checkmates = {:?}", self.checkmates);
        println!("stalemates = {:?}", self.stalemates);
        println!("hits       = {:?}", self.tt_hits);
        println!("misses     = {:?}", self.tt_misses);
        println!("alpha      = {:?}", self.alpha);
        println!("beta       = {:?}", self.beta);
    }

}

impl std::ops::Add for SearchStats {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            nodes:          self.nodes + other.nodes,
            leaves:         self.leaves + other.leaves,
            checkmates:     self.checkmates + other.checkmates,
            stalemates:     self.stalemates + other.stalemates,
            tt_hits:        self.tt_hits + other.tt_hits,
            tt_misses:      self.tt_misses + other.tt_misses,
            alpha:          i32::max(self.alpha, other.alpha),
            beta:           i32::min(self.beta, other.beta),
        }
    }
}

impl std::ops::AddAssign for SearchStats {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl std::iter::Sum<Self> for SearchStats {
    fn sum<I>(iter: I) -> Self where
        I: Iterator<Item = Self> {
        iter.fold(Self::default(), |a,b| a + b)
    }
}

impl<'a> std::iter::Sum<&'a Self> for SearchStats {
    fn sum<I>(iter: I) -> Self where
        I: Iterator<Item = &'a Self> {
        iter.fold(Self::default(), |a,b| a + *b)
    }
}

