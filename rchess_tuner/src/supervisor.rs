


#[derive(Debug,Clone)]
pub struct Supervisor {

}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Clone,Copy)]
pub struct Tunable<T> {
    min:     T,
    max:     T,
    start:   T,
    step:    T,
}



