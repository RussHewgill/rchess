#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]

pub mod notation;

use rchess_engine_lib::types::*;
use rchess_engine_lib::tables::*;
use rchess_engine_lib::explore::*;

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
use gag::Redirect;

use rayon::ThreadPoolBuilder;

const STARTPOS: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn main() -> std::io::Result<()> {

    // let _ = ThreadPoolBuilder::new()
    //     .num_threads(6)
    //     .build_global()
    //     .unwrap();

    let depth = 10;

    let logpath = "log.log";
    let mut logfile = std::fs::OpenOptions::new()
        .truncate(true)
        .read(true)
        .create(true)
        .write(true)
        .open(logpath)
        .unwrap();

    // let timer = Timer::default(should_stop.clone());
    // let searcher = Arc::new(Mutex::new(Searcher::new(EngineSettings::default(), timer)));

    let should_stop  = Arc::new(AtomicBool::new(false));
    let timesettings = TimeSettings::new_f64(10., 0.1);

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
            if line != "" {
                writeln!(&mut logfile, "{}", line)?;
                let mut params = line.split_whitespace();

                match params.next().unwrap() {
                    "uci"        => uci(),
                    "isready"    => println!("readyok"),
                    "ucinewgame" => {
                        // let mut g = Game::new();
                        let mut g = Game::from_fen(&ts, STARTPOS).unwrap();
                        let _ = g.recalc_gameinfo_mut(&ts);
                        // explorer.lock().unwrap().side = Black;
                        // explorer.lock().unwrap().game = g;
                        explorer.side = Black;
                        explorer.game = g;
                    },
                    "position"   => {
                        match params.next().unwrap() {
                            "fen" => {
                                let fen = line.replace("position fen ", "");
                                // eprintln!("fen = {:?}", fen);
                                let mut g = Game::from_fen(&ts, &fen).unwrap();
                                let _ = g.recalc_gameinfo_mut(&ts);
                                // explorer.lock().unwrap().side = g.state.side_to_move;
                                // explorer.lock().unwrap().game = g;
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
                                    g = g.make_move_unchecked(&ts, &mm).unwrap();
                                    // eprintln!("from, to = {:?}, {:?}", from, to);
                                }
                                // explorer.lock().unwrap().side = g.state.side_to_move;
                                // explorer.lock().unwrap().game = g;
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

                        // let m = explorer.lock().unwrap().explore(&ts, depth).unwrap();
                        let (m,_) = explorer.explore(&ts, None);
                        let m = m.unwrap();

                        match m {
                            Move::Promotion { new_piece, .. } | Move::PromotionCapture { new_piece, .. } => {
                                let c = match new_piece {
                                    Queen  => 'q',
                                    Knight => 'n',
                                    Rook   => 'r',
                                    Bishop => 'b',
                                    _      => panic!("Bad promotion"),
                                };
                                let mm = format!("{:?}{:?}{}", m.sq_from(), m.sq_to(), c).to_ascii_lowercase();
                                // let mm = format!("{:?}{:?}", m.sq_from(), m.sq_to(), c).to_ascii_lowercase();
                                println!("bestmove {}", mm);
                            },
                            _ => {
                                let mm = format!("{:?}{:?}", m.sq_from(), m.sq_to()).to_ascii_lowercase();
                                println!("bestmove {}", mm);
                            },
                        }


                    },
                    s            => unimplemented!("bad command: {:?}", s),
                }
            }

        }

    }
    Ok(())
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

