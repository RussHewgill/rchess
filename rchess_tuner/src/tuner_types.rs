
use crate::json_config::Engine;

use std::io::{self};
use std::process::Child;
use std::str::FromStr;
use std::sync::Arc;

use crossbeam::channel::Receiver;
use once_cell::sync::Lazy;
use rchess_engine_lib::explore::AtomicBool;
use rchess_engine_lib::types::Color;

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Clone,Copy)]
pub enum TimeControl {
    Increment(u64),
    TimePlusInc(u64,u64),
}

impl TimeControl {

    pub fn new_f64(t: f64, inc: f64) -> Self {
        Self::TimePlusInc((t * 1000.0) as u64, (inc * 1000.0) as u64)
    }

    pub fn print(self) -> String {
        match self {
            Self::Increment(inc)     => format!("st={:.3}", inc as f64 / 1000.0),
            Self::TimePlusInc(t,inc) => format!("tc={:.3}+{:.3}", t as f64 / 1000.0, inc as f64 / 1000.0),
        }
    }
}

#[derive(Debug,PartialEq,PartialOrd,Clone,Copy)]
pub enum InputParser {
    None,
    Started,
}

#[derive(Clone,Copy)]
pub struct Match {
    game_num:   u32,
    result:     MatchResult,
    sum_score:  (u32,u32,u32),
    elo_diff:   (f64,f64),
    los:        f64,
    draw_ratio: f64,
}

impl Match {
    pub fn parse(input: Vec<String>) -> Option<Self> {
        use regex::Regex;

        static RE0: Lazy<Regex> = Lazy::new(|| { Regex::new(
            r"Finished game (\d+).+\{([^}]+)\}"
        ).unwrap() });

        let res = RE0.captures(&input[1])?;

        let game_num = u32::from_str(res.get(1)?.as_str()).ok()?;
        let result   = MatchResult::parse(res.get(2)?.as_str())?;

        static RE1: Lazy<Regex> = Lazy::new(|| { Regex::new(r"(\d+) - (\d+) - (\d+)").unwrap() });
        let scores = RE1.captures(&input[2]).unwrap();

        let w = u32::from_str(scores.get(1)?.as_str()).ok()?;
        let b = u32::from_str(scores.get(2)?.as_str()).ok()?;
        let d = u32::from_str(scores.get(3)?.as_str()).ok()?;

        // eprintln!("(w,b,d) = {:?}", (w,b,d));

        static RE2: Lazy<Regex> = Lazy::new(|| {
            Regex::new(
                r"(-?\d+\.\d+|-?inf) \+.- (nan|\d+\.\d+), LOS: (\d+\.\d+) %, DrawRatio: (\d+\.\d+) %"
            ).unwrap()
        });

        let elo_diff = RE2.captures(&input.last().unwrap()).unwrap();

        let elo        = f64::from_str(elo_diff.get(1)?.as_str()).ok()?;
        let bounds     = f64::from_str(elo_diff.get(2)?.as_str()).ok()?;
        let los        = f64::from_str(elo_diff.get(3)?.as_str()).ok()?;
        let draw_ratio = f64::from_str(elo_diff.get(4)?.as_str()).ok()?;

        // eprintln!("elo = {:?}", elo);

        Some(Self {
            game_num,
            result,
            sum_score:  (w,b,d),
            elo_diff:   (elo,bounds),
            los,
            draw_ratio,
        })

        // unimplemented!()
        // None
    }
}

#[derive(Debug,Clone,Copy)]
pub enum MatchResult {
    WinLoss(Color, WinLossType),
    Draw(DrawType),
}

#[derive(Debug,Clone,Copy)]
pub enum WinLossType {
    Resign,
    Time,
    AdjudicationSyzygy,
    Adjudication,
    IllegalMove,
    Disconnect,
    Stalled,
    Agreement,
    Checkmate,
}

#[derive(Debug,Clone,Copy)]
pub enum DrawType {
    Timeout,
    Adjudication,
    Stalemate,
    Repetition,
    FiftyMoveRule,
}


impl std::fmt::Debug for Match {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Game {}:\n", self.game_num))?;
        f.write_str(&format!("    result: {:?}\n", self.result))?;
        f.write_str(&format!("    scores: {:>3} - {:>3} - {:>3}",
                             self.sum_score.0, self.sum_score.1, self.sum_score.2))?;

        Ok(())
    }
}



