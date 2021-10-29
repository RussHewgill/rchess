#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_doc_comments)]

#![feature(core_intrinsics)]

#![allow(clippy::all)]

// #![allow(
//     clippy::all,
//     clippy::restriction,
//     clippy::pedantic,
//     clippy::nursery,
//     clippy::cargo,
// )]

use std::collections::HashMap;
use std::slice::SliceIndex;
use std::str::FromStr;

use itertools::Itertools;
use rchess_engine_lib::explore::Explorer;
// use crate::lib::*;
use rchess_engine_lib::types::*;
use rchess_engine_lib::search::*;
use rchess_engine_lib::tables::*;
use rchess_engine_lib::parsing::*;
use rchess_engine_lib::util::*;
use rchess_engine_lib::evaluate::*;
use rchess_engine_lib::explore::*;
use rchess_engine_lib::tuning::*;

use log::{debug, error, log_enabled, info, Level};
use gag::Redirect;
use simplelog::*;
use chrono::Timelike;

const STARTPOS: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[allow(unreachable_code)]
fn main() {

    // let logpath = "./log.log";
    // use std::fs::OpenOptions;
    // let logfile = OpenOptions::new()
    //     .truncate(true)
    //     .read(true)
    //     .create(true)
    //     .write(true)
    //     .open(logpath)
    //     .unwrap();

    // let err_redirect = Redirect::stderr(logfile).unwrap();

    // env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
    // // env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
    //     .format_timestamp(None)
    //     .format_module_path(false)
    //     .format_target(false)
    //     // .format_level(false)
    //     .init();

    // let now = chrono::Local::now();
    // let logpath = format!(
    //     "/home/me/code/rust/rchess/logs/log-{:0>2}:{:0>2}:{:0>2}.log",
    //     now.hour(), now.minute(), now.second());
    // let mut logfile = std::fs::OpenOptions::new()
    //     .truncate(true)
    //     .read(true)
    //     .create(true)
    //     .write(true)
    //     .open(logpath)
    //     .unwrap();
    // WriteLogger::init(LevelFilter::Debug, Config::default(), logfile).unwrap();
    // // WriteLogger::init(LevelFilter::Trace, Config::default(), logfile).unwrap();

    // rayon::ThreadPoolBuilder::new()
    //     .num_threads(1)
    //     .build_global()
    //     .unwrap();

    // let ts = Tables::new();
    // ts.write_to_file("tables.bin").unwrap();
    // let ts = Tables::read_from_file("tables.bin").unwrap();

    // let from: Coord = "A1".into();
    // let to: Coord = "B2".into();
    // let mut mvs = vec![
    //     Move::Quiet { from, to, pc: Pawn },
    //     Move::Quiet { from, to, pc: Queen },
    //     Move::PawnDouble { from, to },
    //     Move::Capture { from, to, pc: Pawn, victim: Pawn },
    //     Move::EnPassant { from, to, capture: from }
    // ];
    // mvs.sort();
    // for m in mvs.iter() {
    //     eprintln!("m = {:?}", m);
    // }

    // // let s = std::mem::size_of::<Eval>();
    // let s = std::mem::size_of::<SearchStats>();
    // // let s = std::mem::size_of::<GHistory>();
    // eprintln!("s = {:?}", s);
    // // let s = u16::MAX;
    // // eprintln!("s = {:#8x}", s);

    // let t0 = std::time::Instant::now();
    // println!("starting tables");
    // let ts = Tables::new();
    // println!("tables done in {} seconds.", t0.elapsed().as_secs_f64());

    // let g = Game::from_fen(&ts, "k7/8/1p1p1p2/8/1p1q1p2/8/1P1P1P2/7K w - - 0 1").unwrap();
    // let rook   = ts.attacks_rook("D4".into(), g.all_occupied());
    // let bishop = ts.attacks_bishop("D4".into(), g.all_occupied());
    // let b = rook | bishop;
    // eprintln!("rook = {:?}", rook);
    // eprintln!("bishop = {:?}", bishop);

    // let tr = ts.table_rook;

    // let m = ts.magics_rook[49];

    // let mut k = (0,0);

    // // for x in ts.magics_rook.iter() {
    // for x in ts.table_rook.iter() {
    //     k.1 += 1;
    //     // if x.magic.0 != 0 {
    //     if x.0 != 0 {
    //         k.0 += 1;
    //     }
    // }
    // eprintln!("k = {:?}", k);

    // return;

    let mut args: Vec<String> = std::env::args().collect();
    match args.get(1) {
        Some(s) => match s.as_str() {
            // "wac"   => main3(false), // read from file and test
            "wac"   => match args.get(2).map(|x| u64::from_str(x).ok()) {
                Some(Some(n)) => main3(Some(n),false),
                _             => main3(None, false),
            }
            "wac2"  => main3(None, true), // read from file and test, send URL to firefox
            "perft" => match args.get(2).map(|x| u64::from_str(x).ok()) {
                Some(n) => main4(n),
                _       => main4(None),
            }
            _       => {},
        },
        None    => main7(),
    }

    // main6();
    // main5(); // search + eval position
    // main2();
    // main4(); // perft

    // // main8(); // eval testing
    // main7();
    // // main3(); // read from file and test

}

/// XXX: Could possibly get entire move sequence by simply looking up best Zobrist?

