#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_doc_comments)]

#![allow(clippy::all)]

mod sprt;
mod tuner_types;
mod parsing;
mod supervisor;
mod json_config;
mod optimizer;
mod gamerunner;
mod simulate;

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
use simulate::*;

// use sprt::*;

use std::io::BufReader;
use std::io::{self,Write,BufRead,Stdout,Stdin};
use std::process::{Command,Stdio};

use itertools::Itertools;

use rand::prelude::SliceRandom;
use rand::{Rng,SeedableRng};
use rand::prelude::StdRng;

use chrono::{Datelike,Timelike};
use log::{debug, error, log_enabled, info, Level};
use simplelog::*;
use gag::Redirect;

use crate::sprt::log_likelyhood;
use crate::tuner_types::MatchResult;
use crate::supervisor::*;

// fn main() {
fn main5() {
    use crate::sprt::*;
    // use crate::sprt::gsprt::*;
    use crate::sprt::sprt_penta::*;
    use crate::sprt::*;
    use crate::sprt::helpers::*;

    let (elo0,elo1) = (10.0,0.0);
    // let (elo0,elo1) = (0.0,200.0);

    // let s0 = log_likelyhood(s0);
    // let s1 = log_likelyhood(s1);

    // simulate(200.0);

    // let wdl = (22,9,14);

    // let total = RunningTotal {
    //     ll:     2424,
    //     ld_dl:  1106,
    //     lw_dd:  5159,
    //     dw_wd:  1187,
    //     ww:     2625,
    // };

    let total = RunningTotal {
        ll:     157,
        ld_dl:  395,
        lw_dd:  538,
        dw_wd:  384,
        ww:     154,
    };

    // let (s0,s1) = (log_likelyhood(elo0), log_likelyhood(elo1));
    // let (sum,pdf) = results_penta_to_pdf(total);
    // let jumps = llr_jumps(&pdf, s0, s1);
    // eprintln!("jumps = {:?}", jumps);
    // let llr = ll_ratio_penta(total, elo0, elo1);
    // eprintln!("llr = {:?}", llr);

    // let llr = ll_ratio(wdl, elo0, elo1);
    // let sprt = sprt(wdl, (elo0, elo1), 0.05, 0.05);

    // eprintln!("llr = {:?}", llr);
    // eprintln!("sprt = {:?}", sprt);

}

fn main() {
// fn main6() {
    use crate::simulate::*;

    init_logger();

    // simulate(5.0, 0.05);
    simulate_supervisor(100.0, 0.05);
}

// fn main() {
fn main7() {
    use crate::sprt::helpers::*;
    use crate::sprt::elo::*;

    // let wdl = (20,20,20);
    // let elo = get_elo(wdl);
    // eprintln!("elo = {:?}", elo);


    let elo = 5.0;
    // let nelo = elo_logistic_to_normalized(elo);
    // let lelo = elo_normalized_to_logistic(elo);
    let belo = elo_to_bayes_elo(elo, 0.4);

    let elo2 = bayes_elo_to_elo(5.0, 327.0);

    eprintln!();
    eprintln!("elo  = {:.1}", elo);
    // eprintln!("nelo = {:.1}", nelo);
    // eprintln!("lelo = {:.1}", lelo);
    eprintln!("belo = {:.1}", belo.0);
    eprintln!("elo2 = {:?}", elo2);

}

// fn main() {
fn main4() {

    init_logger();

    // let engine = Engine::read_from_file("rchess", "engines.json").unwrap();
    // let engine2 = Engine::read_from_file("gnuchess", "engines.json").unwrap();
    // let engine2 = Engine::read_from_file("stockfish", "engines.json").unwrap();
    // let engine2 = Engine::read_from_file("rchess_prev", "engines.json").unwrap();

    let engine1 = Engine::read_from_file("rchess_tuning_0", "engines.json").unwrap();
    let engine2 = Engine::read_from_file("rchess_tuning_1", "engines.json").unwrap();

    let timecontrol = TimeControl::new_f64(1.0, 0.1);
    // let timecontrol = TimeControl::new_f64(0.2, 0.1);
    // let timecontrol = TimeControl::new_f64(0.2, 0.05);

    let tunable = Tunable::new("lmr_reduction".to_string(), 2, 5, 3, 1);

    let mut sup = Supervisor::new(
        engine1,
        engine2,
        // engine.clone(),
        tunable,
        timecontrol,
    );

    sup.find_optimum(2000, true);

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

    // let mut logpath = format!(
    //     "/home/me/code/rust/rchess/logs/log_tuner_{:0>4}-{:0>2}-{:0>2}_{:0>2}:{:0>2}:{:0>2}-1.log",
    //     now.year(), now.month(), now.day(),
    //     now.hour(), now.minute(), now.second());
    // if std::path::Path::new(&logpath).exists() {
    //     logpath = format!(
    //         "/home/me/code/rust/rchess/logs/log_tuner_{:0>4}-{:0>2}-{:0>2}_{:0>2}:{:0>2}:{:0>2}-2.log",
    //         now.year(), now.month(), now.day(),
    //         now.hour(), now.minute(), now.second());
    // };

    let logpath = "test.log";

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
    let log0 = WriteLogger::new(LevelFilter::Trace, cfg.clone(), logfile);

    let log1 = TermLogger::new(LevelFilter::Debug, cfg.clone(), TerminalMode::Stderr, ColorChoice::Auto);

    CombinedLogger::init(vec![
        log0,
        log1,
    ]).unwrap();

    let mut errfile = std::fs::File::create("/home/me/code/rust/rchess/panic.log").unwrap();
    let err_redirect = Redirect::stderr(errfile).unwrap();
}

