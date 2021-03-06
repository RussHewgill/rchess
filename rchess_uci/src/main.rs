#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]

#![allow(clippy::all)]

use rchess_engine_lib::alphabeta::ABResult;
use rchess_engine_lib::movegen::MoveGen;
use rchess_engine_lib::types::*;
use rchess_engine_lib::tables::*;
use rchess_engine_lib::explore::*;
use rchess_engine_lib::evaluate::*;
// use rchess_engine_lib::threading::*;

use std::str::FromStr;
use std::io;
use std::io::{BufRead,Stdout};
use std::sync::mpsc;
use std::{thread, time};
use std::cell::RefCell;

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool,Ordering};

use chrono::{Datelike,Timelike};
use std::io::Write;
use log::{debug, error, log_enabled, info, Level};
use simplelog::*;
use gag::Redirect;

use rayon::ThreadPoolBuilder;

const STARTPOS: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn main() -> std::io::Result<()> {

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

    let mut errfile = std::fs::File::create("/home/me/code/rust/rchess/panic.log").unwrap();
    let err_redirect = Redirect::stderr(errfile).unwrap();

    // let hook = std::panic::take_hook();
    // std::panic::set_hook(Box::new(move |panicinfo| {
    //     let loc = panicinfo.location();
    //     let mut file = std::fs::File::create("/home/me/code/rust/rchess/panic.log").unwrap();
    //     let s = format!("Panicking, Location: {:?}", loc);
    //     file.write(s.as_bytes()).unwrap();
    //     hook(panicinfo)
    // }));

    let timesettings = TimeSettings::new_f64(0.0, 0.5);

    let ts = Tables::new();
    // let ts = &_TABLES;

    let mut g = Game::from_fen(&ts, STARTPOS).unwrap();

    #[cfg(not(feature = "threadpool"))]
    let mut explorer = Explorer::new(White,Game::default(), MAX_SEARCH_PLY, timesettings);

    #[cfg(feature = "threadpool")]
    let mut explorer = Explorer2::new(White,g, MAX_SEARCH_PLY, timesettings);

    explorer.load_nnue("/home/me/code/rust/rchess/nn-63376713ba63.nnue").unwrap();

    #[cfg(feature = "threadpool")]
    explorer.spawn_threads();

    // explorer.load_syzygy("/home/me/code/rust/rchess/tables/syzygy/").unwrap_or_default();

    // let evpath = "/home/me/code/rust/rchess/evparams.bin";
    // let (ev_mid,ev_end) = EvalParams::read_evparams(evpath).unwrap();

    // let (ev_mid,ev_end) = EvalParams::new_mid_end();
    // explorer.cfg.eval_params_mid = ev_mid;
    // explorer.cfg.eval_params_end = ev_end;

    let mut g0 = Game::from_fen(&ts, STARTPOS).unwrap();

    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        if let Ok(line) = line {
            // if line != "" {
            if !line.is_empty() {
                // writeln!(&mut logfile, "{}", line)?;
                debug!("input line: {}", line);
                let mut params = line.split_whitespace();

                match params.next().unwrap() {
                    "uci"        => uci(&explorer),
                    "isready"    => println!("readyok"),
                    "ucinewgame" => {
                        // let mut g = Game::new();
                        let mut g = Game::from_fen(&ts, STARTPOS).unwrap();
                        // explorer.lock().unwrap().side = Black;
                        // explorer.lock().unwrap().game = g;
                        // timeset = false;
                        // explorer.side = Black;
                        // explorer.game = g;
                        explorer.update_game(g);
                        explorer.new_game(&ts, g);
                        #[cfg(feature = "threadpool")]
                        explorer.clear_threads();
                    },
                    "setoption"   => {
                        set_option(&mut explorer, params.clone().collect());
                    },
                    "position"   => {
                        match params.next().unwrap() {
                            "fen" => {
                                let fen = line.replace("position fen ", "");

                                let mut xs = fen.split("moves ");
                                let fen = xs.next().unwrap();

                                let mut g = Game::from_fen(&ts, &fen).unwrap();

                                // eprintln!("fen = {:?}", fen);
                                match xs.next() {
                                    Some(moves) => {
                                        let moves = moves.split(" ");

                                        // for m in moves {
                                        //     let from = &m[0..2];
                                        //     let to = &m[2..4];
                                        //     let other = &m[4..];
                                        //     let mm = g.convert_move(from, to, other).unwrap();
                                        //     g = g.make_move_unchecked(&ts, mm).unwrap();
                                        // }

                                        explorer.update_game_movelist(&ts, &fen, moves);

                                    },
                                    None => {},
                                }

                                // explorer.lock().unwrap().side = g.state.side_to_move;
                                // explorer.lock().unwrap().game = g;
                                debug!("setting game FEN = {}", g.to_fen());
                                // explorer.side = g.state.side_to_move;
                                // explorer.game = g;
                                explorer.update_game(g.clone());
                            },
                            "startpos" => {
                                params.next();
                                let moves: Vec<&str> = params.collect();
                                // let moves = moves.join(" ");
                                // println!("moves = {:?}", moves);
                                let mut g = g0.clone();

                                // for m in moves {
                                //     let from = &m[0..2];
                                //     let to = &m[2..4];
                                //     let other = &m[4..];
                                //     let mm = g.convert_move(from, to, other).unwrap();
                                //     g = g.make_move_unchecked(&ts, mm).unwrap();
                                // }

                                // explorer.side = g.state.side_to_move;
                                // explorer.game = g;
                                // explorer.update_game(g.clone());

                                let moves = moves.into_iter();
                                explorer.update_game_movelist(&ts, STARTPOS, moves);

                                debug!("setting game FEN = {}", explorer.game.to_fen());

                            },
                            x => panic!("Position not fen? {:?},  {:?}", x, params),
                        }
                    },
                    // "stop"       => should_stop.store(true, Ordering::SeqCst),
                    "stop"       => explorer.stop.store(true, Ordering::SeqCst),
                    "ponderhit"  => unimplemented!(),
                    "quit"       => return Ok(()),
                    "go"         => {

                        debug!("explorer going: ");

                        // if let Some(ref mut nnue) = explorer.nnue {
                        //     // nnue.ft.accum.needs_refresh = [true; 2];
                        //     nnue.ft.reset_accum(&explorer.game);
                        // }

                        parse_go(&mut explorer, params.clone().collect());

                        // explorer.time_settings.move_time   = 500;
                        // explorer.time_settings.is_per_move = true;

                        // let m = explorer.lock().unwrap().explore(&ts, depth).unwrap();
                        // let (m,stats) = explorer.explore(&ts);

                        #[cfg(feature = "threadpool")]
                        let (m,stats) = explorer.explore();
                        #[cfg(not(feature = "threadpool"))]
                        let (m,stats) = explorer.explore(&ts);

                        debug!("m = {:?}", m);
                        let (mv,score) = m.unwrap();

                        // let mvs = MoveGen::generate_list_legal(&ts, &explorer.game, None);
                        // let mv = mvs[0];
                        // let score = ABResult::new_single(mv, 0);
                        // let stats = SearchStats::default();

                        let mm = format_move(mv);
                        // print_info(&explorer, (mv,score), stats);
                        print_info((mv,score), stats);
                        println!("bestmove {}", mm);
                    },
                    s            => unimplemented!("bad command: {:?}", s),
                }
            }

        }

    }
    Ok(())
}

