
use crate::sprt::*;
use crate::sprt::sprt_penta::*;
use crate::tuner_types::RunningTotal;

use log::{debug,trace};

use rand::prelude::SliceRandom;
use rand::{Rng,SeedableRng};
use rand::prelude::StdRng;

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Clone,Copy)]
pub enum WDL {
    Win,
    Draw,
    Loss,
}

impl WDL {
    pub fn gen(w_prob: f64, draw_ratio: f64, rng: &mut StdRng) -> Self {
        debug_assert!(w_prob >= 0.0 && w_prob <= 1.0);
        debug_assert!(draw_ratio >= 0.0 && draw_ratio <= 1.0);

        let x0 = rng.gen_range(0.0..1.0);

        // if x0 < w_prob {
        //     // let x1 = rng.gen_range((w_prob - draw_ratio / 2.)..w_prob);
        //     if x0 > w_prob - draw_ratio / 2. {
        //         Self::Draw
        //     } else {
        //         Self::Win
        //     }
        // } else {
        //     if x0 < w_prob + draw_ratio / 2. {
        //         Self::Draw
        //     } else {
        //         Self::Loss
        //     }
        // }

        if x0 < w_prob - (draw_ratio / 2.) {
            WDL::Win
        } else if x0 > w_prob + (draw_ratio / 2.) {
            WDL::Loss
        } else {
            WDL::Draw
        }

    }
}

pub fn simulate(elo_diff: f64, ab: f64) {
    use WDL::*;
    let mut rng: StdRng = SeedableRng::seed_from_u64(1234);

    /// prob(W + 0.5 * D)
    let w_prob = log_likelyhood(elo_diff);

    let draw_ratio = 0.1;
    // let draw_ratio = 0.02;

    // let (alpha,beta) = (0.05,0.05);
    let (alpha,beta) = (ab,ab);

    eprintln!("win prob   = {:.2}", w_prob);
    eprintln!("draw_ratio = {:.2}", draw_ratio);

    let mut wins   = 0;
    let mut draws  = 0;
    let mut losses = 0;

    let mut total = RunningTotal::default();

    let mut n = 0;
    loop {

        let x0: WDL = WDL::gen(w_prob, draw_ratio, &mut rng);
        let x1: WDL = WDL::gen(w_prob, draw_ratio, &mut rng);

        match x0 {
            Win  => wins += 1,
            Draw => draws += 1,
            Loss => losses += 1,
        }
        match x1 {
            Win  => wins += 1,
            Draw => draws += 1,
            Loss => losses += 1,
        }

        /// LL,LD+DL,LW+DD+WL,DW+WD,WW
        match (x0,x1) {
            (Loss,Loss)                           => total.ll += 1,
            (Loss,Draw) | (Draw,Loss)             => total.ld_dl += 1,
            (Loss,Win) | (Win,Loss) | (Draw,Draw) => total.lw_dd += 1,
            (Draw,Win) | (Win,Draw)               => total.dw_wd += 1,
            (Win,Win)                             => total.ww += 1,
        }

        let elo = 10.;

        // let sprt_penta = sprt_penta(total, (0.0,elo), alpha, beta);
        // // let sprt_penta = sprt_penta(total, (elo,0.0), alpha, beta);
        // if let Some(sprt_penta) = sprt_penta {
        //     eprintln!("n = {:?}", n);
        //     eprintln!("sprt_penta = {:?}", sprt_penta);
        //     // let tot = total.to_vec().into_iter().sum::<u32>() as f64;
        //     // eprintln!("total.ll    = {:.2}", total.ll as f64 / tot);
        //     // eprintln!("total.ld_dl = {:.2}", total.ld_dl as f64 / tot);
        //     // eprintln!("total.lw_dd = {:.2}", total.lw_dd as f64 / tot);
        //     // eprintln!("total.dw_wd = {:.2}", total.dw_wd as f64 / tot);
        //     // eprintln!("total.ww    = {:.2}", total.ww as f64 / tot);
        //     break;
        // }

        // let sprt = sprt((wins,draws,losses), (0.0,elo), alpha, beta);
        let sprt = sprt((wins,draws,losses), (elo,0.0), alpha, beta);
        if let Some(sprt) = sprt {
            eprintln!("n = {:?}", n);
            eprintln!("sprt = {:?}", sprt);

            let llr = ll_ratio((wins,draws,losses), elo, 0.0);
            eprintln!("llr = {:?}", llr);

            let tot = wins + draws + losses;
            let tot = tot as f64;
            let w = wins as f64 / tot;
            let d = draws as f64 / tot;
            let l = losses as f64 / tot;
            eprintln!("(w,d,l) = ({:.2},{:.2},{:.2})", w,d,l);

            // eprintln!("wins   = {:?}", wins);
            // eprintln!("draws  = {:?}", draws);
            // eprintln!("losses = {:?}", losses);

            break;
        }

        n += 1;
        #[cfg(feature = "nope")]
        if n % 1_000_000 == 0 {

            let sprt = sprt((wins,draws,losses), (0.0,200.0), alpha, beta);
            let sprt_penta = sprt_penta(total, (0.0,200.0), alpha, beta);

            println!("sprt       = {:?}", sprt);
            println!("sprt_penta = {:?}", sprt_penta);
            println!("");

        }

        #[cfg(feature = "nope")]
        if n % 1_000_000 == 0 {
            eprintln!();
            eprintln!("run {:>8}", n);
            let tot = wins + draws + losses;
            eprintln!("tot = {:?}", tot);
            let tot = tot as f64;
            let w = wins as f64 / tot;
            let d = draws as f64 / tot;
            let l = losses as f64 / tot;
            eprintln!("(w,d,l) = ({:.2},{:.2},{:.2})", w,d,l);

            let tot = total.to_vec().into_iter().sum::<u32>() as f64;
            eprintln!("total.ll    = {:.2}", total.ll as f64 / tot);
            eprintln!("total.ld_dl = {:.2}", total.ld_dl as f64 / tot);
            eprintln!("total.lw_dd = {:.2}", total.lw_dd as f64 / tot);
            eprintln!("total.dw_wd = {:.2}", total.dw_wd as f64 / tot);
            eprintln!("total.ww    = {:.2}", total.ww as f64 / tot);

            // eprintln!("wins   = {:?}", wins);
            // eprintln!("draws  = {:?}", draws);
            // eprintln!("losses = {:?}", losses);

            // // let llr       = ll_ratio((wins,draws,losses), 0.0, elo_diff);
            // // let llr_penta = ll_ratio_penta(total, 0.0, elo_diff);
            // let llr       = ll_ratio((wins,draws,losses), elo_diff, 0.0);
            // let llr_penta = ll_ratio_penta(total, elo_diff, 0.0);

            // eprintln!("llr       = {:.3}", llr);
            // eprintln!("llr_penta = {:.3}", llr_penta);

            for elo in [50.,150.,190.,210.,250.] {
                eprintln!("elo = {:?}", elo);

                let sprt = sprt((wins,draws,losses), (0.0,elo), alpha, beta);
                let sprt_penta = sprt_penta(total, (0.0,elo), alpha, beta);

                println!("sprt       = {:?}", sprt);
                println!("sprt_penta = {:?}", sprt_penta);
                println!("");

            }

            break;
        }

    }
}




