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
mod brownian;

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
use crate::sprt::elo::EloType;

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

use crate::sprt::elo::get_elo_penta;
use crate::sprt::log_likelyhood;
use crate::sprt::sprt_penta::*;
use crate::tuner_types::MatchResult;
use crate::supervisor::*;

// fn main() {
fn main5() {
    use crate::sprt::*;
    // use crate::sprt::gsprt::*;
    use crate::sprt::sprt_penta::*;
    use crate::sprt::*;
    use crate::sprt::helpers::*;
    use crate::sprt::elo::*;

    // let (elo0,elo1) = (10.0,0.0);
    // let (elo0,elo1) = (0.0,200.0);

    // let s0 = log_likelyhood(s0);
    // let s1 = log_likelyhood(s1);

    // simulate(200.0);

    // let wdl = (22,9,14);

    // let wdl = (865,1126,876);
    // let (elo,err) = get_elo(wdl);
    // eprintln!("elo = {:.1}", elo);
    // eprintln!("err = {:.2}", err);

    // let wdl = (4821,15060,4858);

    // let ldw = (4858,15060,4821);
    // let total = RunningTotal {
    //     ll:     457,
    //     ld_dl:  2906,
    //     lw_dd:  5608,
    //     dw_wd:  2928,
    //     ww:     433,
    // };

    let ldw = (4542,7919,4771);
    let total = RunningTotal {
        ll:     14,
        ld_dl:  1655,
        lw_dd:  5048,
        dw_wd:  1886,
        ww:     13,
    };

    // let total = RunningTotal::from_vec(&[39, 2226, 31451, 2412, 40]); // (0.2, 0.9), (0.764, 3.439)

    // let (losses, draws, wins) = (4542,7919,4771);

    // let (s0,s1) = (-3.0, 3.0);
    let (s0,s1) = (0.0, 5.0);
    // let (s0,s1) = (0.466, 2.796);
    // let (s0,s1) = (0.2, 0.9);
    // let (s0,s1) = (0.764, 3.439);

    // eprintln!("s0 = {:.2},{:.2}", s0, s1);

    // let mut sprt = SPRT::new_def_ab(s0, s1);
    let mut sprt = SPRT::new_with_elo_type(s0, s1, 0.05, EloType::Normalized);

    // let (elo,(elo_min,elo_max)) = elo_tri(wins, draws, losses);
    // eprintln!("elo = {:.2}", elo);

    let h = sprt.sprt_penta(total);
    // let h = sprt.sprt_tri(wins, draws, losses);

    // eprintln!("h = {:?}", h);

    if let Some(hyp) = h {
        if hyp == Hypothesis::H0 {
            // println!("H0 (null): A is NOT stronger than B by at least {} ELO points, elo1 = {}",
            //          sprt.elo0, sprt.elo1);

            println!("failed");
            println!("H0 (null): A is not stronger than B by at least {} (ELO1) ELO points",
                     sprt.elo1);

        } else {
            // println!("H1: is that A is stronger than B by at least {} ELO points",
            //          sprt.elo1);

            println!("passed");
            println!("H1: A is stronger than B by at least {} (ELO0) ELO points",
                     sprt.elo0);

        }
    }

    #[cfg(feature = "nope")]
    {
        let mut sprts = vec![];
        for elo in [0.,1.,2.,3.,4.,5.,6.,7.,8.,9.,10.,15.,20.,30.,40.,50.,60.,80.,100.,150.,200.] {
            sprts.push((elo as u32, SPRT::new(0., elo, 0.05)));
        }
        let mut min: Option<u32> = None;
        let mut max: Option<u32> = None;
        let mut brackets = [0.0f64; 2];

        for (elo, sprt) in sprts.iter_mut() {

            if let Some(hyp) = sprt.sprt_penta(total) {
                if hyp == Hypothesis::H0 {
                    println!(" H0 (null): A is NOT stronger than B by at least {} ELO points, elo1 = {}",
                                sprt.elo0, sprt.elo1);

                    // let tot = total.to_vec().into_iter().sum::<u32>() as f64;
                    // debug!("total.ll    = {:.2}", total.ll as f64 / tot);
                    // debug!("total.ld_dl = {:.2}", total.ld_dl as f64 / tot);
                    // debug!("total.lw_dd = {:.2}", total.lw_dd as f64 / tot);
                    // debug!("total.dw_wd = {:.2}", total.dw_wd as f64 / tot);
                    // debug!("total.ww    = {:.2}", total.ww as f64 / tot);

                    max = Some(*elo);
                    brackets[1] = *elo as f64;
                    println!("brackets = {:?}", brackets);
                } else {
                    println!(" H1: is that A is stronger than B by at least {} ELO points",
                                sprt.elo1);

                    // let tot = total.to_vec().into_iter().sum::<u32>() as f64;
                    // debug!("total.ll    = {:.2}", total.ll as f64 / tot);
                    // debug!("total.ld_dl = {:.2}", total.ld_dl as f64 / tot);
                    // debug!("total.lw_dd = {:.2}", total.lw_dd as f64 / tot);
                    // debug!("total.dw_wd = {:.2}", total.dw_wd as f64 / tot);
                    // debug!("total.ww    = {:.2}", total.ww as f64 / tot);

                    min = Some(*elo);
                    brackets[0] = *elo as f64;
                    println!("brackets = {:?}", brackets);
                }
            }

        }

        eprintln!("min = {:?}", min);
        eprintln!("max = {:?}", max);
    }

    // let llr = ll_ratio(wdl, -1.0, 3.0);
    // eprintln!("llr = {:?}", llr);

    // let lelo = (-1.0, 3.0);
    // let nelo = (-1.62, 4.861);
    // let belo = (-1.589, 4.767);

    // let (k0,_) = elo_logistic_to_bayes_elo(lelo.0, 0.609);

    // let draw_elo = calc_draw_elo(ldw);
    // eprintln!("draw_elo = {:?}", draw_elo);

    // let (w,d,l) = bayes_elo_to_prob(belo.0, draw_elo);
    // eprintln!("(w,d,l) = {:?}", (w,d,l));

    // let k1 = bayes_elo_to_logistic(belo.0, draw_elo);
    // eprintln!("k1 = {:.2}", k1);

    // let (elo0,elo1) = (-1.0, 3.0);
    // let (elo0,elo1) = (0.23, 1.379);

    // let (s0,s1) = (log_likelyhood(elo0), log_likelyhood(elo1));
    // eprintln!("(s0,s1) = ({:.2},{:.2})", s0, s1);

    // let (sum,pdf) = results_to_pdf(ldw);
    // let (sum,pdf) = results_to_pdf(wdl);
    // let (sum,pdf) = results_penta_to_pdf(total);

    // let llr3 = ll_ratio(ldw, elo0, elo1);
    // eprintln!("llr3 = {:.3}", llr3);

    // let llr5 = ll_ratio_penta(total, elo0, elo1);
    // eprintln!("llr5 = {:.3}", llr5);

    // let (elo, elo95, los) = get_elo(ldw);
    // // let (elo, elo95, los) = get_elo_penta(total);

    // eprintln!("elo   = {:.3}", elo);
    // eprintln!("elo95 = {:.3}", elo95);
    // eprintln!("los   = {:.3}", los);

    // let llr = ll_ratio_penta(total, elo0, elo1);
    // eprintln!("llr = {:?}", llr);

    // let llr_alt = gsprt::llr_alt(&pdf, s0, s1);
    // eprintln!("llr_alt  = {:?}", llr_alt);

    // let jumps = llr_jumps(&pdf, s0, s1);
    // for j in jumps.iter() {
    //     eprintln!("j = ({:.4},{:.4})", j.0, j.1);
    // }

    // let (elo,err) = get_elo(wdl);
    // eprintln!("elo = {:.1}", elo);
    // eprintln!("err = {:.2}", err);

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
    use crate::sprt::sprt_penta::*;
    use crate::sprt::helpers::*;
    use crate::sprt::elo::*;

    init_logger();

    /// approx +4.4 Elo
    let ldw = (4542,7919,4771);
    let total = RunningTotal {
        ll:     14,
        ld_dl:  1655,
        lw_dd:  5048,
        dw_wd:  1886,
        ww:     13,
    };

    // let sum = total.to_vec().into_iter().map(|x| x as f64).sum::<f64>();
    // let penta = total.to_vec().into_iter().map(|x| x as f64 / sum).collect::<Vec<_>>();
    // println!("(ll,ld_dl,lw_dd,dw_wd,ww) = ({:>3.3},{:>3.3},{:>3.3},{:>3.3},{:>3.3})",
    //        penta[0], penta[1], penta[2], penta[3], penta[4]);

    // let (elo,(elo95,los,stddev)) = get_elo_penta(total);
    // eprintln!("elo    = {:.2}", elo);
    // eprintln!("elo95  = {:.2}", elo95);
    // eprintln!("stddev = {:.2}", stddev);

    fn test(total: RunningTotal, elo0: f64, elo1: f64) -> Option<Hypothesis> {
        let mut sprt = SPRT::new_with_elo_type(elo0, elo1, 0.05, EloType::Logistic);
        sprt.sprt_penta(total)
    }

    let elos = vec![
        (5.0, 0.),
        (0., 0.),
        (0., 5.0),
        (0., 10.0),
        (0., 15.0),
        (0., 20.0),
        (0., 25.0),
    ];

    // for (elo0,elo1) in elos.into_iter() {
    //     let h   = test(total, elo0, elo1);
    //     let llr = ll_ratio_logistic_penta(total, elo0, elo1);
    //     let h   = format!("{:?}", h);
    //     eprintln!("({elo0:>2.0},{elo1:>2.0}) = {:>10}, llr = {:>8.2}", h, llr);
    // }

    // let mut rng: StdRng = SeedableRng::seed_from_u64(1234);
    // let elo_diff = 0.0;
    // let penta_wdl = crate::sprt::random::pick(elo_diff, [-90.0, 200.0], &mut rng);
    // eprintln!("penta_wdl = {:?}", penta_wdl);

    // let elo = 5.0;
    // let belo = elo_logistic_to_bayes_elo(elo, 0.8);
    // eprintln!("belo = {:.3}", belo.0);

    // simulate_supervisor(Some(5.0), 0.05);

    let elo_diff = -5.0;
    simulate_get_elo(elo_diff, 1_000_000);

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
    let belo = elo_logistic_to_bayes_elo(elo, 0.4);

    let elo2 = bayes_elo_to_logistic(5.0, 327.0);

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

