
use std::time::Duration;

pub use crate::{stats,not_stats,stats_or};
pub use crate::util::pretty_print_si;

pub use self::ss::*;

// :norm LxHysL)istats!A;

#[cfg(not(feature = "keep_stats"))]
mod ss {
    use crate::types::*;
    use crate::tables::*;
    use crate::explore::*;

    use std::time::Duration;

    #[derive(Debug,Default,PartialEq,PartialOrd,Clone,Copy)]
    pub struct SearchStats {}

    impl SearchStats {
        pub fn inc_nodes_arr(&mut self, ply: i16) {}

        pub fn print_node_types(&self, tt_r: &TTRead) {}
        pub fn print_ebf(&self, full: bool) {}
        pub fn print(&self, dt: Duration) {}

    }

    impl std::ops::Add for SearchStats {
        type Output = Self;
        fn add(self, other: Self) -> Self {
            Self {}
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

}

#[cfg(feature = "keep_stats")]
mod ss {
    use crate::types::*;
    use crate::tables::*;
    use crate::explore::*;

    use std::time::Duration;

    use derive_more::*;

    #[derive(Debug,Default,PartialEq,PartialOrd,Clone,Copy,Add,AddAssign)]
    pub struct SearchStats {
        pub nodes:              u64,
        pub nodes_arr:          NArr,
        // pub nodes_zb:           NHashes,
        pub leaves:             u32,
        pub quiet_leaves:       u32,
        pub max_depth:          Max,
        pub max_depth_search:   Max,
        pub q_max_depth:        Max,

        pub checkmates:         u32,
        pub stalemates:         u32,

        pub tt_hits:            u32,
        pub tt_halfmiss:        u32,
        pub tt_misses:          u32,
        pub tt_eval:            u32,

        pub ph_hits:            u32,
        pub ph_misses:          u32,

        pub qt_nodes:           u32,
        pub qt_hits:            u32,
        pub qt_misses:          u32,
        pub qs_tt_returns:      u32,
        pub qs_delta_prunes:    u32,

        pub ns_pv:              u32,
        pub ns_all:             u32,
        pub ns_cut:             u32,

        pub nth_best_pv_mv:     NArr,

        pub null_prunes:        u32,
        pub fut_prunes:         u32,
        pub lmrs:               u32,
        pub sing_exts:          SSSingularExtensions,

        pub counter_moves:      u32,
        // pub window_fails:       (u32,u32),
        pub beta_cut_first:     u32,
        pub beta_cut_not_first: u32,
        // #[cfg(feature = "pvs_search")]
        // pub pvs_
    }

    // pub struct NHashes

    #[derive(Debug,Default,Eq,PartialEq,Ord,PartialOrd,Clone,Copy,Add,AddAssign)]
    pub struct SSSingularExtensions {
        pub one:     u32,
        pub two:     u32,
        pub prunes:  u32,
        pub reduce:  u32,
        pub capture: u32,
        pub check:   u32,
    }

    #[derive(Debug,Default,Eq,PartialEq,Ord,PartialOrd,Clone,Copy,AddAssign)]
    pub struct Max(pub u32);

    impl Max {
        pub fn max_mut(&mut self, other: u32) {
            self.0 = self.0.max(other)
        }
        pub fn max(self, other: u32) -> Self {
            Self(self.0.max(other))
        }
    }

    impl std::ops::Add for Max {
        type Output = Self;
        fn add(self, other: Self) -> Self {
            Self(self.0.max(other.0))
        }
    }

    #[derive(Debug,Eq,PartialEq,Ord,PartialOrd,Clone,Copy)]
    pub struct NArr(pub [u32; 64]);

    impl Default for NArr {
        fn default() -> Self {
            Self([0; 64])
        }
    }

    impl std::ops::Add for NArr {
        type Output = Self;
        fn add(self, other: Self) -> Self {
            let mut arr = self;
            other.0.iter().enumerate().for_each(|(i,x)| {
                arr.0[i] += x;
            });
            arr
        }
    }

    impl std::ops::AddAssign for NArr {
        fn add_assign(&mut self, other: Self) {
            *self = *self + other;
        }
    }

    /// Print
    impl SearchStats {