fn main8() {
    let fen = STARTPOS;
    let n = 4;

    let mut fens = vec![
        // "k7/8/8/8/8/8/8/7K b - - 0 1", // nothing
        "k7/8/8/8/8/8/5N2/7K w - - 0 1", // single piece
        "k7/8/8/8/8/8/PPPPPPPP/7K w - - 0 1", // none
        "k7/8/8/8/8/2P5/P1PPPPPP/7K w - - 0 1", // one double
        "k7/8/8/8/2P5/2P5/P1P1PPPP/7K w - - 0 1", // one triple
        "k7/8/8/8/8/2P1P3/P1P1PPPP/7K w - - 0 1", // two double
    ];

    // let fen = "r4q1k/5P1b/2p2n1P/p2p1P2/3P4/8/2PKN1Q1/6R1 w - - 1 34";
    // let fen = "7k/2pq2p1/6rp/1P2p3/2Qp1n2/P2P3P/R1P2PPK/3N2R1 b - - 0 28";

    let ts = Tables::new();

    // let mut g = Game::from_fen(&ts, fen).unwrap();
    // let _ = g.recalc_gameinfo_mut(&ts);
    // eprintln!("g = {:?}", g);

    // let stop = Arc::new(AtomicBool::new(false));
    // let timesettings = TimeSettings::new_f64(5.0, 0.1);
    // let mut ex = Explorer::new(g.state.side_to_move, g.clone(), n, stop.clone(), timesettings);

    fens.truncate(1);

    for (i,fen) in fens.iter().enumerate() {

        let fen = "k7/1r6/8/8/8/8/6R1/7K w - - 0 1"; // single piece

        let mut g = Game::from_fen(&ts, fen).unwrap();
        let _ = g.recalc_gameinfo_mut(&ts);
        // eprintln!("g = {:?}", g);

        // let d_pawns = g.count_pawns_doubled(&ts, White);
        // eprintln!("doubled pawns {} = {:?}", i, d_pawns);

        // let ee = g.evaluate(&ts);
        // eprintln!("ee.sum() = {:?}", ee.sum());

        // let e = stockfish_eval(&fen, true).unwrap();
        let (_,e) = stockfish_eval(&fen, false).unwrap();

        let w_mg = e.material_mg[White];
        let b_mg = e.material_mg[Black];
        let w_eg = e.material_eg[White];
        let b_eg = e.material_eg[Black];

        // eprintln!("w_mg = {:?}", w_mg);
        // eprintln!("w_eg = {:?}", w_eg);

        let m_mg_w: f64 = w_mg.iter().sum();
        let m_eg_w: f64 = w_eg.iter().sum();
        let m_mg_b: f64 = b_mg.iter().sum();
        let m_eg_b: f64 = b_eg.iter().sum();

        eprintln!("m_mg_w = {:?}", m_mg_w);
        eprintln!("m_eg_w = {:?}", m_eg_w);
        eprintln!("m_mg_b = {:?}", m_mg_b);
        eprintln!("m_eg_b = {:?}", m_eg_b);

        let tc = e.total_classic;
        let tn = e.total_nn;
        eprintln!("tc = {:>4.2}", tc);
        // eprintln!("tc, tn = {:>4.2}, {:>4.2}", tc, tn);
        // eprintln!("stockfish = {:>4.2}", (tc + tn) / 2.);

        print!("\n");
    }

}

