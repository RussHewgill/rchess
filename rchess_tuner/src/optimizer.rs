

use rchess_engine_lib::types::Color;

use crate::sprt::sprt_penta::*;
use crate::tuner_types::*;
use crate::sprt::*;
use crate::supervisor::{Supervisor,Tunable, CuteChess};

/// algorithm:
///   Engine A = tuning, engine B = baseline
///   1. increase opt by 1 step
///   2. do SPRT test at X Elo
///   3.
///       if H0 (null hyp):
///          increase opt by 1 step again
///       if H1 (A is stronger than B by at least X points):

impl Supervisor {

    fn update_stats(&mut self, wdl: (u32,u32,u32), total: RunningTotal, pairs: &[(Match,Match)]) -> bool {
        if self.sprts.len() == 0 {
            debug!("elo: [{:>3} : {:>3}]", self.brackets[0] as u32, self.brackets[1] as u32);
            return true;
        }

        let tot = wdl.0 + wdl.1 + wdl.2;
        let tot = tot as f64;
        let w = wdl.0 as f64 / tot;
        let d = wdl.1 as f64 / tot;
        let l = wdl.2 as f64 / tot;

        let sum = total.to_vec().into_iter().map(|x| x as f64).sum::<f64>();
        let penta = total.to_vec().into_iter().map(|x| x as f64 / sum).collect::<Vec<_>>();

        let mut min: Option<i32> = None;
        let mut max: Option<i32> = None;

        let mut found = false;
        let t1 = self.t0.elapsed().as_secs_f64();
        for (elo, sprt) in self.sprts.iter_mut() {
            if let Some(hyp) = sprt.sprt_penta(total) {
                if hyp == Hypothesis::H0 {
                    println!();
                    debug!("{:.0} H0 (null): A is NOT stronger than B by at least {} (elo0) points, elo1 = {}",
                           t1, sprt.elo0, sprt.elo1);
                    debug!("found in {} games", pairs.len() * 2);
                    debug!("(w,d,l) = ({:.2},{:.2},{:.2})", w,d,l);
                    debug!("(ll,ld_dl,lw_dd,dw_wd,ww) = ({:>3.2},{:>3.2},{:>3.2},{:>3.2},{:>3.2})",
                           penta[0], penta[1], penta[2], penta[3], penta[4]);
                    max = Some(*elo);
                    self.brackets[1] = *elo as f64;
                    debug!("brackets = {:?}", self.brackets);
                    found = true;
                } else {
                    println!();
                    debug!("{:.0} H1: is that A is stronger than B by at least {} (elo1) ELO points",
                           t1, sprt.elo1);
                    debug!("found in {} games", pairs.len() * 2);
                    debug!("(w,d,l) = ({:.2},{:.2},{:.2})", w,d,l);
                    debug!("(ll,ld_dl,lw_dd,dw_wd,ww) = ({:>3.2},{:>3.2},{:>3.2},{:>3.2},{:>3.2})",
                           penta[0], penta[1], penta[2], penta[3], penta[4]);
                    min = Some(*elo);
                    self.brackets[0] = *elo as f64;
                    debug!("brackets = {:?}", self.brackets);
                    found = true;
                }
                break;
            }
        }

        if let Some(_min) = min {
            self.sprts.retain(|(elo, sprt)| *elo > _min);
        }
        if let Some(_max) = max {
            self.sprts.retain(|(elo, sprt)| *elo < _max);
        }

        if found {
            let mut elos = vec![];
            for (elo,_) in self.sprts.iter() {
                elos.push(elo);
            }
            eprintln!("elos = {:?}", elos);
        }

        false
    }

