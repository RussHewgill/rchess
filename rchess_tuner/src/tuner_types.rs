
use std::io::{self};
use std::str::FromStr;

// type Out      = io::Result<String>;
// type MatchOut = (Out,Out,Out,Out,Out,Out);

#[derive(Debug,Clone,Copy)]
pub struct Match {
    game_num:   u32,
    result:     MatchResult,
    sum_score:  (u32,u32,u32),
    elo_diff:   (f64,f64),
    los:        f64,
    draw_ratio: f64,
}

// "Started game 1 of 100 (rchess vs rchess_prev)",
// "Finished game 1 (rchess vs rchess_prev): 0-1 {White loses on time}",
// "Score of rchess vs rchess_prev: 0 - 1 - 0  [0.000] 1",
// "...      rchess playing White: 0 - 1 - 0  [0.000] 1",
// "...      White vs Black: 0 - 1 - 0  [0.000] 1",
// "Elo difference: -inf +/- nan, LOS: 15.9 %, DrawRatio: 0.0 %",

impl Match {
    pub fn parse(input: Vec<String>) -> Option<Self> {
        use regex::Regex;

        let reg0 = Regex::new(r"(\d+)").unwrap();
        let game_num = u32::from_str(reg0.find(&input[0])?.as_str()).ok()?;

        let reg1 = Regex::new(r"(\d+) - (\d+) - (\d+)").unwrap();
        // let scores = reg1.find_iter(&input[2]);
        let scores = reg1.captures(&input[2]).unwrap();

        let w = u32::from_str(scores.get(1)?.as_str()).ok()?;
        let b = u32::from_str(scores.get(2)?.as_str()).ok()?;
        let d = u32::from_str(scores.get(3)?.as_str()).ok()?;

        eprintln!("(w,b,d) = {:?}", (w,b,d));

        // for s in scores {
        //     eprintln!("s = {:?}", s);
        // }

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