fn main7() {
    let fen = STARTPOS;
    let n = 10;

    init_logger();

    // let fen = "rnbqkbnr/ppppp1pp/8/5P2/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2";
    // let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1";

    // let fen = "5r1k/4Qpq1/4p3/1p1p2P1/2p2P2/1p2P3/3P4/BK6 b - - 0 1";

    // // // minimax = 869088, 2.08 s
    // // // AB      = 92245,  0.24 s
    // let fen = "r4q1k/5P1b/2p2n1P/p2p1P2/3P4/8/2PKN1Q1/6R1 w - - 1 34";

    // // AB = 808182 leaves, 1.87 s
    // let fen = "7k/2pq2p1/6rp/1P2p3/2Qp1n2/P2P3P/R1P2PPK/3N2R1 b - - 0 28";

    // let fen = "2rr3k/pp3pp1/1nnqbN1p/3pN3/2pP4/2P3Q1/PPB4P/R4RK1 w - - 0 1"; // WAC.001 = Qg6 = g3g6
    // let fen = "2rr3k/pp3pp1/1nnqbNQ1/3pN2p/2pP4/2P5/PPB4P/R4RK1 w - - 0 2"; // WAC.001 = Qg6 = g3g6
    // let fen = "2rr3k/pp4p1/1nnqbNpp/3pN3/2pP4/2P5/PPB4P/R4RK1 w - - 0 2"; // WAC.001

    // let fen = "5rk1/1ppb3p/p1pb4/6q1/3P1p1r/2P1R2P/PP1BQ1P1/5RKN w - -"; // WAC.003, e3g3

    // let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - "; // Position 3

    // let fen = "Q3b3/4bkp1/1q2np1p/NPp1p3/2P1P3/4BP1P/4B1P1/7K b - - 1 1"; // Correct = e6c7
    // let fen = "rnbqkb1r/pppp1ppp/8/4P3/6n1/7P/PPPNPPP1/R1BQKBNR b KQkq - 0 1"; // WAC.007, Ne3 = g4e3
    // let fen = "3q1rk1/p4pp1/2pb3p/3p4/6Pr/1PNQ4/P1PB1PP1/4RRK1 b - - 0 1"; // WAC.009, Bh2+ = d6h2

    // let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"; // Perft Position 2

    // let fen = "5k2/6pp/p1qN4/1p1p4/3P4/2PKP2Q/PP3r2/3R4 b - - 0 1"; // WAC.005, Qc4 = c6c4
    // let fen = "4r3/2R3pp/q2pkp2/4p3/4P1P1/4nQ1P/PP6/2K5 w - - 0 1"; // WAC.005, color reversed

    // let fen = "rnbqkb1r/pppp1ppp/8/4P3/6n1/7P/PPPNPPP1/R1BQKBNR b KQkq - 0 1"; // WAC.007, Ne3 = g4e3

    // let fen = "rn2kbnr/pppppppp/8/8/6b1/1QP4P/PP1PqPPN/RNB1KB1R w KQkq - 0 2"; // 1 move, then lots

    // let fen = "k7/1p6/2p5/8/3N4/8/8/7K w - - 0 1"; // Quiescence test
    // let fen = "rnbqkbnr/pppppppp/8/8/8/8/1PP1PPP1/RNBQKBNR w KQkq - 0 1";

    // let fen = "k7/2n5/4p3/3p4/2P1P3/4N3/8/7K w - - 0 1"; // SEE test
    // let fen = "k7/2n5/4p3/3p3R/2P1P1P1/4N3/8/7K w - - 0 1"; // SEE test

    // let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - "; // Position 3

    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"; // Perft Position 2

    let fen = "r2rb1k1/pp1q1p1p/2n1p1p1/2bp4/5P2/PP1BPR1Q/1BPN2PP/R5K1 w - - 0 1"; // WAC.014, h3h7

    // let fen = "3q1rk1/p4pp1/2pb3p/3p4/6Pr/1PNQ4/P1PB1PP1/4RRK1 b - - 0 1"; // WAC.009, Bh2+ = d6h2

    // let fen = "8/1p4pk/6rp/3Pp3/4Qn2/2P2qP1/1B3P1P/4R1K1 b - - 1 1"; // f4h3, #2
    // let fen = "6k1/6pp/3q4/5p2/QP1pB3/4P1P1/4KPP1/2r5 w - - 0 2"; // a4e8, #3
    let fen = "5rk1/ppR1Q1p1/1q6/8/8/1P6/P2r1PPP/5RK1 b - - 0 1"; // b6f2, #-4
    // let fen = "8/p6k/1p5p/4Bpp1/8/1P3q1P/P1Q2P1K/3r4 w - - 0 2"; // c2c7, #5;
    // let fen = "1rq2k1r/p1p2p2/2B2P2/3RP2p/1b3N1p/2N4P/PPP1QPP1/2K4R w - - 1 23"; // e5e6, #9

    // let fen = "2k5/8/KP6/8/8/8/8/8 w - - 1 10"; // #12
    // let fen = "8/8/1K4k1/8/7Q/8/8/8 w - - 7 16"; // #6

    // let fen = "1k3r1r/p1p3p1/1pn3q1/3R1n2/3P4/P1B1p2p/P1PN1PPP/4QK1R w - - 0 22"; // #-10 if d2b3
    let fen = "r1bqk1nr/ppppbppp/2n5/8/4Q3/N7/PPP1PPPP/R1B1KBNR w KQkq - 3 5"; // ??

    // let fen = "r4rk1/4npp1/1p1q2b1/1B2p3/1B1P2Q1/P3P3/5PP1/R3K2R b KQ - 1 1"; // Q cap d6b4

    // let fen = "7k/1n1n4/2P5/8/5b2/8/7P/7K b - - 0 1"; // Horizon
    // let fen = "7k/8/8/r7/r7/8/p1RR4/7K w - - 0 1"; // Horizon

    // /// https://www.chessprogramming.org/Caesar#HorizonEffect
    // let fen = "2kr4/3nR3/p2B1p2/1p1p1Bp1/1P1P3p/2P4P/P5PK/8 b - - 1 32"; // Horizon

    // let fen = "8/8/p1p5/1p5p/1P5p/8/PPP2K1p/4R1rk w - - 0 1";    // Rf1; id ";zugzwang.001";;
    // let fen = "1q1k4/2Rr4/8/2Q3K1/8/8/8/8 w - - 0 1";            // Kh6;  id ";zugzwang.002";;
    // let fen = "7k/5K2/5P1p/3p4/6P1/3p4/8/8 w - - 0 1";           // g5; id ";zugzwang.003";;
    // let fen = "8/6B1/p5p1/Pp4kp/1P5r/5P1Q/4q1PK/8 w - - 0 32";   // Qxh4; id ";zugzwang.004";;
    // let fen = "8/8/1p1r1k2/p1pPN1p1/P3KnP1/1P6/8/3R4 b - - 0 1"; // Nxd5; id ";zugzwang.005";;

    // let fen = "2q2rk1/p4pp1/5n1p/8/8/Q4N1P/P4PP1/5RK1 b - - 0 1";; // Null move cutoff
    // let fen = "5rk1/p4pp1/5n1p/8/8/5N1P/P4PP1/2Q2RK1 b - - 0 2";; // Null move cutoff

    fn games(i: usize) -> String {
        let mut games = read_epd("testpositions/WAC.epd").unwrap();
        // let mut games = read_epd("testpositions/STS6.epd").unwrap();
        let mut games = games.into_iter();
        let games = games.map(|x| x.0).collect::<Vec<_>>();
        games[i - 1].clone()
    }

    // XXX: STS6
    // let fen = &games(3); // 

    // XXX: WAC
    // let fen = &games(2); // b3b2 (SF says b3b7)
    // let fen = &games(4); // h6h7, #2
    // let fen = &games(6); // b6b7, #11
    // let fen = &games(7); // N g4e3
    // let fen = &games(8); // R e7f7, #7
    // let fen = &games(9); // d6h2, #-5
    // let fen = &games(17); // c4e5
    // let fen = &games(18); // a8h8, #27, Tablebase
    // let fen = &games(21); // d2h6

    eprintln!("fen = {:?}", fen);

    // let ts = Tables::new();
    // ts.write_to_file("tables.bin").unwrap();
    // let ts = Tables::read_from_file("tables.bin").unwrap();
    let ts = &_TABLES;

    let mut g = Game::from_fen(&ts, fen).unwrap();

    let stop = Arc::new(AtomicBool::new(false));
    let timesettings = TimeSettings::new_f64(10.0,0.1);
    let mut ex = Explorer::new(g.state.side_to_move, g.clone(), n, stop.clone(), timesettings);

    // let mut ps = vec![
    //     ("pawn x queen", Move::Capture { from: "E4".into(), to: "F5".into(), pc: Pawn, victim: Queen }),
    //     ("queen x pawn", Move::Capture { from: "E4".into(), to: "F5".into(), pc: Queen, victim: Pawn }),
    //     ("pawn x pawn", Move::Capture { from: "E4".into(), to: "D5".into(), pc: Pawn, victim: Pawn }),
    //     ("pawn x rook",   Move::Capture { from: "E4".into(), to: "D5".into(), pc: Pawn, victim: Rook }),
    //     ("bishop x pawn", Move::Capture { from: "E4".into(), to: "D5".into(), pc: Bishop, victim: Pawn }),
    //     ("quiet", Move::Quiet { from: "E4".into(), to: "E5".into(), pc: Pawn }),
    //     ("promotion", Move::Promotion { from: "E7".into(), to: "E8".into(), new_piece: Queen }),
    //     // ("EP", Move::EnPassant { from: "E5".into(), to: "D6".into(), capture: "D5".into() }),
    // ];

    // order_mvv_lva(&mut ps[..]);
    // for (s,m) in ps.iter() {
    //     eprintln!("{:>15} = {:?}", s, m);
    // }

    // return;

    #[allow(unreachable_code)]
    if true {

        // let k = 4;
        let k = 1;
        let mut t: std::time::Duration = std::time::Duration::from_secs(0);
        for q in 0..k {
            let t0 = std::time::Instant::now();

            println!("g = {:?}", g);

            // let n = 25;
            let n = 35;
            // let n = 10;
            // let n = 5;

            ex.max_depth = n;

            ex.timer.settings = TimeSettings::new_f64(
                0.0,
                2.5,
            );

            let ph = g.game_phase();
            eprintln!("ph = {:?}", ph);

            let t0 = std::time::Instant::now();
            let (mvs,stats0,(tt_r,tt_w)) = ex.lazy_smp(&ts, false, false);
            let (mv0,mvs,score) = mvs.get(0).unwrap();
            // println!("m #{} = {:?}", q, mv0);
            println!("explore lazy_smp  (depth: {}) done in {:.3} seconds.",
                     stats0.max_depth, t0.elapsed().as_secs_f64());
            stats0.print(t0.elapsed());

            println!();
            // println!("m #{} = {:?}", q, mv0);
            println!("score, mv: {} = {:?}", score, mv0);
            // println!("Correct move: Cp Q b6f2");
            // println!("Correct move: Q cap d6b4");
            // println!();

            // let mm = Move::Quiet { from: "E1".into(), to: "F1".into(), pc: Rook };
            // assert_eq!(*mv0, mm);

            // for m in mvs.iter() {
            //     eprintln!("m = {:?}", m);
            // }

            stats0.print_ebf(false);
            // stats0.print_ebf(true);
            // stats0.print_node_types(&tt_r);

            // eprintln!("qt nodes = {:?}", stats0.qt_nodes);
            // eprintln!("null prunes = {:?}", stats0.null_prunes);
            // eprintln!("window fails = {:?}", stats0.window_fails);
            // eprintln!("stats0.lmrs = {:?}", stats0.lmrs);

            // {
            //     let (a,b) = stats0.beta_cut_first;
            //     eprintln!("stats0.beta_cut_first = {:.3?}", a as f64 / (a + b) as f64);
            // }

            // let mut k = 0;
            // for (zb,sis) in tt_r.read().unwrap().iter() {
            //     // let si = sis.iter().next().unwrap();
            //     k += 1;
            // }
            // eprintln!("k = {:?}", k);

            // let g2 = g.flip_sides(&ts);
            // let mut ex2 = Explorer::new(g2.state.side_to_move, g2.clone(), n, stop.clone(), timesettings);
            // eprintln!("g2 = {:?}", g2);

            // let t0 = std::time::Instant::now();
            // let (mvs,stats0,(tt_r,tt_w)) = ex2.lazy_smp(&ts, false, false);
            // let (mv1,mvs,_) = mvs.get(0).unwrap();
            // println!("m #{} = {:?}", q, mv1);
            // println!("explore lazy_smp  (depth: {}) done in {:.3} seconds.",
            //          stats0.max_depth, t0.elapsed().as_secs_f64());
            // // stats0.print(t0.elapsed());

            t += t0.elapsed();
        }
        // println!("\n===\nexplore {} times, done in avg {:.3} seconds.", k, t.as_secs_f64() / k as f64);

    } else {

        let fen0 = "rnb1kb1r/pppppNpp/8/8/8/3n4/P1PPPPPP/R1B1KB1R w KQkq - 0 1";
        let g0 = Game::from_fen(&ts, fen0).unwrap();

        let fen = STARTPOS;
        let mut g2 = Game::from_fen(&ts, fen).unwrap();

        // let ms0 = vec![
        //     "e2e4",
        //     "e7e6",
        //     "d2d3",
        // ];

        let ms = "g1f3 g8f6 f3e5 f6e4 b1c3 e4c3 e5c6 c3d1 c6d8 d1b2 d8f7 b2d3";

        let ms0 = ms.split(" ");

        let mut g2 = g2.clone();
        for m in ms0.into_iter() {
            let from = &m[0..2];
            let to = &m[2..4];
            let other = &m[4..];
            let mm = g2.convert_move(from, to, other).unwrap();
            g2 = g2.make_move_unchecked(&ts, mm).unwrap();
        }
        // eprintln!("hash0 = {:?}", g2.zobrist);

        // eprintln!("g2 = {:?}", g2);
        // eprintln!("g0 = {:?}", g0);

        // eprintln!("g0.zobrist == g2.zobrist = {:?}", g0.zobrist == g2.zobrist);
        // g0.state.debug_equal(g2.state);

        let n = 10;

        let mut ex0 = Explorer::new(g0.state.side_to_move, g0.clone(), n, stop.clone(), timesettings);
        let mut ex2 = Explorer::new(g2.state.side_to_move, g2.clone(), n, stop.clone(), timesettings);

        // let moves = vec![
        //     Move::Quiet { from: "E2".into(), to: "E4".into() },
        //     Move::Quiet { from: "D2".into(), to: "D4".into() },
        //     // Move::Quiet { from: "H2".into(), to: "H3".into() },
        // ];

        let t = std::time::Instant::now();
        let (m,stats) = ex2.explore(&ts, None);
        eprintln!("m = {:?}", m.unwrap());
        // ex.rank_moves(&ts, true);
        println!("explore done in {} seconds.", t.elapsed().as_secs_f64());

    }

}

