#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_doc_comments)]

#![feature(destructuring_assignment)]

use std::collections::HashMap;
use std::str::FromStr;

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

const STARTPOS: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn main() {

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
    // // .format(|buf, record| {
    // //     writeln!(buf, "{}")
    // // })
    //     .format_timestamp(None)
    //     .init();

    // let fen = STARTPOS;
    // let ts = Tables::new();
    // let mut g = Game::from_fen(&ts, fen).unwrap();
    // let _ = g.recalc_gameinfo_mut(&ts);
    // let m0 = Move::Quiet { from: "B1".into(), to: "C3".into() };
    // let m1 = Move::Quiet { from: "C3".into(), to: "B1".into() };
    // let g2 = g.make_move_unchecked(&ts, &m0).unwrap();
    // let g3 = g2.make_move_unchecked(&ts, &m1).unwrap();
    // let z0 = Zobrist::new(&ts, g);
    // let z1 = Zobrist::new(&ts, g2);
    // let z2 = Zobrist::new(&ts, g3);
    // eprintln!("z0 = {:#8x}", z0.0);
    // eprintln!("z1 = {:#8x}", z1.0);
    // eprintln!("z2 = {:#8x}", z2.0);

    // // let s = std::mem::size_of::<Eval>();
    // let s = std::mem::size_of::<Game>();
    // eprintln!("s = {:?}", s);
    // // let s = u16::MAX;
    // // eprintln!("s = {:#8x}", s);

    // main6();
    // main5(); // search + eval position
    // main2();
    // main4(); // perft

    // for f in 0..8 {
    //     let b = BitBoard::mask_rank(f);
    //     // eprintln!("BitBoard({:0<16#8x})", b.0);
    //     // eprintln!("BitBoard({:#16x})", b.0);
    //     eprintln!("{} = BitBoard({:0>#016x})", f, b.0);
    // }

    // main8(); // eval testing
    // main7();
    main3(); // read from file and test

}

