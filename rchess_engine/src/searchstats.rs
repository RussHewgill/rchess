
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

    #[derive(Debug,Default,PartialEq,PartialOrd,Clone,Copy)]
    pub struct SearchStats {
        pub nodes:          u32,
        pub nodes_arr:      NArr,
        // pub nodes_zb:       NHashes,
        pub leaves:         u32,
        pub quiet_leaves:   u32,
        pub max_depth:      u8,
        pub q_max_depth:    u8,
        pub checkmates:     u32,
        pub stalemates:     u32,
        pub tt_hits:        u32,
        pub tt_halfmiss:    u32,
        pub tt_misses:      u32,
        pub ph_hits:        u32,
        pub ph_misses:      u32,
        pub qt_nodes:       u32,
        pub qt_hits:        u32,
        pub qt_misses:      u32,
        pub alpha:          i32,
        pub beta:           i32,
        pub ns_pv:          u32,
        pub ns_all:         u32,
        pub ns_cut:         u32,
        pub null_prunes:    u32,
        pub window_fails:   (u32,u32),
        pub lmrs:           (u32,u32),
        pub beta_cut_first: (u32,u32),
        // #[cfg(feature = "pvs_search")]
        // pub pvs_
    }

    // pub struct NHashes

    #[derive(Debug,Eq,PartialEq,Ord,PartialOrd,Clone,Copy)]
    pub struct NArr(pub [u32; 50]);

    impl Default for NArr {
        fn default() -> Self {
            Self([0; 50])
        }
    }

    impl std::ops::Add for SearchStats {
        type Output = Self;
        fn add(self, other: Self) -> Self {
            let mut arr = self.nodes_arr;
            other.nodes_arr.0.iter().enumerate().for_each(|(i,x)| {
                arr.0[i] += x;
            });
            Self {
                nodes:              self.nodes + other.nodes,
                nodes_arr:          arr,
                leaves:             self.leaves + other.leaves,
                quiet_leaves:       self.quiet_leaves + other.quiet_leaves,
                max_depth:          u8::max(self.max_depth, other.max_depth),
                q_max_depth:          u8::max(self.q_max_depth, other.q_max_depth),
                checkmates:         self.checkmates + other.checkmates,
                stalemates:         self.stalemates + other.stalemates,
                tt_hits:            self.tt_hits + other.tt_hits,
                tt_halfmiss:            self.tt_halfmiss + other.tt_halfmiss,
                tt_misses:          self.tt_misses + other.tt_misses,
                ph_hits:            self.ph_hits + other.ph_hits,
                ph_misses:          self.ph_misses + other.ph_misses,
                qt_nodes:           self.qt_nodes + other.qt_nodes,
                qt_hits:            self.qt_hits + other.qt_hits,
                qt_misses:          self.qt_misses + other.qt_misses,
                alpha:              i32::max(self.alpha, other.alpha),
                beta:               i32::min(self.beta, other.beta),
                ns_pv:              self.ns_pv + other.ns_pv,
                ns_all:             self.ns_all + other.ns_all,
                ns_cut:             self.ns_cut + other.ns_cut,
                null_prunes:        self.null_prunes + other.null_prunes,
                window_fails:       Self::_add_2(self.window_fails, other.window_fails),
                lmrs:               Self::_add_2(self.lmrs, other.lmrs),
                beta_cut_first:     Self::_add_2(self.beta_cut_first, other.beta_cut_first),
            }
        }
    }

    impl SearchStats {

        pub fn print(&self, dt: Duration) {
            println!();
            println!("time         = {:.3}s", dt.as_secs_f64());
            println!("nodes        = {}", Self::_print(self.nodes as i32));
            println!("rate         = {:.1} K nodes/s", (self.nodes as f64 / 1000.) / dt.as_secs_f64());
            // println!("max depth    = {}", self.max_depth);
            // println!("leaves       = {}", Self::_print(self.leaves as i32));
            // println!("quiet_leaves = {}", Self::_print(self.quiet_leaves as i32));
            println!("checkmates   = {}", Self::_print(self.checkmates as i32));
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
            // println!("qt_nodes     = {}", Self::_print(self.qt_nodes as i32));
            // println!("qt_hits      = {}", Self::_print(self.qt_hits as i32));
            // println!("qt_misses    = {}", Self::_print(self.qt_misses as i32));

            eprintln!("nodes/qt nodes = {:.1?}", self.qt_nodes as f64 / self.nodes as f64);
            eprintln!("qt nodes    = {}", pretty_print_si(self.qt_nodes as i64));
            eprintln!("q_max_depth = {:?}", self.q_max_depth);

            eprintln!("null prunes = {:?}", self.null_prunes);
            eprintln!("lmrs        = {:?}", self.lmrs);

            let bcs = self.beta_cut_first;
            eprintln!("beta_cut_first = {:.3?}", bcs.0 as f64 / (bcs.0 + bcs.1) as f64);

            // eprintln!("pawn hash hitrate = {:.3}",
            //           self.ph_hits as f64 / (self.ph_hits as f64 + self.ph_misses as f64));

            // eprintln!("stats0.qt_hits = {}", pretty_print_si(stats0.qt_hits as i64));
            // eprintln!("stats0.qt_misses = {}", pretty_print_si(stats0.qt_misses as i64));

            // println!("alpha      = {:?}", self.alpha);
            // println!("beta       = {:?}", self.beta);
            // println!("PV Nodes   = {:?}", self.ns_pv);
            // println!("All Nodes  = {:?}", self.ns_all);
            // println!("Cut Nodes  = {:?}", self.ns_cut);
        }


        fn _add_2<T: std::ops::Add<Output = T>>(a: (T,T), b: (T,T)) -> (T,T) {
            (a.0 + b.0, a.1 + b.1)
        }

        pub fn inc_nodes_arr(&mut self, ply: Depth) {
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
                sis.iter().next().unwrap().node_type == Node::PV});
            let n_all = tt_r2.iter().filter(|(_,sis)| {
                sis.iter().next().unwrap().node_type == Node::All});
            let n_cut = tt_r2.iter().filter(|(_,sis)| {
                sis.iter().next().unwrap().node_type == Node::Cut});
            let n_root = tt_r2.iter().filter(|(_,sis)| {
                sis.iter().next().unwrap().node_type == Node::Root});

            debug!("n_pv   = {:?}", n_pv.collect::<Vec<_>>().len());
            debug!("n_cut  = {:?}", n_cut.collect::<Vec<_>>().len());
            debug!("n_all  = {:?}", n_all.collect::<Vec<_>>().len());
            debug!("n_root = {:?}", n_root.collect::<Vec<_>>().len());
        }

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

        fn print_ebf2(&self, full: bool) {
            let mut arr = self.nodes_arr.clone();
            let k = arr.0.len();
            let dmax = self.max_depth as usize;
            // let mut arr2 = &mut arr[1..((self.max_depth as usize) + 1)];
            let mut arr2 = &mut arr.0[..dmax + 2];
            arr2.reverse();

            // for (depth,n) in arr2.iter().enumerate() {
            //     eprintln!("depth, n = {:>2}: {:>8}", depth, n);
            // }

            let mut xs = vec![];
            for depth in 1..arr2.len() {
                let n = arr2[depth];
                let ebf = n as f64 / arr2[depth - 1] as f64;
                xs.push(ebf);
                if full {
                    debug!("EBF depth {:>2} = {:>8} nodes, {:.2?}", depth, n, ebf);
                }
            }
            let s: f64 = xs.iter().sum();
            debug!("Average EBF: {:.2}", s / xs.len() as f64);

            // let xs = arr.clone().iter()
            //     // .filter(|x| **x != 0)
            //     .enumerate()
            //     .for_each(|(depth,x)| {
            //     eprintln!("x {} = {:?}", depth, x);
            // });

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