fn main3(num: Option<u64>, send_url: bool) {
    // let mut games = read_ccr_onehour("ccr_onehour.txt").unwrap();
    // let mut games = read_epd("Midgames250.epd").unwrap();
    let mut games = read_epd("testpositions/WAC.epd").unwrap();
    // let mut games = read_epd("testpositions/STS6.epd").unwrap();

    // for (fen,ms) in games.iter() {
    //     // eprintln!("fen, ms = {:?}: {:?}", fen, ms);
    //     eprintln!("ms = {:?}", ms);
    // }

    if let Some(num) = num {
        games.truncate(num as usize);
    }

    let n = 25;

    // let ts = Tables::new();
    let ts = Tables::read_from_file("tables.bin").unwrap();

    let timesettings = TimeSettings::new_f64(
        0.0,
        1.0,
    );

    let mut total = (0,0);
    let t0 = std::time::Instant::now();

    println!("running WAC");

    for (i,(fen,m)) in games.into_iter().enumerate() {
        let i = i + 1;
        let g = Game::from_fen(&ts, &fen).unwrap();
        // eprintln!("g = {:?}", g);

        let stop = Arc::new(AtomicBool::new(false));
        let mut ex = Explorer::new(g.state.side_to_move, g.clone(), n, stop, timesettings);

        // let e = g.evaluate(&ts);
        // let (_,e_sf) = stockfish_eval(&fen, false).unwrap();

        let (m0,stats) = ex.explore(&ts, None);

        // let g = g.flip_sides(&ts);
        // let mut ex = Explorer::new(g.state.side_to_move, g.clone(), n, stop, timesettings);
        // let (m1,_) = ex.explore(&ts, None);

        let mv = m0.unwrap().0.to_algebraic(&g);
        let mv0 = m[0].replace("+", "");
        if mv0 == mv {
            // println!("#{:>2}: Correct", i);
            total.0 += 1;
            total.1 += 1;
        } else {
            total.1 += 1;
            let t = t0.elapsed().as_secs_f64() / total.1 as f64;
            println!(
                "#{:>2}: Wrong, Correct: {:>5}, engine: {:>5}({:?}), ({}/{}), avg: {:.2}",
                i, m[0], mv, m0.unwrap(), total.0, total.1, t);

            if send_url {
                g.open_with_lichess().unwrap();
                break;
            }

            // println!("Correct        Engine");
            // println!("{:<8}       {}", m[0], mv);
        }

    }

    println!("Score = {} / {} ({:.2})", total.0, total.1, total.0 as f64 / total.1 as f64);
    println!("Finished in {:.3} seconds.", t0.elapsed().as_secs_f64());

    // let games = read_json_fens("perft_fens.txt").unwrap();
    // let mut games = games;
    // games.truncate(1);
    // for (depth,nodes,fen) in games.into_iter() {
    //     let depth = 4;
    //     println!("FEN: {}", &fen);
    //     let (done, ((ns0,nodes0),(ns1,nodes1))) = test_stockfish(&fen, depth, false).unwrap();
    //     println!("perft depth {} done in {}", depth, done);
    //     if ns0 == ns1 {
    //         eprintln!("rchess, stockfish = {:>2} / {:>2}", ns0, ns1);
    //     } else {
    //         eprintln!("rchess, stockfish = {:>2} / {:>2} / failed ({})",
    //                   ns0, ns1, ns0 as i64 - ns1 as i64);
    //     }
    // }

}

