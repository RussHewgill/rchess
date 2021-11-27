
use crate::brain::trainer::TDOutcome;
use crate::types::*;
use crate::tables::*;


#[derive(Debug,PartialEq,Clone)]
pub struct TxPosition {
    pub game:    Game,
    pub result:  TDOutcome,
}



pub fn texel_optimize() {
}


pub fn average_eval_error(inputs: Vec<TxPosition>, k: f64) -> f64 {

    fn sigmoid(s: f64, k: f64) -> f64 {
        1.0 / (1.0 + 10.0f64.powf(-k * s / 400.0))
    }

    // let sum = inputs.iter().map(|pos| {
    //     let r = match pos.result {
    //         TDOutcome::Win(White) => 1.0,
    //         TDOutcome::Win(Black) => 0.0,
    //         TDOutcome::Draw       => 0.5,
    //         TDOutcome::Stalemate  => 0.5,
    //     };
    //     let s = unimplemented!();
    //     (r - sigmoid(s, k)).powi(2)
    // }).sum();
    // sum / inputs.len() as f64

    unimplemented!()
}











