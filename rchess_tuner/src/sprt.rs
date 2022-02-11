use crate::tuner_types::RunningTotal;

use self::helpers::*;
pub use self::helpers::log_likelyhood;


/// https://github.com/glinscott/fishtest/blob/master/server/fishtest/stats/LLRcalc.py
#[cfg(feature = "nope")]
mod gsprt {

    // fn wdr_to_penta(win: u64, draw: u64, loss: u64) -> (u64,u64,u64,u64,u64) {
    //     ()
    // }

    pub fn stats(pdf: &[(f64,f64)]) -> (f64,f64) {

        let eps = 1e-6;
        for x in pdf.iter() {
            assert!(-eps <= x.1);
            assert!(x.1 <= 1.0 + eps);
        }

        let n: f64 = pdf.iter().map(|x| x.1).sum();

        assert!((n - 1.0).abs() < eps);

        let s: f64   = pdf.iter().map(|(value,prob)| value * prob).sum();
        let var: f64 = pdf.iter().map(|(value,prob)| prob * (value - s).powi(2)).sum();

        (s,var)
    }

    // fn llr_jumps(pdf: &[(f64,f64)], s0: f64, s1: f64) -> f64 {
    //     let mut out = vec![];
    //     for i in 0..pdf.len() {
    //         out.push((f64::ln(), pdf[i]))
    //     }
    // }

    // pub fn llr(pdf: &[(f64,f64)]) -> f64 {
    //     unimplemented!()
    // }

    pub fn llr_alt(pdf: &[(f64,f64)], s0: f64, s1: f64) -> f64 {
        let r0: f64 = pdf.iter().map(|(prob,value)| prob * (value - s0).powi(2)).sum();
        let r1: f64 = pdf.iter().map(|(prob,value)| prob * (value - s1).powi(2)).sum();

        1.0 / 2.0 * f64::ln(r0 / r1)
    }

    /// This function computes the approximate generalized log likelihood
    /// ratio (divided by N) for s=s1 versus s=s0 where pdf is an empirical
    /// distribution and s is the expectation value of the true
    /// distribution. See
    /// http://hardy.uhasselt.be/Fishtest/GSPRT_approximation.pdf
    /// // XXX: doesn't work
    pub fn llr_alt2(pdf: &[(f64,f64)], s0: f64, s1: f64) -> f64 {
        // let (s,var) = stats(pdf);
        // (s1 - s0) * (2. * s - s0 - s1) / var / 2.0
        unimplemented!()
    }

    pub fn log_likelyhood(x: f64) -> f64 {
        1.0 / (1.0 + 10.0f64.powf(-x / 400.0))
    }

    pub fn llr_logistic(wdl: (u32,u32,u32), s0: f64, s1: f64) -> f64 {

        let elo0 = log_likelyhood(s0);
        let elo1 = log_likelyhood(s1);

        let (n,pdf) = results_to_pdf(wdl);

        unimplemented!()
    }

    fn regularize(xs: &[f64]) -> Vec<f64> {
        let eps = 1e-3;
        let mut xs = xs.to_vec();
        for x in xs.iter_mut() {
            if *x == 0.0 {
                *x = eps;
            }
        }
        xs
    }

    // fn draw_elo_calc()

    pub fn sprt_elo((win,draw,loss): (u32,u32,u32), elo0: f64, elo1: f64, alpha: f64) -> f64 {

        unimplemented!()
    }

}

pub mod sprt_penta {
    use argmin::solver::brent::Brent;
    use argmin::core::{ArgminOp, ArgminSlogLogger, Error, Executor, ObserverMode};

    use crate::sprt::log_likelyhood;
    use crate::tuner_types::RunningTotal;
    use super::helpers::*;

    struct BrentFunc {
        s:      f64,
        pdf:    Vec<(f64,f64)>,
    }

    impl ArgminOp for BrentFunc {
        type Param    = f64;
        type Output   = f64;
        type Hessian  = ();
        type Jacobian = ();
        type Float    = f64;

