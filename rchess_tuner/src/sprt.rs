


pub fn log_likelyhood(x: f64) -> f64 {
    1.0 / (1.0 + 10.0f64.powf(-x / 400.0))
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