fn main6() {
    let fens = vec![
        ("start w", "k7/8/8/4p3/3P4/8/8/7K w - - 0 1"),
        ("start b", "k7/8/8/4p3/3P4/8/8/7K b - - 0 1"),

        ("w push", "k7/8/8/3Pp3/8/8/8/7K b - - 0 1"), // push
        ("w cap", "k7/8/8/4P3/8/8/8/7K b - - 0 1"), // capture

        ("b push", "k7/8/8/8/3Pp3/8/8/7K w - - 0 2"), // push
        ("b cap", "k7/8/8/8/3p4/8/8/7K w - - 0 2"), // capture
    ];

    let n = 1;

    let ts = Tables::new();

    // let fen = fens[1].1;
    let fen = STARTPOS;
    let mut g = Game::from_fen(&ts, fen).unwrap();

    // let mut g = Game::empty();
    // g.insert_pieces_mut_unchecked(&vec![
    //     ("H1", King, White),
    //     ("A8", King, Black),
    //     // ("D4", Pawn, White),
    //     // ("E4", Pawn, Black),
    //     ("D4", Pawn, Black),
    // ]);

    // let _ = g.recalc_gameinfo_mut(&ts);
    // eprintln!("g = {:?}", g);

    // let m = Move::Capture { from: "E5".into(), to: "D4".into() };
    // let m = Move::Quiet { from: "E5".into(), to: "E4".into() };
    // let g2 = g.make_move_unchecked(&ts, &m).unwrap();

    // let ew = g.evaluate(&ts).sum_color(White);
    // let eb = g.evaluate(&ts).sum_color(Black);
    // eprintln!("ew = {:?}", ew);
    // eprintln!("eb = {:?}", eb);

    // let e = g.evaluate(&ts).sum(Black);
    // eprintln!("sum = {:?}", e);


    for (s,fen) in fens.iter() {

        let mut g = Game::from_fen(&ts, fen).unwrap();
        let _ = g.recalc_gameinfo_mut(&ts);

        let e = g.evaluate(&ts).sum();
        eprintln!("{} = {:?}", s, e);

    }

}