        pub fn print(&self, dt: Duration) {
            println!();
            println!("time         = {:.3}s", dt.as_secs_f64());
            println!("nodes        = {}", Self::_print(self.nodes as i32));
            println!("rate         = {:.1} K nodes/s", (self.nodes as f64 / 1000.) / dt.as_secs_f64());
            // println!("max depth    = {}", self.max_depth);
            println!("max depth    = {}", self.max_depth_search.0);
            // println!("leaves       = {}", Self::_print(self.leaves as i32));
            // println!("quiet_leaves = {}", Self::_print(self.quiet_leaves as i32));
            // println!("checkmates   = {}", Self::_print(self.checkmates as i32));
            // println!("stalemates   = {}", Self::_print(self.stalemates as i32));

            let tot = self.tt_hits + self.tt_halfmiss + self.tt_misses;
            let tot = tot as f64;

            println!("hits/half/miss  = {}, {}, {}, ({:.4}%,{:.3}%,{:.3}%)",
                     Self::_print(self.tt_hits as i32),
                     Self::_print(self.tt_halfmiss as i32),
                     Self::_print(self.tt_misses as i32),
                     self.tt_hits as f64 / tot,
                     self.tt_halfmiss as f64 / tot,
                     self.tt_misses as f64 / tot,
            );
            // println!("tt_eval      = {}", Self::_print(self.tt_eval as i32));
            // println!("qt_nodes     = {}", Self::_print(self.qt_nodes as i32));
            // println!("qt_hits      = {}", Self::_print(self.qt_hits as i32));
            // println!("qt_misses    = {}", Self::_print(self.qt_misses as i32));

            // eprintln!("nodes/qt nodes = {:.1?}", self.qt_nodes as f64 / self.nodes as f64);
            eprintln!("qt nodes    = {}", pretty_print_si(self.qt_nodes as i64));
            eprintln!("q_max_depth = {:?}", self.q_max_depth.0);
            // eprintln!("q_tt_returns = {}", pretty_print_si(self.qs_tt_returns as i64));
            // eprintln!("q_delta_prunes = {}", pretty_print_si(self.qs_delta_prunes as i64));

            // eprintln!("null prunes   = {:?}", self.null_prunes);
            // eprintln!("fut prunes    = {:?}", self.fut_prunes);
            // eprintln!("counter_moves = {:?}", self.counter_moves);
            // eprintln!("lmrs          = {:?}", self.lmrs);

            let bcs = (self.beta_cut_first,self.beta_cut_not_first);
            eprintln!("beta_cut_first = {:.3?}", bcs.0 as f64 / (bcs.0 + bcs.1) as f64);

            eprintln!("sing_exts: ({:>5},{:>5},{:>5},{:>5},{:>5},{:>5})",
                      self.sing_exts.one,
                      self.sing_exts.two,
                      self.sing_exts.prunes,
                      self.sing_exts.reduce,
                      self.sing_exts.capture,
                      self.sing_exts.check,
            );

            // eprintln!("pawn hash hitrate = {:.3}",
            //           self.ph_hits as f64 / (self.ph_hits as f64 + self.ph_misses as f64));

            // eprintln!("stats0.qt_hits = {}", pretty_print_si(stats0.qt_hits as i64));
            // eprintln!("stats0.qt_misses = {}", pretty_print_si(stats0.qt_misses as i64));

            // println!("PV Nodes   = {}", pretty_print_si(self.ns_pv as i64));
            // println!("All Nodes  = {}", pretty_print_si(self.ns_all as i64));
            // println!("Cut Nodes  = {}", pretty_print_si(self.ns_cut as i64));

        }

        pub fn print_prunes(&self) {

            println!("null prunes = {:?}, {:.3}",
                     self.null_prunes, self.null_prunes as f64 / self.nodes as f64);
            println!("fut prunes  = {:?}, {:.3}",
                     self.fut_prunes, self.null_prunes as f64 / self.nodes as f64);
            println!("lmrs        = {:?}, {:.3}",
                     self.lmrs, self.lmrs as f64 / self.nodes as f64);

            // println!("fut prunes    = {:?}", self.fut_prunes);
            // println!("counter_moves = {:?}", self.counter_moves);
            // println!("lmrs          = {:?}", self.lmrs);

        }

        pub fn print_nth_best(&self, all: bool) {

            let mut arr = self.nth_best_pv_mv.0.iter().enumerate().rev();

            let idx = arr.clone().position(|x| *x.1 != 0);
            let mut arr = arr.skip(idx.unwrap_or(0)).rev();

            let tot = self.nth_best_pv_mv.0.iter().sum::<u32>();

            // eprintln!("total = {:?}", tot);

            if all {
                for (n,x) in arr {
                    eprintln!("    mv {:>3} = {:>4}, {:>2.1}%", n, x, (*x as f64 / tot as f64) * 100.0);
                }
            } else {
                let x = arr.next().unwrap().1;
                eprintln!("1st move picked = {:>4} / {:>4}, {:>2.1}%",
                          x, tot, (*x as f64 / tot as f64) * 100.0);
            }


        }

