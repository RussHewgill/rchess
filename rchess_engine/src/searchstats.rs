
use std::time::Duration;

use crate::types::*;
use crate::tables::*;

#[derive(Debug,Default,PartialEq,PartialOrd,Clone,Copy)]
pub struct SearchStats {
    pub nodes:          u32,
    pub leaves:         u32,
    pub quiet_leaves:   u32,
    pub max_depth:      u8,
    pub checkmates:     u32,
    pub stalemates:     u32,
    pub tt_hits:        u32,
    pub tt_misses:      u32,
    pub qt_nodes:       u32,
    pub qt_hits:        u32,
    pub qt_misses:      u32,
    pub alpha:          i32,
    pub beta:           i32,
    pub ns_pv:          u32,
    pub ns_all:         u32,
    pub ns_cut:         u32,
}

impl SearchStats {

    fn _print(x: i32) -> String {
        if x.abs() > 1_000_000 {
            format!("{:.1}M", x as f64 / 1_000_000.)
        } else if x > 1000 {
            format!("{:.1}k", x as f64 / 1000.)
        } else {
            format!("{}", x)
        }
    }

    pub fn print(&self, dt: Duration) {
        println!();
        println!("time         = {:.3}s", dt.as_secs_f64());
        println!("nodes        = {}", Self::_print(self.nodes as i32));
        println!("rate         = {:.1} nodes/s", (self.nodes as f64 / 1000.) / dt.as_secs_f64());
        println!("max depth    = {}", self.max_depth);
        println!("leaves       = {}", Self::_print(self.leaves as i32));
        // println!("quiet_leaves = {}", Self::_print(self.quiet_leaves as i32));
        println!("checkmates   = {}", Self::_print(self.checkmates as i32));
        // println!("stalemates   = {}", Self::_print(self.stalemates as i32));
        println!("hits         = {}", Self::_print(self.tt_hits as i32));
        println!("misses       = {}", Self::_print(self.tt_misses as i32));
        // println!("qt_nodes     = {}", Self::_print(self.qt_nodes as i32));
        // println!("qt_hits      = {}", Self::_print(self.qt_hits as i32));
        // println!("qt_misses    = {}", Self::_print(self.qt_misses as i32));

        // println!("alpha      = {:?}", self.alpha);
        // println!("beta       = {:?}", self.beta);
        // println!("PV Nodes   = {:?}", self.ns_pv);
        // println!("All Nodes  = {:?}", self.ns_all);
        // println!("Cut Nodes  = {:?}", self.ns_cut);
    }

}

impl std::ops::Add for SearchStats {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            nodes:              self.nodes + other.nodes,
            leaves:             self.leaves + other.leaves,
            quiet_leaves:       self.quiet_leaves + other.quiet_leaves,
            max_depth:          u8::max(self.max_depth, other.max_depth),
            checkmates:         self.checkmates + other.checkmates,
            stalemates:         self.stalemates + other.stalemates,
            tt_hits:            self.tt_hits + other.tt_hits,
            tt_misses:          self.tt_misses + other.tt_misses,
            qt_nodes:           self.qt_nodes + other.qt_nodes,
            qt_hits:            self.qt_hits + other.qt_hits,
            qt_misses:          self.qt_misses + other.qt_misses,
            alpha:              i32::max(self.alpha, other.alpha),
            beta:               i32::min(self.beta, other.beta),
            ns_pv:              self.ns_pv + other.ns_pv,
            ns_all:             self.ns_all + other.ns_all,
            ns_cut:             self.ns_cut + other.ns_cut,
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