        fn apply(&self, x: &Self::Param) -> Result<Self::Output, Error> {
            Ok(self.pdf.iter().map(|(a,p)| p * (a - self.s) / (1. + x * (a - self.s))).sum::<f64>())
        }
    }

    pub fn mle(pdf: &[(f64,f64)], s: f64) -> Vec<(f64,f64)> {
        let eps = 1e-9;

        let v = pdf[0].0;
        let w = pdf.last().unwrap().0;
        assert!(v < s && s < w);

        let l = -1. / (w - s);
        let u = 1. / (s - v);

        // let f = |x: f64| pdf.iter().map(|(a,p)| p * (a - s) / (1. + x * (a - s))).sum::<f64>();

        let b = BrentFunc { s, pdf: pdf.to_vec() };

        let solver = Brent::new(u - eps, l + eps, 2e-12);

        let res = Executor::new(b, solver, 0.0)
            // .add_observer(ArgminSlogLogger::term(), ObserverMode::Always)
            .max_iters(100)
            .run()
            .unwrap();
        let x = res.state.best_param;

        let pdf_mle: Vec<(f64,f64)> = pdf.iter().map(|(a,p)| (*a, p / (1. + x * (a - s)))).collect();

        let (s2, var) = stats(&pdf_mle);
        assert!((s - s2).abs() < 1e-6);

        pdf_mle
        // unimplemented!()
    }

    pub fn llr_jumps(pdf: &[(f64,f64)], s0: f64, s1: f64) -> Vec<(f64,f64)> {
        let pdf0 = mle(&pdf, s0);
        let pdf1 = mle(&pdf, s1);
        let mut out = vec![];
        for i in 0..pdf.len() {
            out.push((
                pdf1[i].1.ln() - pdf0[i].1.ln(),
                pdf[i].1,
            ));
        }
        out
    }

    pub fn ll_ratio(wdl: (u32,u32,u32), elo0: f64, elo1: f64) -> f64 {
        let (s0,s1) = (log_likelyhood(elo0), log_likelyhood(elo1));

        let (sum,pdf) = results_to_pdf(wdl);

        let jumps = llr_jumps(&pdf, s0, s1);

        let llr = stats(&jumps).0;

        sum * llr
    }

    pub fn ll_ratio_penta(results: RunningTotal, elo0: f64, elo1: f64) -> f64 {

        let (s0,s1) = (log_likelyhood(elo0), log_likelyhood(elo1));

        let (sum,pdf) = results_penta_to_pdf(results);

        let jumps = llr_jumps(&pdf, s0, s1);

        let llr = stats(&jumps).0;

        sum * llr
    }

    pub fn ll_ratio_normalized_penta(results: RunningTotal, nelo0: f64, nelo1: f64) -> f64 {
        let (sum,pdf) = results_penta_to_pdf(results);

        let (mu,var) = stats(&pdf);

        let sigma_pg = (2. * var).powf(0.5);
        let games    = 2. * sum;

        let nelo_divided_by_nt = 800.0 / f64::ln(10.); // 347.43558552260146

        let nt0 = nelo0 / nelo_divided_by_nt;
        let nt1 = nelo1 / nelo_divided_by_nt;

        let nt = (mu - 0.5) / sigma_pg;

        (games / 2.0) * f64::ln(
            (1. + (nt - nt0) * (nt - nt0)) / (1. + (nt - nt1) * (nt - nt1))
        )
    }

    pub fn sprt(
        wdl:          (u32,u32,u32),
        (elo0,elo1):  (f64,f64),
        alpha:        f64,
        beta:         f64,
    ) -> Option<bool> {
        let llr = ll_ratio(wdl, elo0, elo1);

        let la = f64::ln(beta / (1.0 - alpha));
        let lb = f64::ln((1.0 - beta) / alpha);

        if llr > lb {
            return Some(true);
        } else if llr < la {
            return Some(false);
        } else {
            None
        }
    }