fn main8() {
    let fen = STARTPOS;
    let n = 4;

    let mut fens = vec![
        "k7/8/8/8/8/8/8/7K b - - 0 1", // nothing
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

    // fens.truncate(1);

    for (i,fen) in fens.iter().enumerate() {

        let mut g = Game::from_fen(&ts, fen).unwrap();
        let _ = g.recalc_gameinfo_mut(&ts);

        let d_pawns = g.count_pawns_doubled(&ts, White);
        eprintln!("doubled pawns {} = {:?}", i, d_pawns);

        let ee = g.evaluate(&ts);
        eprintln!("ee.sum() = {:?}", ee.sum());

        // let e = stockfish_eval(&fen, true).unwrap();
        let (_,e) = stockfish_eval(&fen, false).unwrap();
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
    let n = 4;

    // let fen = "rnbqkbnr/ppppp1pp/8/5P2/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2";
    // let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1";

    // let fen = "5r1k/4Qpq1/4p3/1p1p2P1/2p2P2/1p2P3/3P4/BK6 b - - 0 1";

    // // minimax = 869088, 2.08 s
    // // AB      = 92245,  0.24 s
    let fen = "r4q1k/5P1b/2p2n1P/p2p1P2/3P4/8/2PKN1Q1/6R1 w - - 1 34";
    // let n = 3;

    // // AB = 808182 leaves, 1.87 s
    // let fen = "7k/2pq2p1/6rp/1P2p3/2Qp1n2/P2P3P/R1P2PPK/3N2R1 b - - 0 28";
    // let n = 4;

    // let fen = "k7/8/8/8/8/6P1/8/7K w - - 0 1";
    // let n = 1;

    // let fen = "2rr3k/pp3pp1/1nnqbN1p/3pN3/2pP4/2P3Q1/PPB4P/R4RK1 w - - 0 1"; // WAC.001 = Qg6 = g3g6
    // let n = 3;

    // let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - "; // Position 3

    // let fen = "Q3b3/4bkp1/1q2np1p/NPp1p3/2P1P3/4BP1P/4B1P1/7K b - - 1 1"; // Correct = e6c7
    // let fen = "rnbqkb1r/pppp1ppp/8/4P3/6n1/7P/PPPNPPP1/R1BQKBNR b KQkq - 0 1"; // WAC.007, Ne3 = g4e3

    // let fen = "5k2/6pp/p1qN4/1p1p4/3P4/2PKP2Q/PP3r2/3R4 b - - 0 1"; // WAC.005, Qc4 = c6c4
    // let fen = "4r3/2R3pp/q2pkp2/4p3/4P1P1/4nQ1P/PP6/2K5 w - - 0 1"; // WAC.005, color reversed

    let n = 4;

    let ts = Tables::new();

    let mut g = Game::from_fen(&ts, fen).unwrap();
    let _ = g.recalc_gameinfo_mut(&ts);
    eprintln!("g = {:?}", g);

    let stop = Arc::new(AtomicBool::new(false));
    let timesettings = TimeSettings::new_f64(
        5.0,
        0.1);
    let mut ex = Explorer::new(g.state.side_to_move, g.clone(), n, stop.clone(), timesettings);

    // let moves = vec![
    //     Move::Quiet   { from: "C6".into(), to: "C4".into() },
    //     Move::Capture { from: "C6".into(), to: "D6".into() },
    //     Move::Quiet   { from: "C6".into(), to: "D7".into() },
    // ];

    // let t0 = std::time::Instant::now();
    // let (mv,stats) = ex.explore(&ts, ex.max_depth, false);
    // eprintln!("mv = {:?}", mv);
    // println!("explore depth done in {:.4} seconds.", t0.elapsed().as_secs_f64());
    // stats.print(t0.elapsed());
    // println!("\n");
    // let t0 = std::time::Instant::now();
    // let (mv,stats) = ex.explore(&ts, ex.max_depth, true);
    // eprintln!("mv = {:?}", mv);
    // println!("explore iter  done in {:.4} seconds.", t0.elapsed().as_secs_f64());
    // stats.print(t0.elapsed());

    // let k = 4;
    let k = 2;
    let mut t: std::time::Duration = std::time::Duration::from_secs(0);
    for q in 0..k {
        let t0 = std::time::Instant::now();

        let (mv,stats) = ex.explore(&ts, ex.max_depth, true);
        // let (mv,stats) = ex.explore(&ts, ex.max_depth, false);
        // let (mv,stats) = ex.iterative_deepening(&ts, false);
        println!("\nm #{} = {:?}", q, mv);

        // let (mvs,stats) = ex._iterative_deepening(&ts, false, moves.clone());
        // let (mvs,stats) = ex.iterative_deepening(&ts, true);
        // let (mvs,stats) = ex.rank_moves_list(&ts, true, moves.clone());

        // for (m,s) in mvs.iter() {
        //     eprintln!("{:>8} = {:?}", s, m);
        // }

        // eprintln!("\nbest move = {:?}", mvs.get(0).unwrap());

        // stats.print(t0.elapsed());

        // println!("explore #{} done in {} seconds.", q, t0.elapsed().as_secs_f64());

        // let m = ex.explore(&ts, ex.max_depth);
        // eprintln!("m #{} = {:?}", q, m);

        t += t0.elapsed();
    }
    println!("explore {} times, done in avg {:.3} seconds.", k, t.as_secs_f64() / k as f64);

    {
        // let s = ex.trans_table.with(|m| m.map_r.len());
        let s = ex.trans_table.with(|m| m.map.len());
        // let h = ex.trans_table.hits();
        // let m = ex.trans_table.misses();
        // let k = ex.trans_table.leaves();
        println!("");
        println!("tt len = {:?}", s);
        // println!("hits   = {:?}", h);
        // println!("misses = {:?}", m);
        // println!("leaves = {:?}", k);
    }

    // let ms0 = vec![
    //     "e2e4",
    //     "e7e5",
    //     "g1f3",
    // ];
    // let ms0 = ms0.split(" ");
    // let mut g2 = g.clone();
    // for m in ms0.into_iter() {
    //     let from = &m[0..2];
    //     let to = &m[2..4];
    //     let other = &m[4..];
    //     let mm = g2.convert_move(from, to, other).unwrap();
    //     g2 = g2.make_move_unchecked(&ts, &mm).unwrap();
    // }
    // eprintln!("g2 = {:?}", g2);
    // eprintln!("hash0 = {:?}", g2.zobrist);

    // let moves = vec![
    //     Move::Quiet { from: "E2".into(), to: "E4".into() },
    //     Move::Quiet { from: "D2".into(), to: "D4".into() },
    //     // Move::Quiet { from: "H2".into(), to: "H3".into() },
    // ];
    // ex.rank_moves_list(&ts, true, moves);
    // // ex.rank_moves(&ts, true);

    // let t = std::time::Instant::now();
    // let m = ex.explore(&ts, ex.depth);
    // eprintln!("m = {:?}", m);
    // // ex.rank_moves(&ts, true);
    // println!("explore done in {} seconds.", t.elapsed().as_secs_f64());


}

fn main3() {
    // let mut games = read_ccr_onehour("ccr_onehour.txt").unwrap();
    // let mut games = read_epd("Midgames250.epd").unwrap();
    let mut games = read_epd("WAC.epd").unwrap();

    // for (fen,ms) in games.iter() {
    //     // eprintln!("fen, ms = {:?}: {:?}", fen, ms);
    //     eprintln!("ms = {:?}", ms);
    // }

    // let g = &games[0];
    // let games = vec![g.clone()];
    // games.truncate(10);

    let n = 5;

    let ts = Tables::new();

    let mut total = (0,0);
    let t0 = std::time::Instant::now();


    for (i,(fen,m)) in games.into_iter().enumerate() {
        let i = i + 1;
        let mut g = Game::from_fen(&ts, &fen).unwrap();
        let _ = g.recalc_gameinfo_mut(&ts);
        // eprintln!("g = {:?}", g);
        let stop = Arc::new(AtomicBool::new(false));
        let timesettings = TimeSettings::new_f64(
            5.0,
            0.1,
        );
        let mut ex = Explorer::new(g.state.side_to_move, g.clone(), n, stop, timesettings);

        // let t0 = std::time::Instant::now();
        // let m0 = ex.explore(&ts, ex.max_depth);
        // let (m0,stats) = ex.iterative_deepening(&ts, true);
        let (m0,stats) = ex.iterative_deepening(&ts, false);
        // println!("explore done in {:.3} seconds.", t0.elapsed().as_secs_f64());

        // eprintln!("m0 = {:?}", m0.map(|x| x.to_long_algebraic()));
        // // eprintln!("m0 = {:?}", m0.map(|x| x.to_algebraic(&g)));
        // eprintln!("m0 = {:?}", m0.unwrap().to_algebraic(&g));
        // // eprintln!("correct: {}", m.join(", "));
        // eprintln!("correct: {}", m);

        let mv = m0.get(0).unwrap().0.to_algebraic(&g);

        let mv0 = m[0].replace("+", "");
        if mv0 == mv {
            // println!("#{:>2}: Correct", i);
            total.0 += 1;
            total.1 += 1;
        } else {
            println!("#{:>2}: Wrong, Correct: {}, engine: {}", i, m[0], mv);
            // println!("Correct        Engine");
            // println!("{:<8}       {}", m[0], mv);
            total.1 += 1;
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
    let (mv1,stats1) = ex1.explore(&ts, n, true);
    // let (mv1,stats1) = ex1.explore(&ts, n, false);
    eprintln!("mv1 = {:?}, (c6c4)", mv1.unwrap());
    stats1.print(t1.elapsed());

    print!("\n");

    let t2 = std::time::Instant::now();
    let (mv2,stats2) = ex2.explore(&ts, n, true);
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

fn main4() {

    // let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    // let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -";
    // let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ";
    // let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
    let fen = STARTPOS;

    // let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - "; // Position 2
    // let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - "; // Position 3
    // let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"; // Position 4
    // let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8  "; // Position 5

    let fen = "rnbqkbnr/ppppp3/8/4N3/2BP1BQ1/4P1Pp/PPP4P/RN2K2R w KQkq - 0 1";

    let n = 4;

    let ts = Tables::new();
    let mut g = Game::from_fen(&ts, fen).unwrap();
    let _ = g.recalc_gameinfo_mut(&ts);
    // eprintln!("g = {:?}", g);

    let ((t,t_sf),(_,_)) = test_stockfish(&ts, fen, n, true).unwrap();
    // let (t,(_,_)) = test_stockfish(fen, n, false).unwrap();
    println!("perft done in {} seconds.", t);
    println!("stockfish took {} seconds.", t_sf);

    // let t = std::time::Instant::now();
    // let _ = g.perft(&ts, n);
    // println!("perft done in {} seconds.", t.elapsed().as_secs_f64());

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


