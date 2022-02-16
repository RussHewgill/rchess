use crate::tuner_types::RunningTotal;

use self::helpers::*;
pub use self::helpers::log_likelyhood;


/// https://github.com/glinscott/fishtest/blob/master/server/fishtest/stats/LLRcalc.py
#[cfg(feature = "nope")]
pub mod gsprt {

    // fn wdr_to_penta(win: u64, draw: u64, loss: u64) -> (u64,u64,u64,u64,u64) {
    //     ()
    // }

    // pub fn stats(pdf: &[(f64,f64)]) -> (f64,f64) {

    //     let eps = 1e-6;
    //     for x in pdf.iter() {
    //         assert!(-eps <= x.1);
    //         assert!(x.1 <= 1.0 + eps);
    //     }

    //     let n: f64 = pdf.iter().map(|x| x.1).sum();

    //     assert!((n - 1.0).abs() < eps);

    //     let s: f64   = pdf.iter().map(|(value,prob)| value * prob).sum();
    //     let var: f64 = pdf.iter().map(|(value,prob)| prob * (value - s).powi(2)).sum();

    //     (s,var)
    // }

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
        let r0: f64 = pdf.iter().map(|(value,prob)| prob * (value - s0).powi(2)).sum();
        let r1: f64 = pdf.iter().map(|(value,prob)| prob * (value - s1).powi(2)).sum();

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

    // pub fn llr_logistic(wdl: (u32,u32,u32), s0: f64, s1: f64) -> f64 {
    //     let elo0 = log_likelyhood(s0);
    //     let elo1 = log_likelyhood(s1);
    //     let (n,pdf) = results_to_pdf(wdl);
    //     unimplemented!()
    // }

    // fn regularize(xs: &[f64]) -> Vec<f64> {
    //     let eps = 1e-3;
    //     let mut xs = xs.to_vec();
    //     for x in xs.iter_mut() {
    //         if *x == 0.0 {
    //             *x = eps;
    //         }
    //     }
    //     xs
    // }

    // fn draw_elo_calc()

    pub fn sprt_elo((win,draw,loss): (u32,u32,u32), elo0: f64, elo1: f64, alpha: f64) -> f64 {

        unimplemented!()
    }

}

pub mod sprt_penta {
    use argmin::solver::brent::Brent;
    use argmin::core::{ArgminOp, ArgminSlogLogger, Error, Executor, ObserverMode};

    use crate::brownian::*;
    use crate::sprt::log_likelyhood;
    use crate::tuner_types::{RunningTotal, Hypothesis};
    use crate::sprt::elo::EloType;
    use crate::sprt::helpers::*;
    use crate::sprt::elo::*;