    #[cfg(feature = "nope")]
    fn update_stats(&mut self, wdl: (u32,u32,u32), total: RunningTotal, pairs: &[(Match,Match)]) -> bool {

        // let (elo0,elo1) = (0,50);
        let (alpha,beta) = (0.05, 0.05);

        // let mut removes = vec![];
        let mut min: Option<i32> = None;
        let mut max: Option<i32> = None;

        if self.sprts.len() == 0 {
            debug!("elo: [{:>3} : {:>3}]", self.brackets[0] as u32, self.brackets[1] as u32);
            return true;
        }

        let tot = wdl.0 + wdl.1 + wdl.2;
        let tot = tot as f64;
        let w = wdl.0 as f64 / tot;
        let d = wdl.1 as f64 / tot;
        let l = wdl.2 as f64 / tot;


        for (elo, sprt) in self.sprts.iter_mut() {

            if let Some(hyp) = sprt.sprt_penta(total) {
            // if let Some(hyp) = sprt.sprt_tri(wdl.0, wdl.1, wdl.2) {
                let t1 = self.t0.elapsed().as_secs_f64();
                if hyp == Hypothesis::H0 {
                    println!();
                    debug!("{:.0} H0 (null): A is NOT stronger than B by at least {} ELO points, elo1 = {}",
                             t1, sprt.elo0, sprt.elo1);
                    debug!("found in {} games", pairs.len() * 2);
                    debug!("(w,d,l) = ({:.2},{:.2},{:.2})", w,d,l);
                    // debug!("{:?}", wdl);

                    // let tot = total.to_vec().into_iter().sum::<u32>() as f64;
                    // debug!("total.ll    = {:.2}", total.ll as f64 / tot);
                    // debug!("total.ld_dl = {:.2}", total.ld_dl as f64 / tot);
                    // debug!("total.lw_dd = {:.2}", total.lw_dd as f64 / tot);
                    // debug!("total.dw_wd = {:.2}", total.dw_wd as f64 / tot);
                    // debug!("total.ww    = {:.2}", total.ww as f64 / tot);

                    max = Some(*elo);
                    self.brackets[1] = *elo as f64;
                    debug!("brackets = {:?}", self.brackets);
                } else {
                    println!();
                    debug!("{:.0} H1: is that A is stronger than B by at least {} ELO points",
                             t1, sprt.elo1);
                    debug!("found in {} games", pairs.len() * 2);
                    debug!("(w,d,l) = ({:.2},{:.2},{:.2})", w,d,l);
                    // debug!("{:?}", wdl);

                    // let tot = total.to_vec().into_iter().sum::<u32>() as f64;
                    // debug!("total.ll    = {:.2}", total.ll as f64 / tot);
                    // debug!("total.ld_dl = {:.2}", total.ld_dl as f64 / tot);
                    // debug!("total.lw_dd = {:.2}", total.lw_dd as f64 / tot);
                    // debug!("total.dw_wd = {:.2}", total.dw_wd as f64 / tot);
                    // debug!("total.ww    = {:.2}", total.ww as f64 / tot);

                    min = Some(*elo);
                    self.brackets[0] = *elo as f64;
                    debug!("brackets = {:?}", self.brackets);
                }
            }

        }

        if let Some(_min) = min {
            self.sprts.retain(|(elo, sprt)| *elo > _min);
        }
        if let Some(_max) = max {
            self.sprts.retain(|(elo, sprt)| *elo < _max);
        }

        false
    }
}

impl Supervisor {

    pub fn spawn_cutechess_mult(&mut self, num_games: u64, threads_per_engine: u32) -> Vec<CuteChess> {

        // let num_cpus  = num_cpus::get();
        let num_pcpus = num_cpus::get_physical();

        let n = num_pcpus / (threads_per_engine as usize * 2);
        debug!("spawning {n} instances of cutechess");

        let mut out = vec![];
        for _ in 0..n {
            let c = self.spawn_cutechess(num_games);
            out.push(c);
        }

        out
    }

    pub fn spawn_cutechess(&mut self, num_games: u64) -> CuteChess {
        let (tx,rx) = self.tx_rx();
        CuteChess::run_cutechess(
            rx,
            tx,
            &self.engine_tuning.name,
            &self.engine_baseline.name,
            self.timecontrol,
            &self.tunable.opt.name,
            num_games)
    }

