
use std::f64::consts::PI;

use crate::sprt::elo::phi;


#[derive(Debug,Clone,Copy)]
pub struct Brownian {
    a:       f64,
    b:       f64,
    mu:      f64,
    sigma:   f64,
    sigma2:  f64,
}

impl Default for Brownian {
    fn default() -> Self {
        Self {
            a:      -1.0,
            b:      1.0,
            mu:     0.0,
            sigma:  0.005,
            sigma2: 0.005 * 0.005,
        }
    }
}

impl Brownian {
    pub fn new(a: f64, b: f64, mu: f64, sigma: f64) -> Self {
        Self {
            a,
            b,
            mu,
            sigma,
            sigma2: sigma * sigma,
        }
    }
}

impl Brownian {
    pub fn outcome_cdf(&self, t: f64, y: f64) -> f64 {

        let sigma2 = self.sigma2;
        let mu = self.mu;
        let gamma = mu / sigma2;
        let a = self.b - self.a;

        let out = if sigma2 * t / a.powi(2) < 1e-2 || (gamma * a).abs() > 15.0 {
            self.outcome_cdf_alt1(t, y)
        } else {
            self.outcome_cdf_alt2(t, y)
        };

        assert!(-1e-3 <= out && out <= 1.0 + 1e-3);

        out
    }

    fn u(n: f64, gamma: f64, a: f64, y: f64) -> f64 {
        (
            2. * a * gamma * f64::sin(PI * n * y / a)
                - 2. * PI * n * f64::cos(PI * n * y / a)
        ) / (a.powi(2) * gamma.powi(2) + PI.powi(2) * n.powi(2))

    }

    fn outcome_cdf_alt1(&self, t: f64, y: f64) -> f64 {

        let sigma2 = self.sigma2;
        let mu = self.mu;
        let gamma = mu / sigma2;
        let a = self.b - self.a;

        let x = 0.0 - self.a;
        let y = y - self.a;

        let mut n = 1;
        let mut s = 0.0;

        let lambda_1 = ((PI / a).powi(2)) * sigma2 / 2. + (mu.powi(2) / sigma2) / 2.;
        let t0 = f64::exp(-lambda_1 * t - x * gamma + y * gamma);

        let mut lambda_n;
        loop {
            lambda_n = ((n as f64 * PI / a).powi(2)) * sigma2 / 2. + (mu.powi(2) / sigma2) / 2.;
            let t1 = f64::exp(-(lambda_n - lambda_1) * t);
            let t3 = Self::u(n as f64, gamma, a, y);
            let t4 = f64::sin(n as f64 * PI * x / a);
            s += t1 * t3 * t4;
            if (t0 * t1 * t3).abs() <= 1e-9 { break; }
            n += 1;
        }

        let pre = if gamma * a > 30.0 {
            f64::exp(-2. * gamma * x)
        } else if (gamma * a).abs() < 1e-8 {
            (a - x) / a
        } else {
            (1. - f64::exp(2. * gamma * (a - x))) / (1. - f64::exp(2. * gamma * a))
        };

        pre + t0 * s
    }

    fn outcome_cdf_alt2(&self, t: f64, y: f64) -> f64 {

        let denom = f64::sqrt(t * self.sigma2);
        let offset = self.mu * t;
        let gamma = self.mu / self.sigma2;

        let a = self.a;
        let b = self.b;

        let z = (y - offset) / denom;
        let za = (-y + offset + 2. * a) / denom;
        let zb = (y - offset - 2. * b) / denom;

        let t1 = phi(z);

        let t2 = if gamma * a >= 5. {
            -f64::exp(-(za.powi(2)) / 2. + 2. * gamma * a)
                / f64::sqrt(2. * PI)
                * (1. / za - 1. / za.powi(3))
        } else {
            f64::exp(2. * gamma * a) * phi(za)
        };

        let t3 = if gamma * b >= 5. {
            -f64::exp(-(zb.powi(2)) / 2. + 2. * gamma * b)
                / f64::sqrt(2. * PI)
                * (1. / zb - 1. / zb.powi(3))
        } else {
            f64::exp(2. * gamma * b) * phi(zb)
        };

        t1 + t2 - t3
    }

}

