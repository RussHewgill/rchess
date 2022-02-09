
use crate::json_config::Engine;


#[derive(Debug,Clone)]
pub struct Supervisor {
    engine:   Engine,

    tunable:  Tunable,

}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Clone)]
pub struct Tunable {
    name:    String,
    min:     u64,
    max:     u64,
    start:   u64,
    step:    u64,
}

pub fn run_cutechess(
    engine1:          Engine,
    engine2:          Engine,
    output_label:     &str,
    num_games:        u64,
    (elo0,elo1):      (u64,u64),
    confidence:       f64,
) {

    let output_file = &format!("tuning_logs/out_{}_{}.pgn", output_label, timestamp);
    let log_file    = &format!("tuning_logs/log_{}_{}.pgn", output_label, timestamp);

}

