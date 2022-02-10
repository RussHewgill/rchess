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
mod optimizer;
mod gamerunner;

use self::json_config::*;

use once_cell::sync::Lazy;
use rchess_engine_lib::alphabeta::ABResult;
use rchess_engine_lib::types::*;
use rchess_engine_lib::tables::*;
use rchess_engine_lib::explore::*;
use rchess_engine_lib::evaluate::*;
use regex::Regex;
use serde_json::json;
use supervisor::Tunable;
use tuner_types::*;

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
use crate::supervisor::*;

// fn main() {
fn main3() {

    let lines = vec![
        "Started game 3 of 100 (rchess vs rchess_prev)".to_string(),
        "Finished game 15 (rchess vs gnuchess): 1/2-1/2 {Draw by adjudication: SyzygyTB}".to_string(),
        "Score of rchess vs gnuchess: 13 - 1 - 1  [0.900] 15".to_string(),
        "...      rchess playing White: 6 - 1 - 1  [0.813] 8".to_string(),
        "...      rchess playing Black: 7 - 0 - 0  [1.000] 7".to_string(),
        "...      White vs Black: 6 - 8 - 1  [0.433] 15".to_string(),
        "Elo difference: 381.7 +/- nan, LOS: 99.9 %, DrawRatio: 6.7 %".to_string(),
        "SPRT: llr 6.42 (218.1%), lbound -2.94, ubound 2.94 - H1 was accepted".to_string(),
        "Finished match".to_string(),
    ];

    let res = MatchOutcome::parse(lines);
    eprintln!("res = {:?}", res);

}

// pub fn json_test() {
//     let path = "engines-test.json";
//     let engines = Engine::read_all_from_file(&path).unwrap();
//     for eng in engines.iter() {
//         eprintln!("eng = {:?}", eng);
//     }
//     let mut eng = engines.get("rchess").unwrap().clone();
//     eng.options.insert("wat2".to_string(), json!(22));
//     eng.write(&path).unwrap();
// }

fn main() {
    use crate::sprt::*;
    use crate::sprt::gsprt::*;

    let (s0,s1) = (50.0,0.0);

    // let results = vec![
    //     ((3,2,1), (58.5, 349.5), (0.0991)),
    //     ((3,3,1), (0.0, 291.2), (-0.0843)),
    // ];

    // let pdf0 = results_to_pdf(results[0].0);
    // let pdf1 = results_to_pdf(results[1].0);

    let pdf0 = results_to_pdf((1,0,0));
    let pdf1 = results_to_pdf((1000,20,10));

    eprintln!("pdf0 = {:?}", pdf0);
    eprintln!("pdf1 = {:?}", pdf1);

    let llr0 = llr_alt(&pdf0.1, s0, s1);
    let llr1 = llr_alt(&pdf1.1, s0, s1);

    eprintln!();
    eprintln!("llr0 = {:?}", llr0);
    eprintln!("llr1 = {:?}", llr1);

}

// fn main() {
fn main4() {

    let engine = Engine::read_from_file("rchess", "engines-test.json").unwrap();

    // let timecontrol = TimeControl::new_f64(1.0, 0.1);
    let timecontrol = TimeControl::new_f64(0.2, 0.025);

    let tunable = Tunable::new("lmr_reduction".to_string(), 2, 5, 3, 1);

    let mut sup = Supervisor {
        engine_baseline: engine.clone(),
        engine_tuning:   engine.clone(),
        tunable,
        timecontrol,
    };

    sup.find_optimum();

}

// fn main() {
fn main2() {

    // init_logger();

    let engine1 = "rchess";
    let engine2 = "rchess_prev";

    let timecontrol = TimeControl::new_f64(0.5, 0.05);
    // let timecontrol = TimeControl::new_f64(1.0, 0.1);
    let output_label = "test";
    let (elo0,elo1) = (0,50);
    let num_games = 50;

    // let hook = std::panic::take_hook();
    // std::panic::set_hook(Box::new(move |panicinfo| {
    //     let loc = panicinfo.location();
    //     let mut file = std::fs::File::create("/home/me/code/rust/rchess/panic.log").unwrap();
    //     let s = format!("Panicking, Location: {:?}", loc);
    //     file.write(s.as_bytes()).unwrap();
    //     // cutechess doesn't kill child processes when parent panics
    //     let child = Command::new("pkill")
    //         .args(["rchess_uci"])
    //         .spawn()
    //         .unwrap();
    //     hook(panicinfo)
    // }));

    let cutechess = CuteChess::run_cutechess_tournament(
        engine1,
        engine2,
        timecontrol,
        output_label,
        num_games,
        (elo0,elo1),
        0.05);

    loop {
        match cutechess.rx.recv() {
            Ok(m)  => eprintln!("m = {:?}", m),
            Err(e) => {
                println!("recv err = {:?}", e);
                break;
            },
        }
    }

    // let child = Command::new("pkill")
    //     .args(["rchess_uci"])
    //     .spawn()
    //     .unwrap();

}

// fn main() {
#[cfg(feature = "nope")]
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

    // WriteLogger::init(LevelFilter::Debug, cfg, logfile).unwrap();
    // // WriteLogger::init(LevelFilter::Trace, cfg, logfile).unwrap();

    let log1 = TermLogger::new(LevelFilter::Debug, cfg.clone(), TerminalMode::Stderr, ColorChoice::Auto);

    CombinedLogger::init(vec![
        // log0,
        log1,
    ]).unwrap();

    let mut errfile = std::fs::File::create("/home/me/code/rust/rchess/panic.log").unwrap();
    let err_redirect = Redirect::stderr(errfile).unwrap();
}

