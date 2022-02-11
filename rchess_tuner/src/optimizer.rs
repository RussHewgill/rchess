

use rchess_engine_lib::types::Color;

use crate::sprt::sprt_penta::ll_ratio;
use crate::sprt::sprt_penta::sprt;
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
///          

impl Supervisor {

    fn update_stats(&mut self, wdl: (u32,u32,u32)) {
        unimplemented!()
    }

    fn update_stats_penta(&mut self, total: &RunningTotal) {
        unimplemented!()
    }
}

impl Supervisor {
    pub fn find_optimum(&mut self) {
        debug!("starting find_optimum, param: {}", &self.tunable.opt.name);

        let output_label = format!("{}", &self.tunable.opt.name);

        let (elo0,elo1) = (0,50);
        let num_games = 50;

        let (alpha,beta) = (0.05, 0.05);

        let cutechess = CuteChess::run_cutechess_tournament(
            &self.engine_tuning.name,
            &self.engine_baseline.name,
            self.timecontrol,
            &self.tunable.opt.name,
            num_games,
            (elo0,elo1),
            alpha);

        // let mut total = RunningTotal::default();
        let mut total = (0,0,0);

        let mut pair: Vec<Match>          = vec![];
        let mut pairs: Vec<(Match,Match)> = vec![];

        loop {
            match cutechess.rx.recv() {
                Ok(MatchOutcome::Match(m))  => {

                    pair.push(m);

                    // let wdl = m.sum_score;
                    // let llr = ll_ratio(wdl, elo0 as f64, elo1 as f64);
                    // let sprt = sprt(wdl, (elo0 as f64, elo1 as f64), alpha, beta);

                    // eprintln!("llr = {:?}, sprt = {:?}", llr, sprt);
                    // eprintln!("m = {:?}", m);
                },
                Ok(_)  => {
                    unimplemented!()
                }
                Err(e) => {
                    debug!("recv err = {:?}", e);
                    break;
                },
            }

            match pair.pop().and_then(|m| Some((m.engine_a, m.result))) {
                Some((ca, MatchResult::WinLoss(c, _))) => {
                    if ca == c {
                        total.0 += 1;
                    } else {
                        total.2 += 1;
                    }
                },
                Some((_,MatchResult::Draw(_)))       => total.1 += 1,
                _ => panic!(),
            }

            debug!("(w,d,l) = {:?}", total);

            let llr = ll_ratio(total, elo0 as f64, elo1 as f64);
            let sprt = sprt(total, (elo0 as f64,elo1 as f64), alpha, beta);

            debug!("llr  = {:?}", llr);
            debug!("sprt = {:?}", sprt);
            debug!("");

            #[cfg(feature = "nope")]
            if pair.len() == 2 {
                assert!(pair[0].engine_a == Color::White);
                use MatchResult::*;
                use Color::*;

                /// LL,LD+DL,LW+DD+WL,DW+WD,WW
                match (pair[0].result, pair[1].result) {
                    (WinLoss(Black,_),WinLoss(White,_)) => total.ll += 1,

                    (WinLoss(Black,_),Draw(_))          => total.ld_dl += 1,
                    (Draw(_),WinLoss(White,_))          => total.ld_dl += 1,

                    (Draw(_),Draw(_))                   => total.lw_dd += 1,
                    (WinLoss(Black,_),WinLoss(Black,_)) => total.lw_dd += 1,
                    (WinLoss(White,_),WinLoss(White,_)) => total.lw_dd += 1,

                    (WinLoss(White,_),Draw(_))          => total.lw_dd += 1,
                    (Draw(_),WinLoss(Black,_))          => total.lw_dd += 1,

                    (WinLoss(White,_),WinLoss(Black,_)) => total.ww += 1,
                }

                self.update_stats(&total);

                pairs.push((pair[0], pair[1]));
                pair.clear();
            }

        }

        unimplemented!()
    }
}



