
use crate::brain::trainer::TDOutcome;
use crate::explore::ExHelper;
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
use crate::qsearch::*;
use crate::searchstats::*;
use crate::pawn_hash_table::*;

use derive_new::new;

#[derive(Debug,PartialEq,Clone,new)]
pub struct TxPosition {
    pub game:     Game,
    pub result:   TDOutcome,
    // pub q_score:  Score,
}

#[cfg(feature = "nope")]
pub fn texel_optimize(
    ts:                          &Tables,
    inputs:                      &[TxPosition],
    mut exhelper:                &mut ExHelper,
    ignore_weights:              Vec<bool>,
    count:                       Option<usize>
) -> (EvalParams,EvalParams) {
    let mut best_error = average_eval_error(ts, &inputs, &exhelper, None);

    let (mut arr_mid, mut arr_end) =
        (exhelper.cfg.eval_params_mid.to_arr(),exhelper.cfg.eval_params_end.to_arr());

    // eprintln!("arr_mid.len() = {:?}", arr_mid.len());
    let t0 = std::time::Instant::now();
    let mut loops = 0;
    loop {

        for n in 0..arr_mid.len() {
            if let Some(true) = ignore_weights.get(n) {
                continue;
            }

            arr_mid[n] = arr_mid[n].checked_add(1).unwrap();
            exhelper.cfg.eval_params_mid = EvalParams::from_arr(&arr_mid);
            exhelper.ph_rw.purge_scores();

            let new_error = average_eval_error(ts, &inputs, &exhelper, None);

            if new_error < best_error {
                best_error = new_error;
            } else {
                arr_mid[n] = arr_mid[n].checked_sub(2).unwrap();
                exhelper.cfg.eval_params_mid = EvalParams::from_arr(&arr_mid);
                exhelper.ph_rw.purge_scores();
                let new_error = average_eval_error(ts, &inputs, &exhelper, None);

                if new_error < best_error {
                    best_error = new_error;
                } else {
                    arr_mid[n] = arr_mid[n].checked_add(1).unwrap();
                    exhelper.cfg.eval_params_mid = EvalParams::from_arr(&arr_mid);
                    exhelper.ph_rw.purge_scores();
                }
            }

            // let t1 = t0.elapsed().as_secs_f64();
            // println!("n {} in {:.3} seconds, {:.1}", n, t1, n as f64 / t1);
        }

        loops += 1;
        let t1 = t0.elapsed().as_secs_f64();
        eprintln!("loops = {:>3}, best_error = {:.3}, time: {:.1}s, {:.1} weights/s",
                  loops, best_error, t1, arr_mid.len() as f64 / t1,
        );
        if let Some(c) = count { if loops >= c { break; } }
    }

    // (ev_mid,ev_end)
    (exhelper.cfg.eval_params_mid,exhelper.cfg.eval_params_end)
}

pub fn find_k(
    ts:         &Tables,
    inputs:     &[TxPosition],
    exhelper:   &ExHelper,
) -> f64 {
    // let mut k = 0.0;
    // let mut best = average_eval_error(&ts, inputs, exhelper, Some(k));
    unimplemented!()
}

pub fn average_eval_error(
    ts:         &Tables,
    inputs:     &[TxPosition],
    exhelper:   &ExHelper,
    k:          Option<f64>,
) -> f64 {
    const K: f64 = 1.0;
    let k = k.unwrap_or(K);

    // const K: f64 = 1.0;

    fn sigmoid(s: f64, k: f64) -> f64 {
        1.0 / (1.0 + 10.0f64.powf(-k * s / 400.0))
    }

    let (alpha,beta) = (i32::MIN,i32::MAX);
    let (alpha,beta) = (alpha + 200,beta - 200);
    let mut stats = SearchStats::default();

    let sum: f64 = inputs.iter().map(|pos| {
        let r = match pos.result {
            TDOutcome::Win(White) => 1.0,
            TDOutcome::Win(Black) => 0.0,
            TDOutcome::Draw       => 0.5,
            TDOutcome::Stalemate  => 0.5,
        };
        // let q_score = qsearch_once(&ts, &pos.game, pos.game.state.side_to_move, &ev_mid, &ev_end, ph_rw);
        // exhelper.game = pos.game.clone();
        // exhelper.side = pos.game.state.side_to_move;

        // let q_score = exhelper.qsearch(ts, &pos.game, (0,0), (alpha, beta), &mut stats);
        // let q_score = pos.game.state.side_to_move.fold(q_score, -q_score);

        let score = pos.game.sum_evaluate(
            ts, &exhelper.cfg.eval_params_mid, &exhelper.cfg.eval_params_end,
            Some(&exhelper.ph_rw),
            // None,
        );

        // assert_eq!(score, q_score);

        (r - sigmoid(score as f64, k)).powi(2)
    }).sum();
    sum / inputs.len() as f64

    // unimplemented!()
}











