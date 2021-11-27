
use crate::brain::trainer::TDOutcome;
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
use crate::qsearch::*;

use derive_new::new;

#[derive(Debug,PartialEq,Clone,new)]
pub struct TxPosition {
    pub game:     Game,
    pub result:   TDOutcome,
    // pub q_score:  Score,
}



pub fn texel_optimize() {
}


pub fn average_eval_error(
    ts:         &Tables,
    ev_mid:     &EvalParams,
    ev_end:     &EvalParams,
    inputs:     Vec<TxPosition>,
    k:          f64,
) -> f64 {

    // const K: f64 = 1.0;

    fn sigmoid(s: f64, k: f64) -> f64 {
        1.0 / (1.0 + 10.0f64.powf(-k * s / 400.0))
    }

    let sum: f64 = inputs.iter().map(|pos| {
        let r = match pos.result {
            TDOutcome::Win(White) => 1.0,
            TDOutcome::Win(Black) => 0.0,
            TDOutcome::Draw       => 0.5,
            TDOutcome::Stalemate  => 0.5,
        };
        let q_score = qsearch_once(&ts, &pos.game, pos.game.state.side_to_move, &ev_mid, &ev_end);
        (r - sigmoid(q_score as f64, k)).powi(2)
    }).sum();
    sum / inputs.len() as f64

    // unimplemented!()
}











