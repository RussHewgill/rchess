
use crate::sprt::*;

use rand::prelude::SliceRandom;
use rand::{Rng,SeedableRng};
use rand::prelude::StdRng;


pub fn simulate(elo_diff: f64) {
    let mut rng: StdRng = SeedableRng::seed_from_u64(1234);

    /// prob(W + 0.5 * D)
    let w_prob = log_likelyhood(elo_diff);

    let draw_ratio = 0.2;

    eprintln!("win prob   = {:.2}", w_prob);
    eprintln!("draw_ratio = {:.2}", draw_ratio);

    let mut wins   = 0;
    let mut draws  = 0;
    let mut losses = 0;

    let mut n = 0;
    loop {
        let x: f64 = rng.gen_range(0.0..1.0);

        if x < w_prob - (draw_ratio / 2.) {
            wins += 1;
        } else if x > w_prob + (draw_ratio / 2.) {
            losses += 1;
        } else {
            draws += 1;
        }

        n += 1;
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

            eprintln!("wins   = {:?}", wins);
            eprintln!("draws  = {:?}", draws);
            eprintln!("losses = {:?}", losses);

            // let llr = ll_ratio((wins,draws,losses), 0.0, elo_diff);
            // eprintln!("llr = {:.3}", llr);

            break;
        }

    }
}




