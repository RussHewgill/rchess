
use std::io::{self};

// type Out      = io::Result<String>;
// type MatchOut = (Out,Out,Out,Out,Out,Out);

#[derive(Debug,Clone,Copy)]
pub struct Match {
    game_num:   usize,
    result:     MatchResult,
    sum_score:  (u32,u32,u32),
    elo_diff:   (f64,f64),
    los:        f64,
    draw_ratio: f64,
}

impl Match {
    pub fn parse(input: Vec<String>) -> Self {
        unimplemented!()
    }
}

#[derive(Debug,Clone,Copy)]
pub enum MatchResult {
    Win(WinType),
    Draw(DrawType),
    Loss(LossType,)
}

#[derive(Debug,Clone,Copy)]
pub enum WinType {
}

#[derive(Debug,Clone,Copy)]
pub enum DrawType {
}

#[derive(Debug,Clone,Copy)]
pub enum LossType {
}




