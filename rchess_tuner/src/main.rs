#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]

#![allow(clippy::all)]

mod sprt;
mod tuner_types;
mod parsing;
mod supervisor;
mod json_config;

use self::json_config::*;

use rchess_engine_lib::alphabeta::ABResult;
use rchess_engine_lib::types::*;
use rchess_engine_lib::tables::*;
use rchess_engine_lib::explore::*;
use rchess_engine_lib::evaluate::*;
use tuner_types::InputParser;
use tuner_types::Match;

// use sprt::*;

use std::io::BufReader;
use std::io::{self,Write,BufRead,Stdout,Stdin};
use std::process::{Command,Stdio};

use itertools::Itertools;

use chrono::{Datelike,Timelike};
use log::{debug, error, log_enabled, info, Level};
use simplelog::*;
use gag::Redirect;

use crate::tuner_types::MatchResult;

fn main() {
    // json_test();
}

fn main2() {
// fn main() {

    // let lines = vec![
    //     "Started game 1 of 100 (rchess vs rchess_prev)",
    //     "Finished game 1 (rchess vs rchess_prev): 0-1 {White loses on time}",
    //     "Score of rchess vs rchess_prev: 0 - 1 - 0  [0.000] 1",
    //     "...      rchess playing White: 0 - 1 - 0  [0.000] 1",
    //     "...      White vs Black: 0 - 1 - 0  [0.000] 1",
    //     "Elo difference: -inf +/- nan, LOS: 15.9 %, DrawRatio: 0.0 %",
    // ];

    let lines0 = vec![
        "Started game 3 of 100 (rchess vs rchess_prev)",
        "Finished game 3 (rchess vs rchess_prev): 0-1 {White loses on time}",
        "Score of rchess vs rchess_prev: 2 - 1 - 0  [0.667] 3",
        "...      rchess playing White: 1 - 1 - 0  [0.500] 2",
        "...      rchess playing Black: 1 - 0 - 0  [1.000] 1",
        "...      White vs Black: 1 - 2 - 0  [0.333] 3",
        "Elo difference: -inf +/- nan, LOS: 15.9 %, DrawRatio: 0.0 %",
    ];

    let lines1 = vec![
        "Started game 3 of 100 (rchess vs rchess_prev)",
        "Finished game 3 (rchess vs rchess_prev): 0-1 {White loses on time}",
        "Score of rchess vs rchess_prev: 2 - 1 - 0  [0.667] 3",
        "...      rchess playing White: 1 - 1 - 0  [0.500] 2",
        "...      rchess playing Black: 1 - 0 - 0  [1.000] 1",
        "...      White vs Black: 1 - 2 - 0  [0.333] 3",
        "Elo difference: 120.4 +/- 123.5, LOS: 71.8 %, DrawRatio: 0.0 %",
    ];

    let res0 = Match::parse(lines0.into_iter().map(|s| s.to_owned()).collect());
    eprintln!("res0 = {:?}", res0);

    let res1 = Match::parse(lines1.into_iter().map(|s| s.to_owned()).collect());
    eprintln!("res1 = {:?}", res1);

}