    pub fn find_optimum(&mut self, num_games: u64, spawn: bool) {
        debug!("starting find_optimum, param: {}", &self.tunable.opt.name);

        let output_label = format!("{}", &self.tunable.opt.name);

        // let (elo0,elo1) = (0,50);
        // let num_games = 1000;

        self.t0 = std::time::Instant::now();

        // let (alpha,beta) = (0.05, 0.05);

        // let cutechess = CuteChess::run_cutechess_tournament(
        //     &self.engine_tuning.name,
        //     &self.engine_baseline.name,
        //     self.timecontrol,
        //     &self.tunable.opt.name,
        //     num_games,
        //     (elo0,elo1),
        //     alpha);

        let cutechess = if spawn {
            self.spawn_cutechess_mult(num_games, 1)
        } else {
            vec![]
        };

        self.listen_loop();
    }

    pub fn listen_loop(&mut self) -> [f64; 2] {

        let mut total = RunningTotal::default();
        let mut wdl = (0,0,0);

        let mut pair: Vec<Match>          = vec![];
        let mut pairs: Vec<(Match,Match)> = vec![];

        let rx = self.get_rx();
        loop {
            match rx.recv() {
                Ok(MatchOutcome::Match(m) | MatchOutcome::SPRTFinished(m,_,_))  => {
                    // pair.push(m);
                    // match (m.engine_a, m.result) {
                    //     (ca, MatchResult::WinLoss(c, _)) => {
                    //         if ca == c {
                    //             wdl.0 += 1;
                    //         } else {
                    //             wdl.2 += 1;
                    //         }
                    //     },
                    //     (_,MatchResult::Draw(_))       => wdl.1 += 1,
                    // }
                    panic!();
                },
                Ok(MatchOutcome::MatchPair(m0, m1)) => {
                    pair.push(m0);
                    pair.push(m1);
                },
                Err(e) => {
                    debug!("recv err = {:?}", e);
                    break;
                },
            }

            // {
            //     let m = &pair[0];
            //     debug!("(m.engine_a,m.result) = {:?}", (m.engine_a,m.result));
            // }

            // match pair.pop().and_then(|m| Some((m.engine_a, m.result))) {
            //     Some((ca, MatchResult::WinLoss(c, _))) => {
            //         if ca == c {
            //             wdl.0 += 1;
            //         } else {
            //             wdl.2 += 1;
            //         }
            //     },
            //     Some((_,MatchResult::Draw(_)))       => wdl.1 += 1,
            //     _ => panic!(),
            // }

            // #[cfg(feature = "nope")]
            if pair.len() == 2 {
                assert!(pair[0].engine_a == Color::White);
                use MatchResult::*;
                use Color::*;

                match pair[0].result {
                    WinLoss(White,_) => wdl.0 += 1,
                    WinLoss(Black,_) => wdl.2 += 1,
                    Draw(_)          => wdl.1 += 1,
                }

                /// LL,LD+DL,LW+DD+WL,DW+WD,WW
                match (pair[0].result, pair[1].result) {
                    (WinLoss(Black,_),WinLoss(White,_)) => total.ll += 1,

                    (WinLoss(Black,_),Draw(_))          => total.ld_dl += 1,
                    (Draw(_),WinLoss(White,_))          => total.ld_dl += 1,

                    (Draw(_),Draw(_))                   => total.lw_dd += 1,
                    (WinLoss(Black,_),WinLoss(Black,_)) => total.lw_dd += 1,
                    (WinLoss(White,_),WinLoss(White,_)) => total.lw_dd += 1,

                    (WinLoss(White,_),Draw(_))          => total.dw_wd += 1,
                    (Draw(_),WinLoss(Black,_))          => total.dw_wd += 1,

                    (WinLoss(White,_),WinLoss(Black,_)) => total.ww += 1,
                }

                pairs.push((pair[0], pair[1]));
                pair.clear();

                if self.update_stats(wdl, total, &pairs) {
                    break;
                }

            }

        }

        // debug!("elo: [{:>3} : {:>3}]", self.brackets[0] as u32, self.brackets[1] as u32);

        self.brackets
    }

}



