
use crate::json_config::Engine;

pub use log::{info,warn,debug,trace};

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

#[derive(Debug,Default,PartialEq,PartialOrd,Clone,Copy)]
pub struct RunningTotal {
    pub ll:     u32,
    pub ld_dl:  u32,
    pub lw_dd:  u32,
    pub dw_wd:  u32,
    pub ww:     u32,
}

impl RunningTotal {
    pub fn to_vec(&self) -> Vec<u32> {
        vec![self.ll,self.ld_dl,self.lw_dd,self.dw_wd,self.ww]
    }
}

#[derive(Clone,Copy)]
pub enum MatchOutcome {
    Match(Match),
    // MatchPair(Match, Match),
    SPRTFinished(Match, Elo, SPRTResult),
}

#[derive(Clone,Copy)]
pub struct Match {
    pub engine_a:   Color,
    pub game_num:   u32,
    pub result:     MatchResult,
    pub sum_score:  (u32,u32,u32),
    pub elo:        Option<Elo>,
    pub sprt:       Option<SPRTResult>,
}

#[derive(Clone,Copy)]
/// LOS: likelihood of superiority
pub struct Elo {
    pub elo:        f64,
    pub bounds:     f64,
    pub los:        f64,
    pub draw_ratio: f64,
}

#[derive(Clone,Copy)]
/// LLR: log-likelihood ratio
pub struct SPRTResult {
    pub llr:           f64,
    pub llr_pct:       f64,
    pub lbound:        f64,
    pub ubound:        f64,
    pub hyp_accepted:  Option<Hypothesis>,
}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Clone,Copy)]
/// Hypothesis H1:        is that A is stronger than B by at least ELO0 ELO points
/// H0 (null hypothesis): is that A is NOT stronger than B by at least ELO1 ELO points
pub enum Hypothesis {
    H0,
    H1,
}

/// parse
impl MatchOutcome {
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
                r"(-?\d+\.\d+|-?inf) \+.- (nan|\d+\.\d+), LOS: (nan|\d+\.\d+) %, DrawRatio: (\d+\.\d+) %"
            ).unwrap()
        });

        let last = input.last().unwrap();
        let (line_elo,line_sprt,finished): (&str,Option<&str>,bool) = if last.starts_with("Finished match") {
            (&input[input.len() - 3],Some(&input[input.len() - 2]),true)
        } else if last.starts_with("SPRT") {
            (&input[input.len() - 2],Some(&last),false)
        } else {
            (&last,None,false)
        };

        let elo_diff = RE2.captures(line_elo).unwrap();

        let elo        = f64::from_str(elo_diff.get(1)?.as_str()).ok();
        let bounds     = f64::from_str(elo_diff.get(2)?.as_str()).ok();
        let los        = f64::from_str(elo_diff.get(3)?.as_str()).ok();
        let draw_ratio = f64::from_str(elo_diff.get(4)?.as_str()).ok()?;

        let elo = match (elo,bounds,los) {
            (Some(elo),Some(bounds),Some(los)) => Some(Elo {
                elo,
                bounds,
                los,
                draw_ratio,
            }),
            _ => None,
        };

        static RE3: Lazy<Regex> = Lazy::new(|| {
            Regex::new(
                r"(-?\d+\.\d+) \((-?\d+\.\d+).+ (-?\d+\.\d+).+ (-?\d+\.\d+)(?: - H(\d) was accepted)?"
            ).unwrap()
        });

        let sprt = if let Some(line_sprt) = line_sprt {
            let sprt = RE3.captures(line_sprt).unwrap();

            let hyp_accepted = if let Some(h) = sprt.get(5) {
                let x = i32::from_str(h.as_str()).unwrap();
                match x {
                    0 => Some(Hypothesis::H0),
                    1 => Some(Hypothesis::H1),
                    _ => panic!("bad hypothesis?"),
                }
            } else { None };

            Some(SPRTResult {
                llr:      f64::from_str(sprt.get(1)?.as_str()).ok()?,
                llr_pct:  f64::from_str(sprt.get(2)?.as_str()).ok()?,
                lbound:   f64::from_str(sprt.get(3)?.as_str()).ok()?,
                ubound:   f64::from_str(sprt.get(4)?.as_str()).ok()?,
                hyp_accepted,
            })
        } else { None };

        // eprintln!("elo = {:?}", elo);

        let engine_a = if game_num % 2 == 0 { Color::Black } else { Color:: White };

        let m = Match {
            engine_a,
            game_num,
            result,
            sum_score:  (w,b,d),
            elo,
            sprt,
        };

        if finished {
            Some(MatchOutcome::SPRTFinished(m, elo.unwrap(), sprt.unwrap()))
        } else {
            Some(MatchOutcome::Match(m))
        }

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


impl std::fmt::Debug for MatchOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &MatchOutcome::Match(m)            => {
                f.write_str(&format!("Match: {:?}", m))?;
            },
            // &MatchOutcome::MatchPair(m1,m2)            => {
            //     f.write_str(&format!("{:?}\n{:?}", m1, m2))?;
            // },
            &MatchOutcome::SPRTFinished(m,_,_) => {
                f.write_str(&format!("SPRTFinished: {:?}", m))?;
            },
        }
        Ok(())
    }
}

impl std::fmt::Debug for Match {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Game {}:\n", self.game_num))?;
        f.write_str(&format!("    result: {:?}\n", self.result))?;
        f.write_str(&format!("    scores: {:>3} - {:>3} - {:>3}\n",
                             self.sum_score.0, self.sum_score.1, self.sum_score.2))?;
        f.write_str(&format!("    {:?}\n", self.elo))?;
        f.write_str(&format!("    {:?}", self.sprt))?;

        Ok(())
    }
}

impl std::fmt::Debug for Elo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Elo: {:.1} +/- {:.1}, LOS: {:.1}%",
                             self.elo, self.bounds, self.los))?;
        Ok(())
    }
}

impl std::fmt::Debug for SPRTResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("LLR: {:.2}, ({:.1}%) l[{:.2},{:.2}]u",
                             self.llr, self.llr_pct, self.lbound, self.ubound,
        ))?;
        if let Some(h) = self.hyp_accepted {
            f.write_str(&format!(" - {:?} was accepted", h))?;
        }
        Ok(())
    }
}