fn main5() {

    let fen = STARTPOS;

    // let fen = "3b4/1pN5/1P1p4/3pN2R/3kP3/K2B1bP1/1P3P2/6B1 w - - 0 1";
    // let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1";

    // let fen = "rnbqk1nr/p3ppbp/2pp2p1/8/3PP3/4BN2/PPp2PPP/1R1NK2R b Kkq - 1 9";
    // let fen = "rnbqkbnr/pp3ppp/2pp4/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR b KQkq - 1 4";

    // let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - "; // Position 2
    // let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - "; // Position 3
    // let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"; // Position 4

    // let fen = "k7/8/8/4p3/3P4/8/8/7K b - - 0 1";
    let fen1 = "k7/8/8/4p3/3P4/8/8/7K w - - 0 1";
    let fen2 = "k7/8/8/4p3/3P4/8/8/7K b - - 0 1";

    // let games = read_epd("WAC.epd").unwrap();
    // let fen = &games[1].0;

    let fen = "Q3b3/4bkp1/1q2np1p/NPp1p3/2P1P3/4BP1P/4B1P1/7K b - - 1 1"; // Correct = e6c7

    let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - "; // Position 3

    let fen1 = "5k2/6pp/p1qN4/1p1p4/3P4/2PKP2Q/PP3r2/3R4 b - - 0 1"; // WAC.005, Qc4 = c6c4
    let fen2 = "4r3/2R3pp/q2pkp2/4p3/4P1P1/4nQ1P/PP6/2K5 w - - 0 1"; // WAC.005, color reversed

    let n = 3;

    let ts = Tables::new();

    // let mut g = Game::from_fen(&ts, fen).unwrap();
    // let _ = g.recalc_gameinfo_mut(&ts);
    // // eprintln!("g = {:?}", g);

    let mut g1 = Game::from_fen(&ts, fen1).unwrap();
    let _ = g1.recalc_gameinfo_mut(&ts);
    let mut g2 = Game::from_fen(&ts, fen2).unwrap();
    let _ = g2.recalc_gameinfo_mut(&ts);

    eprintln!("g1 = {:?}", g1);
    eprintln!("g2 = {:?}", g2);

    // let mut g1 = Game::from_fen(&ts, fen1).unwrap();
    // let _ = g1.recalc_gameinfo_mut(&ts);

    // let mut g2 = Game::from_fen(&ts, fen2).unwrap();
    // let _ = g2.recalc_gameinfo_mut(&ts);

    let stop = Arc::new(AtomicBool::new(false));
    let timesettings = TimeSettings::new_f64(10., 0.1);
    // let ex = Explorer::new(g.state.side_to_move, g.clone(), n, stop.clone(), timesettings);
    // let ex = Explorer::new(White, g.clone(), n, stop.clone(), timesettings);

    let ex1 = Explorer::new(g1.state.side_to_move, g1.clone(), n, stop.clone(), timesettings);
    let ex2 = Explorer::new(g2.state.side_to_move, g2.clone(), n, stop.clone(), timesettings);

    let t1 = std::time::Instant::now();
    let (mv1,stats1) = ex1.explore(&ts, None);
    // let (mv1,stats1) = ex1.explore(&ts, n, false);
    eprintln!("mv1 = {:?}, (c6c4)", mv1.unwrap());
    stats1.print(t1.elapsed());

    print!("\n");

    let t2 = std::time::Instant::now();
    let (mv2,stats2) = ex2.explore(&ts, None);
    // let (mv2,stats2) = ex2.explore(&ts, n, false);
    eprintln!("mv2 = {:?}, (f3f5)", mv2.unwrap());
    stats2.print(t2.elapsed());

    // eprintln!("stats1 = {:?}", stats1);

    // let moves = vec![
    //     Move::Quiet { from: "C4".into(), to: "C3".into() },
    //     Move::Capture { from: "B3".into(), to: "B2".into() },
    // ];
    // ex.rank_moves_list(&ts, true, moves);

    // let mut ms0 = g.search_sliding(&ts, Rook, col);
    // let mut ms1 = g.search_sliding_iter(&ts, Rook, col).collect::<Vec<_>>();
    // ms0.sort_by(|a,b| a.partial_cmp(&b).unwrap());
    // ms1.sort_by(|a,b| a.partial_cmp(&b).unwrap());
    // eprintln!("ms0 = {:?}", ms0);
    // eprintln!("ms1 = {:?}", ms1);
    // assert_eq!(ms0, ms1);

    // let t = std::time::Instant::now();
    // let m = ex.explore(&ts, ex.depth);
    // eprintln!("m = {:?}", m);
    // // ex.rank_moves(&ts, true);
    // println!("explore done in {} seconds.", t.elapsed().as_secs_f64());

    // let mut ex1 = Explorer::new(g1.state.side_to_move, g1.clone(), n, stop.clone(), timesettings);
    // let mut ex2 = Explorer::new(g2.state.side_to_move, g2.clone(), n, stop, timesettings);

    // let m = ex.explore(&ts, ex.depth);
    // eprintln!("m w = {:?}", m);
    // ex.rank_moves(&ts, true, true);

    // println!("w:");
    // let moves0 = vec![
    //     Move::Capture { from: "D4".into(), to: "E5".into() },
    //     Move::Quiet { from: "D4".into(), to: "D5".into() },
    //     Move::Quiet { from: "H1".into(), to: "H2".into() },
    // ];
    // ex1.rank_moves_list(&ts, true, moves0);
    // // ex.rank_moves(&ts, true, true);

    // println!("\nb:");
    // let moves1 = vec![
    //     Move::Capture { from: "E5".into(), to: "D4".into() },
    //     Move::Quiet { from: "E5".into(), to: "E4".into() },
    //     Move::Quiet { from: "A8".into(), to: "A7".into() },
    // ];
    // ex2.rank_moves_list(&ts, true, moves1);
    // // ex2.rank_moves(&ts, true, true);

    // let m0 = ex1.explore(&ts, n);
    // eprintln!("m0 w = {:?}", m0);
    // let m1 = ex2.explore(&ts, n);
    // eprintln!("m1 b = {:?}", m1);

    // assert_eq!(m0, Some(Move::Capture { from: "D4".into(), to: "E5".into() }));
    // assert_eq!(m1, Some(Move::Capture { from: "E5".into(), to: "D4".into() }));


    // let m = ex2.explore(&ts, ex2.depth);
    // eprintln!("m b = {:?}", m);

    // let e = g2.evaluate(&ts);

    // let mw: Score = e.material_white.iter().sum();
    // let mb: Score = e.material_black.iter().sum();
    // let mw: Score = e.piece_positions_white.iter().sum();
    // let mb: Score = e.piece_positions_black.iter().sum();

    // eprintln!("mw = {:?}", e.piece_positions_white);
    // eprintln!("mb = {:?}", e.piece_positions_black);
    // // eprintln!("mw = {:?}", e.material_white);
    // // eprintln!("mb = {:?}", e.material_black);

    // let s = g.score_positions(&ts, Pawn, !White);
    // eprintln!("s = {:?}", s);

    // let ew = g.evaluate(&ts).sum_color(White);
    // let eb = g.evaluate(&ts).sum_color(Black);
    // eprintln!("sum w = {:?}", ew);
    // eprintln!("sum b = {:?}", eb);

    // let e = g.score_material(Pawn, White);
    // eprintln!("w = {:?}", e);
    // let e = g.score_material(Pawn, Black);
    // eprintln!("b = {:?}", e);

    // let moves = g.search_all(&ts, White);
    // for m in moves.clone() {
    //     eprintln!("m = {:?}", m);
    // }
    // eprintln!("moves.len() = {:?}", moves.get_moves_unsafe().len());

    // let maximizing = ex.side == g2.state.side_to_move;
    // eprintln!("maximizing = {:?}", maximizing);

    // let alpha = (None,i32::MIN);
    // let beta  = (None,i32::MAX);
    // let (_,score) = ex._ab_search(&ts, g2, n, 1, None, alpha, beta);
    // eprintln!("score = {:?}", score);

    // let from = "e5";
    // let to = "d6";
    // let other = "";
    // let m = g.convert_move(from, to, other);
    // eprintln!("m = {:?}", m);

}

/// Perft
fn main4(depth: Option<u64>) {

    // let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    // let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -";
    // let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ";
    // let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
    let fen = STARTPOS;

    // let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - "; // Position 2
    // let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - "; // Position 3
    // let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"; // Position 4
    // let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8  "; // Position 5

    // let fen = "rnbqkbnr/ppppp3/8/4N3/2BP1BQ1/4P1Pp/PPP4P/RN2K2R w KQkq - 0 1";

    let n = match depth {
        None    => 4,
        Some(d) => d,
    };

    // let ts = Tables::new();
    let ts = &_TABLES;
    let mut g = Game::from_fen(&ts, fen).unwrap();
    let _ = g.recalc_gameinfo_mut(&ts);
    // eprintln!("g = {:?}", g);

    // let ((t,t_sf),(_,_)) = test_stockfish(&ts, fen, n, true).unwrap();
    // // let (t,(_,_)) = test_stockfish(fen, n, false).unwrap();
    // println!("perft done in {} seconds.", t);
    // println!("stockfish took {} seconds.", t_sf);

    // let t0 = std::time::Instant::now();
    // let (tot,_) = g.perft(&ts, n);
    // // let (tot,vs) = g.perft2(&ts, n as Depth);
    // // eprintln!("n = {:?}", n);
    // let t1 = t0.elapsed().as_secs_f64();
    // println!("perft done in {} seconds.", t1);

    for d in 1..n+1 {
        let (tot,_) = g.perft(&ts, d);
        println!("depth {:>2}: {:>12} leaves", d, tot);
    }

    // for (d, n) in vs.iter().enumerate() {
    //     if *n > 0 {
    //         println!("depth {:>2}: {:>12} leaves", d, n);
    //     }
    // }
    // println!("total:    {:>12} leaves", tot);

    fn _print(x: i32) -> String {
        if x.abs() > 1_000_000 {
            format!("{:.1}M", x as f64 / 1_000_000.)
        } else if x > 1000 {
            format!("{:.1}k", x as f64 / 1000.)
        } else {
            format!("{}", x)
        }
    }

    // println!("speed: {} leaves / second", _print((tot as f64 / t1) as i32));

}