fn uci(ex: &Explorer) {
    println!("id name RChess");
    println!("id author me");

    ex.options.print();

    println!("uciok");
}

fn set_option(mut ex: &mut Explorer, params: Vec<&str>) {
    let mut ps = params.clone().into_iter();
    // println!("params = {:?}", params);

    ps.next().unwrap(); // name
    let name = ps.next().unwrap();
    ps.next().unwrap(); // value
    let val = ps.next().unwrap();

    ex.set_option(name, val);

    // println!("ex.cfg.num_threads = {:?}", ex.cfg.num_threads);
}

// fn parse_go(mut ex: &mut Explorer,params: Vec<&str>) {
fn parse_go(
    #[cfg(feature = "threadpool")]
    mut ex: &mut Explorer2,
    #[cfg(not(feature = "threadpool"))]
    mut ex: &mut Explorer,
    params: Vec<&str>
) {

    let mut ps = params.clone().into_iter();

    while let Some(cmd) = ps.next() {
        match cmd {
            "searchmoves" => {
                unimplemented!()
            },
            "ponder"      => {
                // ex.timer.settings.ponder = true;
                unimplemented!()
            },
            "wtime"       => {
                let val = i64::from_str(ps.next().unwrap()).unwrap();
                if val < 0 {
                    ex.time_settings.update_time_remaining(0, White, ex.side == White);
                } else {
                    ex.time_settings.update_time_remaining(val as u64, White, ex.side == White);
                }
            },
            "btime"       => {
                let val = i64::from_str(ps.next().unwrap()).unwrap();
                if val < 0 {
                    ex.time_settings.update_time_remaining(0, Black, ex.side == Black);
                } else {
                    ex.time_settings.update_time_remaining(val as u64, Black, ex.side == Black);
                }
            },
            "winc"        => {
                let val = u64::from_str(ps.next().unwrap()).unwrap();
                // let t = val as f64 / 1000.0;
                // ex.timer.settings.increment[White] = t;
                ex.time_settings.increment = val;
            },
            "binc"        => {
                let val = u64::from_str(ps.next().unwrap()).unwrap();
                let t = val as f64 / 1000.0;
                // ex.timer.settings.increment[White] = t;
                ex.time_settings.increment = val;
            },
            "movestogo"   => {
                let val = u32::from_str(ps.next().unwrap()).unwrap();
                // ex.timer.moves_to_go = Some(val);
                ex.time_settings.moves_to_go = Some(val);
            },
            "depth"       => {
                let val = u32::from_str(ps.next().unwrap()).unwrap();
            },
            "nodes"       => {
                let val = u32::from_str(ps.next().unwrap()).unwrap();
            },
            "mate"        => {
                let val = u32::from_str(ps.next().unwrap()).unwrap();
            },
            "movetime"    => {
                let val = u64::from_str(ps.next().unwrap()).unwrap();

                // let t = val as f64 / 1000.0;
                // ex.timer.settings.increment   = [t; 2];
                // ex.timer.settings.increment_m = val as u64;
                // ex.timer.move_time = val as u64;

                ex.time_settings.move_time = val;
                ex.time_settings.is_per_move = true;

            },
            "infinite"    => {
                // ex.timer.settings.infinite = true;
            },
            _             => {
                debug!("unrecognized go command: {:?}", &params);
                break;
            },
        }
    }

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

// fn print_info(ex: &Explorer, (mv,res): (Move, ABResult), stats: SearchStats) {
fn print_info((mv,res): (Move, ABResult), stats: SearchStats) {

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
        m@Move::Promotion { .. } | m@Move::PromotionCapture { .. } => {

            let new_piece = match mv {
                Move::Promotion { new_piece, .. }  => new_piece,
                Move::PromotionCapture { pcs, .. } => pcs.first(),
                _                                  => unimplemented!(),
            };

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

