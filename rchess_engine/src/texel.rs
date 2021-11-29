
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


pub fn texel_optimize(
    ts:                          &Tables,
    (mut ev_mid,mut ev_end):     (EvalParams,EvalParams),
    inputs:                      Vec<TxPosition>,
    exhelper:                    &ExHelper,
    ignore_weights:              Vec<bool>,
) {
    let mut best_error = average_eval_error(ts, &ev_mid, &ev_end, inputs, exhelper, None);

    loop {

        let (arr_mid, arr_end) = (ev_mid.to_arr(), ev_end.to_arr());
        let mut weights = vec![];
        weights.extend_from_slice(&arr_mid);
        weights.extend_from_slice(&arr_end);

        // for (n,w) in weights.iter().enumerate() {
        //     if ignore_weights[n] {
        //         continue;
        //     }
        // }

        break;
    }

}

pub fn texel_optimize_once(
    ts:                          &Tables,
    (mut ev_mid,mut ev_end):     (EvalParams,EvalParams),
    inputs:                      Vec<TxPosition>,
    exhelper:                    &ExHelper,
) -> (EvalParams,EvalParams) {
    unimplemented!()
}


pub fn average_eval_error(
    ts:         &Tables,
    ev_mid:     &EvalParams,
    ev_end:     &EvalParams,
    inputs:     Vec<TxPosition>,
    // ph_rw:      Option<&PHTable>,
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
        let q_score = exhelper.qsearch(ts, &pos.game, (0,0), (alpha, beta), &mut stats);
        (r - sigmoid(q_score as f64, k)).powi(2)
    }).sum();
    sum / inputs.len() as f64

    // unimplemented!()
}