    pub fn sprt_penta(
        results:      RunningTotal,
        (elo0,elo1):  (f64,f64),
        alpha:        f64,
        beta:         f64,
    ) -> Option<bool> {
        let llr = ll_ratio_penta(results, elo0, elo1);
        // let llr = ll_ratio_normalized_penta(results, elo0, elo1);

        let la = f64::ln(beta / (1.0 - alpha));
        let lb = f64::ln((1.0 - beta) / alpha);

        if llr > lb {
            return Some(true);
        } else if llr < la {
            return Some(false);
        } else {
            None
        }
    }

}

#[cfg(feature = "nope")]
mod prev {

    pub fn ll_ratio((win,draw,loss): (u32,u32,u32), elo0: f64, elo1: f64) -> f64 {
        if win == 0 || draw == 0 || loss == 0 {
            return 0.0;
        }
        let (w,d,l) = (win as f64, draw as f64, loss as f64);

        let n = w + d + l;

        let w = w / n;
        let d = d / n;
        let l = l / n;

        let s     = w + d / 2.0;
        let m2    = w + d / 4.0;
        let var   = m2 - s.powi(2);
        let var_s = var / n;

        let s0 = log_likelyhood(elo0);
        let s1 = log_likelyhood(elo1);

        (s1 - s0) * (2.0 * s - s0 - s1) / var_s / 2.0
    }

    pub fn sprt(
        (win,draw,loss): (u32,u32,u32),
        (elo0,elo1): (f64,f64),
        alpha: f64,
        beta:  f64,
    ) -> Option<bool> {

        let llr = ll_ratio((win,draw,loss), elo0, elo1);

        let la = f64::ln(beta / (1.0 - alpha));
        let lb = f64::ln((1.0 - beta) / alpha);

        if llr > lb {
            return Some(true);
        } else if llr < la {
            return Some(false);
        } else {
            None
        }
    }
}

pub mod helpers {
    use crate::tuner_types::RunningTotal;

    /// expected score = prob(win) + 0.5 * prob(draw)
    pub fn log_likelyhood(elo_diff: f64) -> f64 {
        1.0 / (1.0 + 10.0f64.powf(-elo_diff / 400.0))
    }

    pub fn stats(pdf: &[(f64,f64)]) -> (f64,f64) {

        let eps = 1e-6;
        for x in pdf.iter() {
            assert!(-eps <= x.1);
            assert!(x.1 <= 1.0 + eps);
        }

        let n: f64 = pdf.iter().map(|x| x.1).sum();

        assert!((n - 1.0).abs() < eps);

        let s: f64   = pdf.iter().map(|(value,prob)| value * prob).sum();
        let var: f64 = pdf.iter().map(|(value,prob)| prob * (value - s).powi(2)).sum();

        (s,var)
    }

    pub fn results_penta_to_pdf(results: RunningTotal) -> (f64, Vec<(f64,f64)>) {
        let mut results: Vec<f64> = results.to_vec().into_iter().map(|x| x as f64).collect();
        regularize_mut(&mut results);

        let sum: f64 = results.iter().sum();
        let len = results.len();

        let mut out = vec![];

        for i in 0..len {
            out.push((i as f64 / (len as f64 - 1.0), results[i] / sum))
        }
        (sum, out)
    }

    pub fn results_to_pdf((win,draw,loss): (u32,u32,u32)) -> (f64, Vec<(f64,f64)>) {
        let wdl = regularize(&[win as f64,draw as f64,loss as f64]);

        let sum: f64 = wdl.iter().sum();
        let len = wdl.len();

        let mut out = vec![];

        for i in 0..len {
            out.push((i as f64 / (len as f64 - 1.0), wdl[i] / sum))
        }
        (sum, out)
    }


    pub fn regularize_mut(xs: &mut [f64]) {
        let eps = 1e-3;
        for x in xs.iter_mut() {
            if *x == 0.0 {
                *x = eps;
            }
        }
    }

    pub fn regularize(xs: &[f64]) -> Vec<f64> {
        let eps = 1e-3;
        let mut xs = xs.to_vec();
        for x in xs.iter_mut() {
            if *x == 0.0 {
                *x = eps;
            }
        }
        xs
    }
}