        pub fn print_mbf(&self) {
            let mbf = (self.nodes as f64 + self.leaves as f64) / self.leaves as f64;
            eprintln!("MBF = {:.3}", mbf);
        }

        #[cfg(feature = "nope")]
        pub fn print_ebf(&self, full: bool) {
            let arr = self.nodes_arr.0;
            for (ply, n) in arr.iter().enumerate() {
            }
        }

        // #[cfg(feature = "nope")]
        pub fn print_ebf(&self, full: bool) {

            let mut arr = self.nodes_arr.0.iter()
                .enumerate()
                .filter(|x| *x.1 != 0)
                .map(|(i,n)| if i != 0 {
                    (i,(*n,self.nodes_arr.0[i-1]))
                } else { (i,(*n,1)) })
                .collect::<Vec<(usize,(u32,u32))>>();

            arr.sort_by(|a,b| a.0.cmp(&b.0));
            // arr.reverse();

            let mut xs = vec![];
            for (i,(n0,n1)) in arr.iter() {
                // let n = arr2[depth];
                // let ebf = n as f64 / arr2[depth - 1] as f64;
                let ebf = *n0 as f64 / *n1 as f64;
                xs.push(ebf);
                if full {
                    debug!("EBF depth {:>2} = {:>8} nodes, {:.2?}", i, n0, ebf);
                }
            }
            let s: f64 = xs.iter().sum();
            debug!("Average EBF: {:.2}", s / xs.len() as f64);

        }

    }

    /// Misc
    impl SearchStats {

        fn _add_2<T: std::ops::Add<Output = T>>(a: (T,T), b: (T,T)) -> (T,T) {
            (a.0 + b.0, a.1 + b.1)
        }

        pub fn inc_nodes_arr(&mut self, ply: Depth) {
            if ply as usize >= self.nodes_arr.0.len() {
                // debug!("inc_nodes_arr: depth more than 64");
                return;
            }
            // self.nodes_arr.0[d as usize] += 1;
            self.nodes_arr.0[ply as usize] += 1;
        }

        fn _print(x: i32) -> String {
            if x.abs() > 1_000_000 {
                format!("{:.1}M", x as f64 / 1_000_000.)
            } else if x > 1000 {
                format!("{:.1}k", x as f64 / 1000.)
            } else {
                format!("{}", x)
            }
        }

        pub fn print_node_types(&self, tt_r: &TTRead) {
            let tt_r2 = tt_r.read().unwrap();

            let n_pv = tt_r2.iter().filter(|(_,sis)| {
                sis.iter().next().unwrap().node_type == Node::Exact});
            let n_all = tt_r2.iter().filter(|(_,sis)| {
                sis.iter().next().unwrap().node_type == Node::Upper});
            let n_cut = tt_r2.iter().filter(|(_,sis)| {
                sis.iter().next().unwrap().node_type == Node::Lower});
            // let n_root = tt_r2.iter().filter(|(_,sis)| {
            //     sis.iter().next().unwrap().node_type == Node::Root});

            debug!("n_pv   = {:?}", n_pv.count());
            debug!("n_cut  = {:?}", n_cut.count());
            debug!("n_all  = {:?}", n_all.count());
            // debug!("n_root = {:?}", n_root.collect::<Vec<_>>().len());
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

}

#[derive(Debug,Clone,Copy,Default)]
pub struct RunningAverage {
    average: u64,
}

impl RunningAverage {

    const PERIOD: u64     = 4096;
    const RESOLUTION: u64 = 1024;

    pub fn new(p: u64, q: u64) -> Self {
        let mut out = Self::default();
        out.set(p, q);
        out
    }

    pub fn set(&mut self, p: u64, q: u64) {
        self.average = p * Self::PERIOD * Self::RESOLUTION / q;
    }

    pub fn update(&mut self, v: u64) {
        self.average = Self::RESOLUTION * v
            + (Self::PERIOD - 1) * self.average / Self::PERIOD;
    }

    pub fn is_greater(&self, a: u64, b: u64) -> bool {
        b * self.average > a * Self::PERIOD * Self::RESOLUTION
    }

}

