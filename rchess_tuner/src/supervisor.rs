
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



