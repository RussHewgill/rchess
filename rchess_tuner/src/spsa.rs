
use rand::prelude::SliceRandom;
use rand::{Rng,SeedableRng};
use rand::prelude::StdRng;



pub fn spsa(
    y:      f64,
    t0:     f64,
    a:      &[f64],
    c:      &[f64],
    delta:  fn(&mut StdRng) -> Vec<f64>,
    rng:    &mut StdRng,
) {

    let theta = t0;

    for (ak,ck) in a.iter().zip(c.iter()) {
        // let gk = 
    }

}

fn estimate_gk(y: f64, theta: f64, delta: fn(&mut StdRng) -> Vec<f64>, ck: f64, rng: &mut StdRng) -> f64 {

    let delta_k = delta(rng);

    unimplemented!()
}

pub fn spsa2(
    a: f64,
    mut params: Vec<i64>,
    rng: &mut StdRng
) {

    let a     = 1.0;
    let b     = 1.0;
    let c     = 1.0;
    let alpha = 0.602;
    let gamma = 0.101;

    let n = 10;

    for k in 0..n {
        let ak = a / (k as f64 + 1.0 + a).powf(alpha);
        let ck = c / (k as f64 + 1.0).powf(gamma);

        for p in params.iter_mut() {
            let x: f64 = rng.gen_range(0.0..1.0);
            let dp = 2.0 * (x / (1.0 + 1.0)).round() - 1.0;
        }

    }

}