// fn main() {
fn main3() {

    init_logger();

    let now = chrono::Local::now();
    let timestamp = format!("{:0>4}-{:0>2}-{:0>2}_{:0>2}:{:0>2}:{:0>2}",
        now.year(), now.month(), now.day(),
        now.hour(), now.minute(), now.second());

    let output_label = "";

    let output_file = &format!("tuning_logs/out_{}_{}.pgn", output_label, timestamp);
    let log_file    = &format!("tuning_logs/log_{}_{}.pgn", output_label, timestamp);

    let engine2 = "rchess_prev";

    let num_games = 50;

    let time = "tc=1+0.1";
    // let time = "tc=0.5+0.05";
    // let time = "st=0.05";

    let (elo0, elo1) = (0, 50);

    let args = [
        "-tournament gauntlet",
        "-concurrency 1",
        &format!("-pgnout {}", output_file),
        &format!("-engine conf=rchess {} timemargin=50 restart=off", time),
        &format!("-engine conf={} {} timemargin=50 restart=off", engine2, time),
        "-each proto=uci",
        "-openings file=tables/openings-10ply-100k.pgn policy=round",
        "-tb tables/syzygy/",
        "-tbpieces 5",
        "-repeat",
        &format!("-rounds {}", num_games),
        "-games 2",
        "-draw movenumber=40 movecount=4 score=8",
        "-resign movecount=4 score=500",
        "-ratinginterval 1",
        &format!("-sprt elo0={} elo1={} alpha=0.05 beta=0.05",
                 elo0, elo1),
    ];

    let args = args.into_iter()
        .map(|arg| {
            arg.split_ascii_whitespace()
        })
        .flatten()
        .collect::<Vec<_>>();

    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panicinfo| {
        let loc = panicinfo.location();
        let mut file = std::fs::File::create("/home/me/code/rust/rchess/panic.log").unwrap();
        let s = format!("Panicking, Location: {:?}", loc);
        file.write(s.as_bytes()).unwrap();

        // cutechess doesn't kill child processes when parent panics
        let child = Command::new("pkill")
            .args(["rchess_uci"])
            .spawn()
            .unwrap();
        hook(panicinfo)
    }));

    // let mut output = Stdio::piped();
    let mut child = Command::new("cutechess-cli")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        // .stdout(output)
        .spawn()
        .expect("failed to spawn cutechess-cli");

    let reader = BufReader::new(child.stdout.unwrap());

    let mut game: Vec<String>   = vec![];
    let mut matches: Vec<Match> = vec![];

    let mut state = InputParser::None;

    for line in reader.lines() {
        let line = line.unwrap();

        match state {
            InputParser::None => {
                game.push(line);
                state = InputParser::Started;
            },
            InputParser::Started => {
                game.push(line.clone());

                if line.starts_with("Elo difference") {
                    let v = std::mem::replace(&mut game, vec![]);

                    // for line in v {
                    //     eprintln!("{:?}", line);
                    // }

                    let res = Match::parse(v).unwrap();

                    eprintln!("{:?}", res);

                    matches.push(res);

                    state = InputParser::Started;
                }
            },
        }

    }

    for m in matches.iter() {
        eprintln!("m = {:?}", m);
    }

}

fn init_logger() {
    let now = chrono::Local::now();
    let mut logpath = format!(
        "/home/me/code/rust/rchess/logs/log_tuner_{:0>4}-{:0>2}-{:0>2}_{:0>2}:{:0>2}:{:0>2}-1.log",
        now.year(), now.month(), now.day(),
        now.hour(), now.minute(), now.second());
    if std::path::Path::new(&logpath).exists() {
        logpath = format!(
            "/home/me/code/rust/rchess/logs/log_tuner_{:0>4}-{:0>2}-{:0>2}_{:0>2}:{:0>2}:{:0>2}-2.log",
            now.year(), now.month(), now.day(),
            now.hour(), now.minute(), now.second());
    };
    let mut logfile = std::fs::OpenOptions::new()
        .truncate(true)
        .read(true)
        .create(true)
        .write(true)
        .open(logpath)
        .unwrap();

    let cfg = ConfigBuilder::new()
        .set_time_level(LevelFilter::Off)
        .set_target_level(LevelFilter::Off)
        // .set_thread_level(LevelFilter::Info)
        .set_thread_level(LevelFilter::Off)
        .set_location_level(LevelFilter::Off)
        .build();

    WriteLogger::init(LevelFilter::Debug, cfg, logfile).unwrap();
    // WriteLogger::init(LevelFilter::Trace, cfg, logfile).unwrap();

    let mut errfile = std::fs::File::create("/home/me/code/rust/rchess/panic.log").unwrap();
    let err_redirect = Redirect::stderr(errfile).unwrap();
}

