
use crate::sprt::*;

use rand::prelude::SliceRandom;
use rand::{Rng,SeedableRng};
use rand::prelude::StdRng;


pub fn simulate(elo_diff: f64) {
    let mut rng: StdRng = SeedableRng::seed_from_u64(1234);

    /// prob(W + 0.5 * D)
    let w_prob = log_likelyhood(elo_diff);


}




