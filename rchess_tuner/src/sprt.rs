



/// https://github.com/glinscott/fishtest/blob/master/server/fishtest/stats/LLRcalc.py
pub mod gsprt {

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

    // fn draw_elo_calc()

    pub fn sprt_elo((win,draw,loss): (u32,u32,u32), elo0: f64, elo1: f64, alpha: f64) -> f64 {

        unimplemented!()
    }

}

/// expected score = prob(win) + 0.5 * prob(draw)
pub fn log_likelyhood(elo_diff: f64) -> f64 {
    1.0 / (1.0 + 10.0f64.powf(-elo_diff / 400.0))
}

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