fn main2() {

    // println!("start");
    let now = std::time::Instant::now();

    // let game = Game::new();

    // let mut g = Game::new();
    // let mut g = Game::empty();

    let mut g = Game::empty();

    // g.insert_pieces_mut_unchecked(&vec![
    //     ("E1", King, White),
    //     ("E8", King, Black),
    //     ("B2", Pawn, White),
    //     ("C4", Pawn, Black),
    // ]);
    // // g.state.side_to_move = Black;
    // g.state.castling = Castling::new_with(true, true);

    // let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ";
    // let fen = "4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1";
    // let fen = "4k3/8/8/8/1Pp5/8/8/4K3 b - b3 0 1";
    // let fen = "r1bqkbnr/p1pppppp/n7/1p6/8/N7/PPPPPPPP/R1BQKBNR b Kkq - 1 3";
    // let fen = "8/2p5/3p4/KP5r/1R3pPk/8/4P3/8 b - g3";
    let fen = "r3k3/p1ppqpb1/bn2pnp1/3PN3/1p2P2r/2N1Q2p/PPPBBPPP/R3K2R w KQq - 2 2";

    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    // let fen = "1rbqkbnr/pppppppp/n7/8/3P4/8/PPPKPPPP/RNBQ1BNR w k - 3 3";

    let fen = "8/2p5/3p4/KP5r/1R3pP1/4P2k/8/8 b - - 0 2";

    let fen = "rnQq1k1r/pp3ppp/2pQ4/8/2B5/8/PPP1NnPP/RNB1K2R b KQ - 0 9";

    let fen = "8/8/8/p1Nk4/8/8/8/7K b - - 0 1";
    let fen = "rnbqkbnr/pppp1pp1/7p/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq e6 0 3";
    let fen = "8/8/3p3k/8/3R4/7K/3P4/8 w - - 0 1";

    let fen = "1k1r4/pp1b1R2/3q2p1/4p2p/2B5/4Q3/PPP2B2/2K5 w - - 0 2";
    let fen = "1k1r4/pp1b1R2/6pp/4p3/2B5/4Q3/PPP2B2/2Kq4 w - - 1 2";
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -";

    let fen = "B5KR/1r5B/6R1/2b1p1p1/2P1k1P1/1p2P2p/1P2P2P/3N1N2 w - - 0 1";

    // let mut g = Game::from_fen(&ts, "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ").unwrap();
    // let mut g = Game::from_fen(&ts, "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ").unwrap();
    // let mut g = Game::from_fen(&ts, "8/2p5/3p4/KP5r/1R3p1k/6P1/4P3/8 b - - 0 1").unwrap();
    // let mut g = Game::from_fen(&ts, fen).unwrap();

    // let mut g = Game::from_fen(&ts, "8/2p5/3p4/KP5r/1R3p1k/6P1/4P3/8 b - - 0 1").unwrap();
    // g.state.side_to_move = Black;

    // let mut g = Game::from_fen(&ts, "8/2p5/3p4/KP5r/1R3p1k/6P1/4P3/8 w - - 0 1").unwrap();

    let ts = Tables::new();
    // let mut g = Game::new();

    let mut g = Game::from_fen(&ts, fen).unwrap();

    let _ = g.recalc_gameinfo_mut(&ts);

    // // let m = Move::Quiet { from: "G2".into(), to: "G4".into() };
    // let m = Move::Castle {
    //     from:        "E1".into(),
    //     to:          "G1".into(),
    //     rook_from:   "H1".into(),
    //     rook_to:     "F1".into(),
    // };
    // let mut g = g.make_move_unchecked(&ts, &m).unwrap();
    // g.recalc_gameinfo_mut(&ts);

    eprintln!("{:?}", g);

    // let b = g.find_attacks_by_side(&ts, "B1".into(), !g.state.side_to_move, false);
    // eprintln!("b = {:?}", b);

    // let b = g.all_occupied() & !g.get(King, Black);
    // let b = !g.get(King, Black);
    // eprintln!("b = {:?}", b);

    // let moves = g._search_castles(&ts);
    // let moves = g.search_all(&ts, g.state.side_to_move);
    // let moves = g.search_sliding(&ts, Rook, g.state.side_to_move);
    // let moves = g._search_sliding_single(&ts, Rook, "D4".into(), g.state.side_to_move);
    // eprintln!("moves = {:?}", moves);

    // // let m = Move::Quiet { from: "C1".into(), to: "D1".into() };
    // let m = Move::Quiet { from: "C1".into(), to: "B1".into() };
    // let b = g.move_is_legal(&ts, &m);
    // eprintln!("b = {:?}", b);


    // eprintln!("moves.len() = {:?}", moves.get_moves_unsafe().len());
    // // eprintln!("moves.len() = {:?}", moves.len());
    // for m in moves.into_iter() {
    //     eprintln!("m = {:?}", m);
    //     // let b = g.move_is_legal(&ts, &m);
    //     // eprintln!("b = {:?}", b);
    // }

    // // // let moves = g._search_castles(&ts);
    // // let moves = g._search_pawns(None, &ts, White);
    // let moves = g._search_pawns(None, &ts, g.state.side_to_move);
    // let m = moves[1];
    // eprintln!("m = {:?}", m);

    // let m = Move::EnPassant { from: "D5".into(), to: "E6".into() };

    // let mut g = g.make_move_unchecked(&ts, &m).unwrap();
    // g.recalc_gameinfo_mut(&ts);

    // eprintln!("{:?}", g);

    // let depth = 3;
    // let (ns,cs) = g._perft(&ts, depth, true);
    // eprintln!("\nperft total    = {:?}", ns);
    // eprintln!("perft captures = {:?}", cs);

    // let moves = g.search_all(&ts, g.state.side_to_move);
    // let moves = g.search_sliding(Bishop, &ts, White);
    // let moves = g._search_pawns(None, &ts, Black);
    // let moves = g._search_promotions(&ts, None, White);
    // let m = moves[10];
    // eprintln!("m = {:?}", m);

    // let x = g.state.checkers;
    // // let x = g.move_is_legal(&ts, &m3);
    // eprintln!("x = {:?}", x);

    // eprintln!("moves.len() = {:?}", moves.len());
    // for m in moves.iter() {
    //     eprintln!("m = {:?}", m);
    // }

    // let (blockers,pinners) = g.find_slider_blockers(&ts, "A3".into());

    // // let c0 = g.get(King, White);
    // // let c0 = c0.bitscan().into();
    // let c0: Coord = "E1".into();
    // let (bs_w, ps_b) = g.find_slider_blockers(&ts, c0, White);

    // eprintln!("bs_w = {:?}", bs_w);
    // eprintln!("ps_b = {:?}", ps_b);

    // let b = g.state.pinners;
    // eprintln!("b = {:?}", b);

    // let c0: Coord = "A1".into();
    // let c1: Coord = "B3".into();
    // let b = ts.between(c0, c1);
    // eprintln!("b = {:?}", b);

    // let b = g.find_pins_absolute(&ts, White);
    // eprintln!("b = {:?}", b);

    // let moves = g.find_attacks_to(&ts, "A2".into(), Black);
    // let moves: Vec<Move> = moves.collect();

    // let k = g.find_attacks_by_side(&ts, "B1".into(), Black);
    // eprintln!("k = {:?}", k);

    // let moves = g.search_king(&ts, White);

    // let moves = g.search_sliding(Rook, &ts, White);

    // let g2 = g.make_move_unchecked(&m).unwrap();
    // eprintln!("{:?}", g2);

    // let moves = g.search_king(&ts, White);
    // let m = Move::Quiet { from: "A2".into(), to: "A1".into() };
    // let x = g.move_is_legal(&ts, &m);
    // eprintln!("x = {:?}", x);

    // let xs = g.find_attacks_by_side(&ts, "A1".into(), Black);
    // eprintln!("xs = {:?}", xs);

    // // let ms = g.search_all(&ts, White);
    // // let ms = g.search_king(White);
    // // let ms = g.search_knights(&ts, White);
    // // let ms = g.search_sliding(Rook, &ts, White);
    // // let ms = g.search_sliding(Bishop, &ts, White);
    // let ms = g.search_sliding(Queen, &ts, White);
    // // let ms = g.search_pawns(White);
    // // ms.sort_by(|a,b| a.partial_cmp(b).unwrap());

    // let b = BitBoard::new(&vec![
    //     Coord(1,1),
    //     Coord(1,0),
    //     Coord(0,1),
    //     Coord(2,1),
    //     Coord(1,2),
    //     Coord(1,3),

    //     Coord(7,0),
    //     Coord(0,7),

    //     // Coord(0,0),
    //     // Coord(7,7),

    // ]);

    // // let v: Vec<Coord> = (0..8).map(|x| Coord(1,x)).collect();
    // // let v: Vec<Coord> = (0..8).map(|x| Coord(x,x)).collect();
    // let v: Vec<Coord> = (0..8).map(|x| Coord(x,7-x)).collect();
    // let b = BitBoard::new(&v);
    // eprintln!("{:?}\n", b);

    // // let b = b.mirror_vert();
    // // let b = b.mirror_horiz();
    // // let b = b.flip_diag();
    // // let b = b.flip_antidiag();
    // // let b = b.rotate_180();
    // // let b = b.rotate_90_ccw();
    // // let b = b.rotate_45_cw();
    // let b = b.rotate_45_ccw();

    // eprintln!("{:?}", b);
    // // let x: u64 = 2u64.pow(63);
    // // eprintln!("b = {:0>64b}", x);
    // // eprintln!("b = {:0>64b}", x as i64);

    // let ms2 = vec![
    //     Move::Capture { from: Coord(0, 0), to: Coord(0, 2) },
    //     Move::Quiet { from: Coord(0, 0), to: Coord(0, 1) },
    //     Move::Quiet { from: Coord(0, 0), to: Coord(1, 0) },
    // ];
    // assert_eq!(ms, ms2);

    // let mut ms2: Vec<Move> = (0..9).into_iter()
    //     .zip(0..9)
    //     .map(|(x,y)| (Move::Quiet { from: Coord(1,1), to: Coord(x,y) }))
    //     .collect();
    // ms2.sort_by(|a,b| a.partial_cmp(b).unwrap());

    // println!("====");
    // for m in ms2.iter() {
    //     eprintln!("m = {:?}", m);
    // }

    // // let b = g.get(Pawn, White);
    // let b = g.get_piece(Pawn);
    // eprintln!("{:?}", b);

    // let c0 = Pawn.print(White);
    // eprintln!("c0 = {:?}", c0);

    // let b = BitBoard::new(&vec![Coord(1,0), Coord(2,0)]);

    // let (b0,x) = b.bitscan_reset();

    // let b = g.search_king(White);

    // let cs = b.serialize();

    // eprintln!("{:?}", b);

    // let k = std::mem::size_of::<Coord>();
    // eprintln!("k = {:?}", k);

    // let b = g.get(King, White);
    // assert_eq!(!BitBoard::mask_file(7).0, 0x7f7f7f7f7f7f7f7fu64);

    // let b = Tables::gen_knight_move(Coord(2,2));

    // let b = BitBoard(0xfcfcfcfcfcfcfcfcu64);
    // let b = BitBoard(!(BitBoard::mask_file(6).0 | BitBoard::mask_file(7).0));

    // let mut x: u64 = (!0u8) as u64 | (((!0u8) as u64) << 8);
    // let mut x: u64 = 15u8 as u64 | ((15u8 as u64) << 8);

    // let b = BitBoard(x);

    // let b = BitBoard::new(&vec![Coord(1,1)]);
    // let b = BitBoard::mask_rank(7);

    // let b = b.flip_diag();

    // let b0 = b.shift_unwrapped(D::W);
    // let b0 = BitBoard(b.0 << -1i64);

    // eprintln!("{:?}", b);

    // let b0: u8 = 0b0000_1111;
    // eprintln!("b0 = {:?}", b0);
    // let b1 = b0.reverse_bits();
    // eprintln!("b1 = {:0>8b}", b1);

    // println!("finished in {} seconds.", now.elapsed().as_secs_f64());

    // main2()
}

fn init_logger() {
    let cfg = ConfigBuilder::new()
            .set_time_level(LevelFilter::Off)
            .set_target_level(LevelFilter::Off)
            .set_thread_level(LevelFilter::Info)
        // .set_thread_level(LevelFilter::Off)
            .set_location_level(LevelFilter::Off)
            .build();

    let logfile = std::fs::File::create("test.log").unwrap();
    let log0 = WriteLogger::new(LevelFilter::Trace, cfg.clone(), logfile);

    // let log1 = TermLogger::new(LevelFilter::Trace, cfg.clone(), TerminalMode::Stderr, ColorChoice::Auto);
    let log1 = TermLogger::new(LevelFilter::Debug, cfg.clone(), TerminalMode::Stderr, ColorChoice::Auto);

    CombinedLogger::init(vec![
        log0,
        log1,
    ]).unwrap();

}

