
use rand::prelude::SliceRandom;
use rand::{Rng,SeedableRng};
use rand::prelude::StdRng;

use std::iter::{from_fn,FromFn, Zip};

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

    // pub fn spsa_next(
    //     &mut self,
    //     y:          fn(&[f64]) -> f64,
    //     theta0:     &[f64],
    //     a:          &[f64],
    //     c:          &[f64],
    //     delta:      fn(&mut StdRng) -> Vec<f64>,
    //     rng:        &mut StdRng,
    //     constraint: fn(Vec<f64>) -> Vec<f64>,
    // ) -> Option<Vec<f64>> {
    //     unimplemented!()
    // }

    // /// y:  fn(theta: &[f64]) -> f64
    // pub fn spsa(
    //     y:          fn(&[f64]) -> f64,
    //     theta0:     &[f64],
    //     a:          &[f64],
    //     c:          &[f64],
    //     delta:      fn(&mut StdRng) -> Vec<f64>,
    //     rng:        &mut StdRng,
    //     constraint: fn(Vec<f64>) -> Vec<f64>,
    // ) -> Self {

    //     let mut theta = theta0.to_vec();

    //     let xs = a.iter();
    //     // let xs = a.iter().zip(c.iter());

    //     for (ak,ck) in a.iter().zip(c.iter()) {
    //         let gk = Self::estimate_gk(y, &theta, delta, *ck, rng);

    //         for (t, gkk) in theta.iter_mut().zip(gk.iter()) {
    //             *t = *t - ak * gkk;
    //         }

    //         theta = constraint(theta);

    //     }

    //     unimplemented!()
    // }

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

    // fn estimate_gk(
    //     y:       fn(&[f64]) -> f64,
    //     theta:   &[f64],
    //     delta:   fn(&mut StdRng) -> Vec<f64>,
    //     ck:      f64,
    //     rng:     &mut StdRng,
    // ) -> Vec<f64> {

    //     let delta_k = delta(rng);

    //     /// Get the two perturbed values of theta
    //     let ta = theta.iter().zip(delta_k.iter())
    //         .map(|(t,dk)| t + ck * dk).collect::<Vec<_>>();
    //     let tb = theta.iter().zip(delta_k.iter())
    //         .map(|(t,dk)| t - ck * dk).collect::<Vec<_>>();

    //     let ya = y(&ta);
    //     let yb = y(&tb);

    //     let gk = delta_k.into_iter().map(|dk| {
    //         (ya - yb) / (2.0 * ck * dk)
    //     }).collect::<Vec<_>>();
    //     gk
    // }

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




