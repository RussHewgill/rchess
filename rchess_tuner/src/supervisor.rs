
use crate::json_config::Engine;


#[derive(Debug,Clone)]
pub struct Supervisor {
    engine:   Engine,

    // tunable:  Tunable<>

}



#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Clone,Copy)]
pub struct Tunable<T> {
    min:     T,
    max:     T,
    start:   T,
    step:    T,
}



