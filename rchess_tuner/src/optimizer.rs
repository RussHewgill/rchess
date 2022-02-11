

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
///          

impl Supervisor {

    fn update_stats(&mut self, wdl: (u32,u32,u32), total: RunningTotal, pairs: &[(Match,Match)]) {

        // let (elo0,elo1) = (0,50);
        let (alpha,beta) = (0.05, 0.05);

        // for elo_diff in self.hyps.iter() {
        for elo_diff in [50.0] {

            let llr  = ll_ratio(wdl, 0.0, elo_diff);
            let sprt = sprt(wdl, (0.0, elo_diff), alpha, beta);

            let llr_penta  = ll_ratio_penta(total, 0.0, elo_diff);
            let sprt_penta = sprt_penta(total, (0.0, elo_diff), alpha, beta);

            debug!("");
            debug!("(w,d,l) = {:?}", wdl);
            debug!("penta   = {:?}", total);

            debug!("");
            debug!("llr  = {:?}", llr);
            debug!("sprt = {:?}", sprt);

            debug!("");
            debug!("llr_p  = {:?}", llr_penta);
            debug!("sprt_p = {:?}", sprt_penta);

        }

        // let llr = ll_ratio(wdl, elo0 as f64, elo1 as f64);
        // let sprt = sprt(wdl, (elo0 as f64,elo1 as f64), alpha, beta);

    }
}

impl Supervisor {
    pub fn find_optimum(&mut self) {
        debug!("starting find_optimum, param: {}", &self.tunable.opt.name);

        let output_label = format!("{}", &self.tunable.opt.name);

        // let (elo0,elo1) = (0,50);
        let num_games = 1000;

        // let (alpha,beta) = (0.05, 0.05);

        // let cutechess = CuteChess::run_cutechess_tournament(
        //     &self.engine_tuning.name,
        //     &self.engine_baseline.name,
        //     self.timecontrol,
        //     &self.tunable.opt.name,
        //     num_games,
        //     (elo0,elo1),
        //     alpha);

        let cutechess = CuteChess::run_cutechess(
            &self.engine_tuning.name,
            &self.engine_baseline.name,
            self.timecontrol,
            &self.tunable.opt.name,
            num_games);

        let mut total = RunningTotal::default();
        let mut wdl = (0,0,0);

        let mut pair: Vec<Match>          = vec![];
        let mut pairs: Vec<(Match,Match)> = vec![];

        loop {
            match cutechess.rx.recv() {
                Ok(MatchOutcome::Match(m) | MatchOutcome::SPRTFinished(m,_,_))  => {
                    pair.push(m);
                    match (m.engine_a, m.result) {
                        (ca, MatchResult::WinLoss(c, _)) => {
                            if ca == c {
                                wdl.0 += 1;
                            } else {
                                wdl.2 += 1;
                            }
                        },
                        (_,MatchResult::Draw(_))       => wdl.1 += 1,
                    }
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

                pairs.push((pair[0], pair[1]));
                pair.clear();

                self.update_stats(wdl, total, &pairs);

            }

        }
    }
}



