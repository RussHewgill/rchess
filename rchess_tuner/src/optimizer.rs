

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

        loop {
            match cutechess.rx.recv() {
                Ok(MatchOutcome::Match(m))  => {

                    let wdl = m.sum_score;

                    let llr = ll_ratio(wdl, elo0 as f64, elo1 as f64);
                    let sprt = sprt(wdl, (elo0 as f64, elo1 as f64), alpha, beta);

                    eprintln!("llr = {:?}, sprt = {:?}", llr, sprt);

                    eprintln!("m = {:?}", m);
                },
                Ok(_)  => {
                    unimplemented!()
                }
                Err(e) => {
                    println!("recv err = {:?}", e);
                    break;
                },
            }
        }

        unimplemented!()
    }
}



