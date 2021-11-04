#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]

#![allow(clippy::all)]

pub mod notation;

use chrono::Datelike;
use chrono::Timelike;
use rchess_engine_lib::alphabeta::ABResult;
use rchess_engine_lib::types::*;
use rchess_engine_lib::tables::*;
use rchess_engine_lib::explore::*;
use rchess_engine_lib::evaluate::*;

use std::io;
use std::io::{BufRead,Stdout};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::TryRecvError;
use std::{thread, time};

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool,Ordering};

use std::io::Write;
use log::{debug, error, log_enabled, info, Level};
use simplelog::*;
use gag::Redirect;

use rayon::ThreadPoolBuilder;

const STARTPOS: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn main() -> std::io::Result<()> {

    // let _ = ThreadPoolBuilder::new()
    //     .num_threads(6)
    //     .build_global()
    //     .unwrap();

    let depth = 35;
    // let depth = 25;

    let now = chrono::Local::now();
    let mut logpath = format!(
        "/home/me/code/rust/rchess/logs/log{:0>4}-{:0>2}-{:0>2}_{:0>2}:{:0>2}:{:0>2}-1.log",
        now.year(), now.month(), now.day(),
        now.hour(), now.minute(), now.second());
    if std::path::Path::new(&logpath).exists() {
        logpath = format!(
            "/home/me/code/rust/rchess/logs/log{:0>4}-{:0>2}-{:0>2}_{:0>2}:{:0>2}:{:0>2}-2.log",
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

    // let timer = Timer::default(should_stop.clone());
    // let searcher = Arc::new(Mutex::new(Searcher::new(EngineSettings::default(), timer)));

    let should_stop  = Arc::new(AtomicBool::new(false));
    // let timesettings = TimeSettings::new_f64(10., 0.1);
    let timesettings = TimeSettings::new_f64(
        0.0,
        // 0.5,
        1.0,
        // 0.4,
        // 0.4,
    );
    // let mut timeset = false;

    // let explorer = Arc::new(Mutex::new(
    //     Explorer::new(White,Game::empty(), depth, should_stop.clone(), timesettings)));
    let mut explorer = Explorer::new(White,Game::empty(), depth, should_stop.clone(), timesettings);
    let ts = Tables::new();

    let g0 = {
        let mut g0 = Game::from_fen(&ts, STARTPOS).unwrap();
        let _ = g0.recalc_gameinfo_mut(&ts);
        g0
    };

    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        if let Ok(line) = line {
            // if line != "" {
            if !line.is_empty() {
                // writeln!(&mut logfile, "{}", line)?;
                debug!("input line: {}", line);
                let mut params = line.split_whitespace();

                match params.next().unwrap() {
                    "uci"        => uci(),
                    "isready"    => println!("readyok"),
                    "ucinewgame" => {
                        // let mut g = Game::new();
                        let mut g = Game::from_fen(&ts, STARTPOS).unwrap();
                        // explorer.lock().unwrap().side = Black;
                        // explorer.lock().unwrap().game = g;
                        // timeset = false;
                        explorer.side = Black;
                        explorer.game = g;
                    },
                    "position"   => {
                        match params.next().unwrap() {
                            "fen" => {
                                let fen = line.replace("position fen ", "");

                                let mut xs = fen.split("moves ");
                                let fen = xs.next().unwrap();

                                let mut g = Game::from_fen(&ts, &fen).unwrap();
                                let _ = g.recalc_gameinfo_mut(&ts);

                                // eprintln!("fen = {:?}", fen);
                                match xs.next() {
                                    Some(moves) => {

                                        let moves = moves.split(" ");
                                        for m in moves {
                                            let from = &m[0..2];
                                            let to = &m[2..4];
                                            let other = &m[4..];
                                            let mm = g.convert_move(from, to, other).unwrap();
                                            g = g.make_move_unchecked(&ts, mm).unwrap();
                                        }

                                    },
                                    None => {},
                                }

                                // explorer.lock().unwrap().side = g.state.side_to_move;
                                // explorer.lock().unwrap().game = g;
                                debug!("setting game FEN = {}", g.to_fen());
                                explorer.side = g.state.side_to_move;
                                explorer.game = g;
                            },
                            "startpos" => {
                                params.next();
                                let moves: Vec<&str> = params.collect();
                                // let moves = moves.join(" ");
                                // println!("moves = {:?}", moves);
                                let mut g = g0.clone();
                                for m in moves {
                                    let from = &m[0..2];
                                    let to = &m[2..4];
                                    let other = &m[4..];
                                    let mm = g.convert_move(from, to, other).unwrap();
                                    g = g.make_move_unchecked(&ts, mm).unwrap();
                                }
                                debug!("setting game FEN = {}", g.to_fen());
                                explorer.side = g.state.side_to_move;
                                explorer.game = g;
                            },
                            x => panic!("Position not fen? {:?},  {:?}", x, params),
                        }
                    },
                    "stop"       => should_stop.store(true, Ordering::Relaxed),
                    "ponderhit"  => unimplemented!(),
                    "quit"       => return Ok(()),
                    "go"         => {

                        debug!("explorer going: ");

                        // let m = explorer.lock().unwrap().explore(&ts, depth).unwrap();
                        let (m,stats) = explorer.explore(&ts, None);
                        let (mv,score) = m.unwrap();

                        let mm = format_move(mv);
                        println!("bestmove {}", mm);

                        print_info(&explorer, (mv,score), stats);

                    },
                    s            => unimplemented!("bad command: {:?}", s),
                }
            }

        }

    }
    Ok(())
}

// info depth 245
//     seldepth 3
//     multipv 1
//     score mate -1
//     nodes 12806
//     nps 2561200
//     tbhits 0
//     time 5
//     pv d5d4 e2e8

fn print_info(ex: &Explorer, (mv,res): (Move, ABResult), stats: SearchStats) {

    print!("info");

    // print!(" depth {}", stats.max_depth);
    // print!(" nodes {}", stats.nodes + stats.qt_nodes);

    print!(" score ");
    let score = res.score;
    if score > CHECKMATE_VALUE - 200 {
        let s = score - CHECKMATE_VALUE;
        print!("mate {}", s.abs());
    } else if score < -CHECKMATE_VALUE + 200 {
        let s = score + CHECKMATE_VALUE;
        print!("mate -{}", s.abs());
    } else {
        print!("cp {}", score);
    }

    // let ms = res.moves.iter().map(|m| format_move(*m)).collect::<Vec<_>>().join(" ");
    // print!(" pv {}", ms);

    println!();
}

fn format_move(mv: Move) -> String {
    match mv {
        m@Move::Promotion { new_piece, .. } | m@Move::PromotionCapture { new_piece, .. } => {
            let c = match new_piece {
                Queen  => 'q',
                Knight => 'n',
                Rook   => 'r',
                Bishop => 'b',
                _      => panic!("Bad promotion"),
            };
            let mm = format!("{:?}{:?}{}", m.sq_from(), m.sq_to(), c).to_ascii_lowercase();
            mm
        },
        _ => {
            let mm = format!("{:?}{:?}", mv.sq_from(), mv.sq_to()).to_ascii_lowercase();
            mm
        },
    }
}

// fn handle_command(line: &str) -> std::io::Result<()> {
//     let mut params = line.split_whitespace();

//     match params.next().unwrap() {
//         "uci"        => {
//             uci();
//             Ok(())
//         },
//         "isready"    => unimplemented!(),
//         "ucinewgame" => unimplemented!(),
//         "position"   => unimplemented!(),
//         "go"         => unimplemented!(),
//         "stop"       => unimplemented!(),
//         "ponderhit"  => unimplemented!(),
//         "quit"       => unimplemented!(),
//         _            => unimplemented!(),
//     }
// }

fn uci() {
    println!("id name RChess");
    println!("id author me");
    println!("uciok");
}

// fn spawn_stdout_channel() -> Sender<String> {
//     let (tx, rx) = mpsc::channel::<String>();
//     thread::spawn(move || loop {
//         match rx.try_recv() {
//             Ok(line) => println!("{}", line),
//             _        => unimplemented!(),
//         }
//         // io::stdin().read_line(&mut buffer).unwrap();
//         // tx.send(buffer).unwrap();
//     });
//     tx
// }

// fn spawn_stdin_channel() -> Receiver<String> {
//     let (tx, rx) = mpsc::channel::<String>();
//     thread::spawn(move || loop {
//         let mut buffer = String::new();
//         io::stdin().read_line(&mut buffer).unwrap();
//         tx.send(buffer).unwrap();
//     });
//     rx
// }