    // #[cfg(feature = "nope")]
    pub fn mle(pdf: &[(f64,f64)], s: f64) -> Vec<(f64,f64)> {

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

    pub fn mle_expected(pdfhat: &[(f64,f64)], s: f64) -> Vec<(f64,f64)> {
        // let pdf1 = pdfhat.iter().map(|(ai,pi)| (ai - s, pi))
        unimplemented!()
    }

    pub fn mle_t_value(pdf: &[(f64,f64)], s: f64) -> Vec<(f64,f64)> {
        unimplemented!()
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

    pub fn ll_ratio(ldw: (u32,u32,u32), elo0: f64, elo1: f64) -> f64 {
        let (s0,s1) = (log_likelyhood(elo0), log_likelyhood(elo1));

        let (sum,pdf) = results_to_pdf(ldw);

        let jumps = llr_jumps(&pdf, s0, s1);

        let llr = stats(&jumps).0;

        sum * llr
    }

    pub fn ll_ratio_logistic_penta(results: RunningTotal, elo0: f64, elo1: f64) -> f64 {

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

    #[derive(Debug,Clone,Copy)]
    pub struct SPRT {
        pub elo0:     f64,
        pub elo1:     f64,
        pub alpha:    f64,
        pub beta:     f64,

        /// LA
        llr_lower:           f64,
        /// LB
        llr_upper:           f64,

        elo_type:     EloType,
        sigma_pg:     f64,
        t:            f64,
        llr:          f64,

        sq0:          f64,
        sq1:          f64,
        max_llr:      f64,
        min_llr:      f64,
        o0:           f64,
        o1:           f64,

    }

    /// new
    impl SPRT {
        pub fn new_def_ab(elo0: f64, elo1: f64) -> Self {
            Self::new(elo0, elo1, 0.05)
        }
        pub fn new_with_elo_type(elo0: f64, elo1: f64, ab: f64, elo_type: EloType) -> Self {
            let mut out = Self::new(elo0, elo1, 0.05);
            out.elo_type = elo_type;
            out
        }
        pub fn new(elo0: f64, elo1: f64, ab: f64) -> Self {
            let (alpha,beta) = (ab,ab);
            Self {
                elo0,
                elo1,
                alpha,
                beta,
                llr_lower: f64::ln(beta / (1.0 - alpha)),
                llr_upper: f64::ln((1.0 - beta) / alpha),

                elo_type: EloType::Logistic,
                sigma_pg: 0.0,
                t:        0.0,
                llr:      0.0,

                sq0:     0.0,
                sq1:     0.0,
                max_llr: 0.0,
                min_llr: 0.0,
                o0:      0.0,
                o1:      0.0,
            }
        }
    }

    /// run, fishtest
    #[cfg(feature = "nope")]
    impl SPRT {
        pub fn sprt_penta(&mut self, results: RunningTotal) -> Option<Hypothesis> {

            let (sum,pdf) = results_penta_to_pdf(results);

            if self.elo_type == EloType::Normalized {
                let (mu,var) = stats(&pdf);
                if pdf.len() == 5 {
                    self.sigma_pg = (2. * var).powf(0.5);
                } else if pdf.len() == 3 {
                    self.sigma_pg = var.powf(0.5);
                } else { panic!(); }
            }

            let (s0,s1) = (self.elo_to_score(self.elo0),self.elo_to_score(self.elo1));

            let (mu_llr,var_llr) = Self::llr_drift_variance(&pdf, s0, s1, None);

            self.llr = sum * mu_llr;
            self.t = sum;

            let mut clamped = false;

            eprintln!("llr = {:?}", self.llr);

            let slope = self.llr / sum;

            if self.llr > 1.03 * self.llr_upper || self.llr < 1.03 * self.llr_lower {
                // clamped = true;
            }
            if self.llr < self.llr_lower {
                self.t   = self.llr_lower / slope;
                self.llr = self.llr_lower;
            } else if self.llr > self.llr_upper {
                self.t   = self.llr_upper / slope;
                self.llr = self.llr_upper;
            }

            let outcome_prob = self.outcome_prob(&pdf, 0.0);

            eprintln!("outcome_prob = {:?}", outcome_prob);

            // let elo = 

            unimplemented!()
        }

        fn outcome_prob(&self, pdf: &[(f64,f64)], elo: f64) -> f64 {
            let s = log_likelyhood(elo);

            let (s0,s1) = (self.elo_to_score(self.elo0),self.elo_to_score(self.elo1));

            let (mu_llr, var_llr) = Self::llr_drift_variance(pdf, s0, s1, Some(s));

            let sigma_llr = f64::sqrt(var_llr);

            Brownian::new(self.llr_lower, self.llr_upper, mu_llr, sigma_llr)
                .outcome_cdf(self.t, self.llr)
        }

        fn lower_cb(&self, p: f64) -> f64 {

            struct BrentFunc {
                p: f64
            }

            impl ArgminOp for BrentFunc {
                type Param    = f64;
                type Output   = f64;
                type Hessian  = ();
                type Jacobian = ();
                type Float    = f64;

                fn apply(&self, elo: &Self::Param) -> Result<Self::Output, Error> {
                    // Ok(self.pdf.iter().map(|(a,p)| p * (a - self.s) / (1. + x * (a - self.s))).sum::<f64>())
                    // Ok()
                    unimplemented!()
                }
            }

            let avg_elo = (self.elo0 + self.elo1) / 2.0;
            let delta = self.elo1 - self.elo0;

            let n = 30;

            // let mut elo0;
            // let mut elo1;

            loop {

                // elo0 = f64::max(avg_elo - n as f64 * delta, -1000.);
                // elo1 = f64::max(avg_elo + n as f64 * delta, 1000.);

                // let b = BrentFunc { s, pdf: pdf.to_vec() };

                // let solver = Brent::new(u - eps, l + eps, 2e-12);

                // let res = Executor::new(b, solver, 0.0)
                // // .add_observer(ArgminSlogLogger::term(), ObserverMode::Always)
                //     .max_iters(100)
                //     .run()
                //     .unwrap();
                // let x = res.state.best_param;

                break;
            }

            unimplemented!()
        }

        fn llr_drift_variance(pdf: &[(f64,f64)], s0: f64, s1: f64, s: Option<f64>) -> (f64,f64) {

            let (s2,v2) = stats(pdf);

            let (s,v) = if let Some(s) = s {
                (s, v2 + (s - s2).powi(2))
            } else {
                (s2,v2)
            };

            let mu = (s - (s0 + s1) / 2.0) * (s1 - s0) / v;
            let var = (s1 - s0).powi(2) / v;

            (mu,var)
        }

        fn elo_to_score(&self, elo: f64) -> f64 {
            match self.elo_type {
                EloType::Normalized => {
                    let nt = elo / (800. / f64::ln(10.));
                    nt * self.sigma_pg + 0.5
                },
                EloType::Logistic => log_likelyhood(elo),
                _ => unimplemented!(),
            }
        }
    }

    /// run
    // #[cfg(feature = "nope")]
    impl SPRT {

        #[cfg(feature = "nope")]
        pub fn sprt_tri(&self, wins: u32, draws: u32, losses: u32) -> Option<Hypothesis> {
            if !(wins > 0 && draws > 0 && losses > 0) { return None; }
            let (wins,draws,losses) = (wins as f64,draws as f64,losses as f64);
            let sum = wins + draws + losses;

            let (elo, draw_elo) = prob_to_bayes_elo(wins / sum, losses / sum);

            let (p0win, p0draw, p0loss) = bayes_elo_to_prob(self.elo0, draw_elo);
            let (p1win, p1draw, p1loss) = bayes_elo_to_prob(self.elo1, draw_elo);

            let llr = wins * f64::ln(p1win / p0win)
                + losses * f64::ln(p1loss / p0loss)
                + draws * f64::ln(p1draw / p0draw);

            // eprintln!("llr = {:?}", llr);

            if llr > self.llr_upper {
                // passed
                Some(Hypothesis::H1)
            } else if llr < self.llr_lower {
                // failed
                Some(Hypothesis::H0)
            } else {
                None
            }
        }

        pub fn sprt_tri(&self, wins: u32, draws: u32, losses: u32) -> Option<Hypothesis> {
            use super::prev::*;

            match sprt((wins,draws,losses), (self.elo0, self.elo1), self.alpha, self.beta) {
                None        => None,
                Some(true)  => Some(Hypothesis::H1),
                Some(false) => Some(Hypothesis::H0),
            }

        }

        pub fn sprt_penta(&mut self, results: RunningTotal) -> Option<Hypothesis> {

            let llr = match self.elo_type {
                EloType::Logistic   => ll_ratio_logistic_penta(results, self.elo0, self.elo1),
                EloType::Normalized => ll_ratio_normalized_penta(results, self.elo0, self.elo1),
                _                   => unimplemented!(),
            };

            // eprintln!("llr = {:?}", llr);

            /// Dynamic overshoot correction using
            /// Siegmund - Sequential Analysis - Corollary 8.33.
            if llr > self.max_llr {
                self.sq1 += (llr - self.max_llr).powi(2);
                self.max_llr = llr;
                self.o1 = self.sq1 / llr / 2.0;
            }
            if llr > self.min_llr {
                self.sq1 += (llr - self.min_llr).powi(2);
                self.min_llr = llr;
                self.o0 = -self.sq0 / llr / 2.0;
            }

            if llr > self.llr_upper - self.o1 {
                Some(Hypothesis::H1)
            } else if llr < self.llr_lower + self.o0 {
                Some(Hypothesis::H0)
            } else {
                None
            }
        }

    }

    impl SPRT {
        pub fn elo_logistic_to_elo(&mut self, pdf: &[(f64,f64)], lelo: f64) -> f64 {

            if self.elo_type == EloType::Logistic {
                panic!();
            }

            let (mu,var) = stats(pdf);

            if pdf.len() == 5 {
                self.sigma_pg = (2. * var).powf(0.5);
            } else if pdf.len() == 3 {
                self.sigma_pg = var.powf(0.5);
            } else { panic!(); }

            let score = log_likelyhood(lelo);
            let nt = (score - 0.5) / self.sigma_pg;

            nt * (800.0 / f64::ln(10.))
        }
    }

    #[cfg(feature = "nope")]
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

    #[cfg(feature = "nope")]
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

// #[cfg(feature = "nope")]
pub mod prev {
    use super::log_likelyhood;

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

    // pub fn results_to_pdf((win,draw,loss): (u32,u32,u32)) -> (f64, Vec<(f64,f64)>) {
    //     let wdl = regularize(&[win as f64,draw as f64,loss as f64]);
    pub fn results_to_pdf(ldw: (u32,u32,u32)) -> (f64, Vec<(f64,f64)>) {
        let ldw = regularize(&[ldw.0 as f64, ldw.1 as f64, ldw.2 as f64]);

        let sum: f64 = ldw.iter().sum();
        let len = ldw.len();

        let mut out = vec![];

        for i in 0..len {
            out.push((i as f64 / (len as f64 - 1.0), ldw[i] / sum))
        }
        (sum, out)
        // unimplemented!()
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

pub mod elo {
    use crate::tuner_types::RunningTotal;

    use super::helpers::*;
    use statrs::distribution::{Continuous,ContinuousCDF};

    #[derive(Debug,PartialEq,Clone,Copy)]
    pub enum EloType {
        Normalized,
        Logistic,
        Bayes,
    }

    pub fn stats2(results: &[f64]) -> (f64,f64,f64) {
        let len = results.len();
        let n: f64 = results.iter().sum();

        let games = n * (len as f64 - 1.0) / 2.0;

        let mu = (0..len).map(|i| results[i] * (i as f64 / 2.0)).sum::<f64>() / games;

        let mu2 = (len as f64 - 1.0) / 2.0 * mu;

        let var = (0..len).map(|i| results[i] * (i as f64 / 2.0 - mu2).powi(2)).sum::<f64>() / games;

        (games,mu,var)
    }

    pub fn phi(q: f64) -> f64 {
        let n = statrs::distribution::Normal::new(0.0, 1.0).unwrap();
        n.cdf(q)
    }

    pub fn phi_inv(p: f64) -> f64 {
        let n = statrs::distribution::Normal::new(0.0, 1.0).unwrap();
        n.inverse_cdf(p)
    }

    pub fn get_elo(ldw: (u32,u32,u32)) -> (f64,(f64,f64,f64)) {
        let mut results = vec![ldw.0 as f64,ldw.1 as f64,ldw.2 as f64];
        regularize_mut(&mut results);
        _get_elo(&results)
    }

    pub fn get_elo_penta(results: RunningTotal) -> (f64,(f64,f64,f64)) {
        let mut results: Vec<f64> = results.to_vec().into_iter().map(|x| x as f64).collect();
        regularize_mut(&mut results);
        _get_elo(&results)
    }

    pub fn _get_elo(results: &[f64]) -> (f64,(f64,f64,f64)) {
        let (games,mu,var) = stats2(&results);

        let stddev = var.sqrt();

        let mu_min = mu + phi_inv(0.025) * stddev / games.sqrt();
        let mu_max = mu + phi_inv(0.975) * stddev / games.sqrt();

        fn f_elo(mut x: f64) -> f64 {
            let eps = 1e-3;
            x = x.max(eps);
            x = x.min(1. - eps);
            -400.0 * f64::log10(1. / x - 1.)
        }

        let elo = f_elo(mu);

        let elo95 = (f_elo(mu_max) - f_elo(mu_min)) / 2.0;

        let los = phi((mu - 0.5) / (stddev / games.sqrt()));

        (elo, (elo95, los, stddev))
    }

    /// from OpenBench
    pub fn elo_tri(wins: u32, draws: u32, losses: u32) -> (f64,(f64,f64)) {
        let (wins,draws,losses) = (wins as f64,draws as f64,losses as f64);

        fn _elo(x: f64) -> f64 {
            if x <= 0. || x >= 1. { panic!()
            } else { -400. * f64::log10(1. / x - 1.) }
        }

        let sum = wins + draws + losses;

        let w = wins / sum;
        let d = draws / sum;
        let l = losses / sum;

        let mu = w + d / 2.0;

        let stddev = f64::sqrt(w * (1. - mu).powi(2)
                               + l * (0.0 - mu).powi(2)
                               + d * (0.5 - mu).powi(2))
            / sum.sqrt();

        let mu_min = mu + phi_inv(0.025) * stddev;
        let mu_max = mu + phi_inv(0.975) * stddev;

        (_elo(mu), (_elo(mu_min), _elo(mu_max)))
    }

    #[cfg(feature = "nope")]
    /// from cutechess
    pub fn get_elo(wdl: (u32,u32,u32)) -> (f64,f64) {
        // let wdl = regularize(&[wdl.0 as f64,wdl.1 as f64,wdl.2 as f64]);
        let (w,d,l) = wdl;
        let (wins,draws,losses) = (w as f64,d as f64,l as f64);

        let n = wins + draws + losses;
        let w = wins / n;
        let d = draws / n;
        let l = losses / n;

        let mu = w + d / 2.0;

        let dev_w = w * f64::powi(1.0 - mu, 2);
        let dev_l = l * f64::powi(0.0 - mu, 2);
        let dev_d = d * f64::powi(0.5 - mu, 2);

        let std_dev = f64::sqrt(dev_w + dev_l + dev_d) / f64::sqrt(n);

        let mu_min = mu + phi_inv(0.025) * std_dev;
        let mu_max = mu + phi_inv(0.975) * std_dev;

        fn diff(x: f64) -> f64 {
            -400.0 * f64::log10(1.0 / x - 1.0)
        }

        let err = (diff(mu_max) - diff(mu_min)) / 2.0;

        let elo = diff(mu);

        eprintln!("mu      = {:.3}", mu);
        eprintln!("std_dev = {:.3}", std_dev);
        eprintln!("err     = {:.3}", err);
        eprintln!("elo     = {:?}", elo);

        unimplemented!()
    }

    // pub fn elo_logistic_to_normalized(lelo: f64) -> f64 {
    //     // let score = log_likelyhood(lelo);
    //     unimplemented!()
    // }

    // pub fn elo_normalized_to_logistic(nelo: f64) -> f64 {
    //     unimplemented!()
    // }

    pub fn calc_draw_elo(ldw: (u32,u32,u32)) -> f64 {
        let ldw = [ldw.0 as f64, ldw.1 as f64, ldw.2 as f64];
        let n: f64 = ldw.iter().sum();
        let p = ldw.iter().map(|p| *p / n).collect::<Vec<_>>();
        let (_,draw_elo) = prob_to_bayes_elo(p[2], p[0]);
        draw_elo
    }

    pub fn elo_logistic_to_bayes_elo(elo: f64, draw_ratio: f64) -> (f64,f64) {
        assert!(draw_ratio >= 0.);

        let s = log_likelyhood(elo);

        let w = s - draw_ratio / 2.0;
        let d = draw_ratio;
        let l = 1.0 - d - w;

        if w <= 0.0 || l <= 0.0 {
            panic!();
        }

        let (elo,draw_elo) = prob_to_bayes_elo(w,l);
        (elo,draw_elo)
    }

    pub fn prob_to_bayes_elo(p_win: f64, p_loss: f64) -> (f64,f64) {
        assert!(0.0 < p_win && p_win < 1.0 && 0.0 < p_loss && p_loss < 1.0);
        let elo      = 200.0 * f64::log10(p_win / p_loss * (1. - p_loss) / (1. - p_win));
        let draw_elo = 200.0 * f64::log10((1. - p_loss) / p_loss * (1. - p_win) / p_win);
        (elo,draw_elo)
    }

    pub fn bayes_elo_to_prob(belo: f64, draw_elo: f64) -> (f64,f64,f64) {
        let w = 1.0 / (1.0 + f64::powf(10.0, (-belo + draw_elo) / 400.0));
        let l = 1.0 / (1.0 + f64::powf(10.0, (belo + draw_elo) / 400.0));
        let d = 1.0 - w - l;
        (w,l,d)
    }

    pub fn bayes_elo_to_logistic(belo: f64, draw_elo: f64) -> f64 {
        let (w,d,l) = bayes_elo_to_prob(belo, draw_elo);
        elo(w + 0.5 * d)
    }

    pub fn elo(mut x: f64) -> f64 {
        let eps = 1e-3;
        x = x.max(eps);
        x = x.min(1. - eps);
        -400. * f64::log10(1. / x - 1.)
    }

    #[cfg(feature = "nope")]
    pub fn elo_to_bayes_elo(elo: f64) -> f64 {
        use argmin::solver::brent::Brent;
        use argmin::core::{ArgminOp, ArgminSlogLogger, Error, Executor, ObserverMode};

        let draw_elo = 327.;
        let biases   = [-90., 200.];

        struct BrentFunc {
            s: f64,
        }

        // fn _probs(biases: [f64; 2], )

        // fn score(probs: &[f64]) -> 

        impl ArgminOp for BrentFunc {
            type Param    = f64;
            type Output   = f64;
            type Hessian  = ();
            type Jacobian = ();
            type Float    = f64;

            fn apply(&self, x: &Self::Param) -> Result<Self::Output, Error> {
                // Ok(self.pdf.iter().map(|(a,p)| p * (a - self.s) / (1. + x * (a - self.s))).sum::<f64>())
                unimplemented!()
            }
        }

        let bb = f64::ln(10.0) / 400.0;

        let s = if elo >= 0.0 {
            1. / (1. + f64::exp(-bb * elo))
        } else {
            let e = f64::exp(bb * elo);
            e / (e + 1.)
        };

        // let b = BrentFunc { s, };

        // let solver = Brent::new(-1000, 1000, 2e-12);
        // let res = Executor::new(b, solver, 0.0)
        // // .add_observer(ArgminSlogLogger::term(), ObserverMode::Always)
        //     .max_iters(100)
        //     .run()
        //     .unwrap();
        // let x = res.state.best_param;

        unimplemented!()
    }


}

pub mod random {
    use crate::simulate::{WDL, PentaWDL};
    use crate::sprt::elo::elo_logistic_to_bayes_elo;

    use rand::prelude::SliceRandom;
    use rand::{Rng,SeedableRng};
    use rand::prelude::StdRng;

    // pub fn pick(elo: f64, draw_elo: f64, biases: [f64; 2], rng: &mut StdRng) -> PentaWDL {
    // #[cfg(feature = "nope")]
    pub fn pick(elo: f64, biases: [f64; 2], rng: &mut StdRng) -> PentaWDL {

        let draw_ratio = 0.74; /// gives ~327 draw_elo

        // let belo = elo_to_belo(elo);
        let (belo, draw_elo) = elo_logistic_to_bayes_elo(elo, draw_ratio);
        let bias = *biases.choose(rng).unwrap();

        // eprintln!("draw_elo = {:?}", draw_elo);

        let ldw1 = ldw(belo, draw_elo, bias);
        let ldw2 = ldw(belo, draw_elo, -bias);

        // eprintln!("ldw1 = {:?}", ldw1);
        // eprintln!("ldw2 = {:?}", ldw2);

        let i = pentanomial_pick(&ldw1, rng);
        let j = pentanomial_pick(&ldw2, rng);

        // eprintln!("i = {:?}", i);
        // eprintln!("j = {:?}", j);

        PentaWDL::from_int(i + j)
        // unimplemented!()
    }

    fn pentanomial_pick(probs: &[f64], rng: &mut StdRng) -> usize {
        let x = rng.gen_range(0.0..1.0);

        let mut p = 0.0;

        for i in 0..probs.len() {
            let pp = probs[i];
            p += pp;
            if p >= x {
                return i;
            }
        }

        panic!("pentanomial_pick");
    }

    fn ldw(belo: f64, draw_elo: f64, bias: f64) -> Vec<f64> {
        let w = l(belo - draw_elo + bias);
        let l = l(-belo - draw_elo - bias);
        let d = 1. - w - l;
        vec![l, d, w]
    }

    fn l(x: f64) -> f64 {
        let bb = f64::ln(10.) / 400.;
        if x >= 0. {
            1. / (1. + f64::exp(-bb * x))
        } else {
            let e = f64::exp(bb * x);
            e / (e + 1.)
        }
    }

    fn elo_to_belo(elo: f64) -> f64 {
        use argmin::solver::brent::Brent;
        use argmin::core::{ArgminOp, ArgminSlogLogger, Error, Executor, ObserverMode};

        struct BrentFunc {
            // s:      f64,
            // pdf:    Vec<(f64,f64)>,
        }

        impl ArgminOp for BrentFunc {
            type Param    = f64;
            type Output   = f64;
            type Hessian  = ();
            type Jacobian = ();
            type Float    = f64;

            fn apply(&self, x: &Self::Param) -> Result<Self::Output, Error> {
                // Ok(self.pdf.iter().map(|(a,p)| p * (a - self.s) / (1. + x * (a - self.s))).sum::<f64>())
                unimplemented!()
            }
        }


        let s = l(elo);

        // let b = BrentFunc { s, pdf: pdf.to_vec() };

        // let solver = Brent::new(u - eps, l + eps, 2e-12);

        // let res = Executor::new(b, solver, 0.0)
        // // .add_observer(ArgminSlogLogger::term(), ObserverMode::Always)
        //     .max_iters(100)
        //     .run()
        //     .unwrap();
        // let x = res.state.best_param;

        unimplemented!()
    }

    fn _probs(belo: f64, draw_elo: f64, biases: [f64; 2]) -> (Vec<f64>,Vec<f64>) {

        let mut ps3 = vec![];
        let mut ps5 = vec![];

        for bias in biases.iter() {
            let (p3,p5) = probs_(belo, draw_elo, *bias);

            ps3.push(p3);
            ps5.push(p5);

        }

        unimplemented!()
    }

    // fn add(xs: &[f64]) -> Vec<f64> {
    //     let l = xs.len();
    // }

    fn avg(xs: &[f64]) -> Vec<f64> {
        // let l = 
        unimplemented!()
    }

    fn probs_(belo: f64, draw_elo: f64, bias: f64) -> (f64,f64) {
        unimplemented!()
    }

}

