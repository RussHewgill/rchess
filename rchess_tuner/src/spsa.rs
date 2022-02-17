
use rand::prelude::SliceRandom;
use rand::{Rng,SeedableRng};
use rand::prelude::StdRng;

use std::iter::{from_fn,FromFn, Zip};

pub fn test_spsa() {

    let p = 20;

    let theta = vec![1.0; p];

    fn gen_a(a: f64, aa: f64, alpha: f64, len: usize) -> Vec<f64> {
        (0..len).map(|k| a / (k as f64 + 1.0 + aa).powf(alpha)).collect()
    }

    fn gen_c(c: f64, gamma: f64, len: usize) -> Vec<f64> {
        (0..len).map(|k| c / (k as f64 + 1.0).powf(gamma)).collect()
    }

    let a = gen_a(1.0, 100.0, 0.602, theta.len());
    let c = gen_c(1.0, 0.101, theta.len());

    fn y(xs: &[f64]) -> f64 {
        unimplemented!()
    }

    fn delta(rng: &mut StdRng) -> Vec<f64> {
        unimplemented!()
    }

    fn constraint(_: &mut [f64]) {}

    let mut rng: StdRng = SeedableRng::seed_from_u64(1234);

    let mut spsa = SPSAGenerator::new(
        &theta,
        &a,
        &c,
        y,
        delta,
        constraint,
        rng,
    );

}

pub struct SPSAGenerator {
    theta:   Vec<f64>,
    ak_ck:   Vec<(f64,f64)>,

    y:          fn(&[f64]) -> f64,
    delta:      fn(&mut StdRng) -> Vec<f64>,
    constraint: fn(&mut [f64]),

    rng:        StdRng,
}

impl Iterator for SPSAGenerator {
    type Item = Vec<f64>;
    fn next(&mut self) -> Option<Self::Item> {

        let (ak,ck) = self.ak_ck.pop()?;

        let gk = self.estimate_gk(ck);

        for (t, gkk) in self.theta.iter_mut().zip(gk.iter()) {
            *t = *t - ak * gkk;
        }

        (self.constraint)(&mut self.theta);

        Some(self.theta.clone())
    }
}

impl SPSAGenerator {

    pub fn new(
        theta:      &[f64],
        a:          &[f64],
        c:          &[f64],
        y:          fn(&[f64]) -> f64,
        delta:      fn(&mut StdRng) -> Vec<f64>,
        constraint: fn(&mut [f64]),
        rng:        StdRng,
    ) -> Self {
        Self {
            theta: theta.to_vec(),
            ak_ck: a.to_vec().into_iter().zip(c.to_vec().into_iter()).collect(),

            y,
            delta,
            constraint,

            rng,
        }
    }

    fn estimate_gk(
        &mut self,
        ck:      f64,
    ) -> Vec<f64> {

        let delta_k = (self.delta)(&mut self.rng);

        /// Get the two perturbed values of theta
        let ta = self.theta.iter().zip(delta_k.iter())
            .map(|(t,dk)| t + ck * dk).collect::<Vec<_>>();
        let tb = self.theta.iter().zip(delta_k.iter())
            .map(|(t,dk)| t - ck * dk).collect::<Vec<_>>();

        let ya = (self.y)(&ta);
        let yb = (self.y)(&tb);

        let gk = delta_k.into_iter().map(|dk| {
            (ya - yb) / (2.0 * ck * dk)
        }).collect::<Vec<_>>();
        gk
    }

}

