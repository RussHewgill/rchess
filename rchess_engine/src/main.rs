#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]

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

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
    // .format(|buf, record| {
    //     writeln!(buf, "{}")
    // })
        .format_timestamp(None)
        .init();

    let fen = STARTPOS;

    let ts = Tables::new();
    let mut g = Game::from_fen(fen).unwrap();
    let _ = g.recalc_gameinfo_mut(&ts);

    let z = Zobrist::new(&ts, g);

    eprintln!("z = {:#8x}", z.0);

    // main6();
    // main7();
    // main5(); // search + eval position
    // main2();
    // main4(); // perft
    // main3(); // read from file and test

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
    // let mut g = Game::from_fen(fen).unwrap();

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

        let mut g = Game::from_fen(fen).unwrap();
        let _ = g.recalc_gameinfo_mut(&ts);

        let e = g.evaluate(&ts).sum();
        eprintln!("{} = {:?}", s, e);

    }

}

fn main7() {
    let fen = STARTPOS;
    // let fen = "rnbqkbnr/ppppp1pp/8/5P2/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2";
    // let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1";

    // let fen = "5r1k/4Qpq1/4p3/1p1p2P1/2p2P2/1p2P3/3P4/BK6 b - - 0 1";

    let n = 5;

    let ts = Tables::new();
    // let ts = Tables::_new(false);
    let mut g = Game::from_fen(fen).unwrap();
    let _ = g.recalc_gameinfo_mut(&ts);
    // eprintln!("g = {:?}", g);

    let stop = Arc::new(AtomicBool::new(false));
    let timesettings = TimeSettings::new_f64(10., 0.1);

    // let mut g = g.clone();
    // let ms = vec!["e2e4"];
    // for m in ms.iter() {
    //     let from = &m[0..2];
    //     let to = &m[2..4];
    //     let other = &m[4..];
    //     let mm = g.convert_move(from, to, other).unwrap();
    //     g = g.make_move_unchecked(&ts, &mm).unwrap();
    // }
    // eprintln!("g = {:?}", g);

    let ex = Explorer::new(g.state.side_to_move, g.clone(), n, stop.clone(), timesettings);

    // let moves = vec![
    //     Move::Quiet { from: "F7".into(), to: "F6".into() },
    //     Move::Quiet { from: "B3".into(), to: "B2".into() },
    // ];
    // ex.rank_moves_list(&ts, true, moves);

    // for m in moves.iter() {
    //     let g2 = self.game.make_move_unchecked(&ts, &m).unwrap();
    //     let alpha = i32::MIN;
    //     let beta  = i32::MAX;
    //     let score = self._ab_search(&ts, g2, self.depth, 1, alpha, beta, true);
    // }

    let t = std::time::Instant::now();
    let m = ex.explore(&ts, ex.depth);
    eprintln!("m = {:?}", m);
    // ex.rank_moves(&ts, true);
    println!("explore done in {} seconds.", t.elapsed().as_secs_f64());

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

    let n = 1;

    let ts = Tables::new();
    // let ts = Tables::_new(false);
    // let mut g = Game::from_fen(fen).unwrap();
    // let _ = g.recalc_gameinfo_mut(&ts);
    // eprintln!("g = {:?}", g);


    let mut g1 = Game::from_fen(fen1).unwrap();
    let _ = g1.recalc_gameinfo_mut(&ts);

    let mut g2 = Game::from_fen(fen2).unwrap();
    let _ = g2.recalc_gameinfo_mut(&ts);

    let stop = Arc::new(AtomicBool::new(false));
    let timesettings = TimeSettings::new_f64(10., 0.1);
    // let ex = Explorer::new(g.state.side_to_move, g.clone(), n, stop.clone(), timesettings);
    // let ex = Explorer::new(White, g.clone(), n, stop.clone(), timesettings);

    // let moves = vec![
    //     Move::Quiet { from: "D4".into(), to: "E5".into() },
    //     Move::Quiet { from: "D4".into(), to: "D5".into() },
    // ];
    // // ex.rank_moves_list(&ts, true, moves);

    // let t = std::time::Instant::now();
    // let m = ex.explore(&ts, ex.depth);
    // eprintln!("m = {:?}", m);
    // // ex.rank_moves(&ts, true);
    // println!("explore done in {} seconds.", t.elapsed().as_secs_f64());

    let ex1 = Explorer::new(g1.state.side_to_move, g1.clone(), n, stop.clone(), timesettings);
    let ex2 = Explorer::new(g2.state.side_to_move, g2.clone(), n, stop, timesettings);

    // let m = ex.explore(&ts, ex.depth);
    // eprintln!("m w = {:?}", m);
    // ex.rank_moves(&ts, true, true);

    println!("w:");
    let moves0 = vec![
        Move::Capture { from: "D4".into(), to: "E5".into() },
        Move::Quiet { from: "D4".into(), to: "D5".into() },
        Move::Quiet { from: "H1".into(), to: "H2".into() },
    ];
    ex1.rank_moves_list(&ts, true, moves0);
    // ex.rank_moves(&ts, true, true);

    println!("\nb:");
    let moves1 = vec![
        Move::Capture { from: "E5".into(), to: "D4".into() },
        Move::Quiet { from: "E5".into(), to: "E4".into() },
        Move::Quiet { from: "A8".into(), to: "A7".into() },
    ];
    ex2.rank_moves_list(&ts, true, moves1);
    // ex2.rank_moves(&ts, true, true);

    let m0 = ex1.explore(&ts, n);
    eprintln!("m0 w = {:?}", m0);
    let m1 = ex2.explore(&ts, n);
    eprintln!("m1 b = {:?}", m1);

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
    let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"; // Position 4
    // let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8  "; // Position 5

    // let fen = "8/2p5/n2p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";

    // let fen = "r3k2r/Pppp1ppp/1b3nbN/nPP5/BB2P3/q4N2/Pp1P2PP/R2Q1RK1 b kq - 0 1";
    // let fen = "r3k2r/Ppp2ppp/1b3nbN/nPPp4/BB2P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 2";

    let n = 5;

    let ts = Tables::new();
    let mut g = Game::from_fen(fen).unwrap();
    let _ = g.recalc_gameinfo_mut(&ts);
    eprintln!("g = {:?}", g);

    let ((t,t_sf),(_,_)) = test_stockfish(fen, n, true).unwrap();
    // let (t,(_,_)) = test_stockfish(fen, n, false).unwrap();
    println!("perft done in {} seconds.", t);
    println!("stockfish took {} seconds.", t_sf);

    // let t = std::time::Instant::now();
    // let _ = g.perft(&ts, n);
    // println!("perft done in {} seconds.", t.elapsed().as_secs_f64());

}

fn main3() {
    // let mut games = read_ccr_onehour("ccr_onehour.txt").unwrap();
    let mut games = read_epd("Midgames250.epd").unwrap();

    // for (fen,ms) in games.iter() {
    //     // eprintln!("fen, ms = {:?}: {:?}", fen, ms);
    //     eprintln!("ms = {:?}", ms);
    // }

    let g = &games[1];
    let games = vec![g.clone()];

    let n = 4;

    let ts = Tables::new();

    for (fen,m) in games.into_iter() {
        let mut g = Game::from_fen(&fen).unwrap();
        let _ = g.recalc_gameinfo_mut(&ts);
        eprintln!("g = {:?}", g);
        let stop = Arc::new(AtomicBool::new(false));
        let timesettings = TimeSettings::new_f64(10., 0.1);
        let ex = Explorer::new(g.state.side_to_move, g.clone(), n, stop, timesettings);
        let m0 = ex.explore(&ts, ex.depth);

        eprintln!("m0 = {:?}", m0.map(|x| x.to_long_algebraic()));
        // eprintln!("m0 = {:?}", m0.map(|x| x.to_algebraic(&g)));
        eprintln!("m0 = {:?}", m0.unwrap().to_algebraic(&g));
        // eprintln!("correct: {}", m.join(", "));
        eprintln!("correct: {}", m);
    }

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

    // let mut g = Game::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ").unwrap();
    // let mut g = Game::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ").unwrap();
    // let mut g = Game::from_fen("8/2p5/3p4/KP5r/1R3p1k/6P1/4P3/8 b - - 0 1").unwrap();
    // let mut g = Game::from_fen(fen).unwrap();

    // let mut g = Game::from_fen("8/2p5/3p4/KP5r/1R3p1k/6P1/4P3/8 b - - 0 1").unwrap();
    // g.state.side_to_move = Black;

    // let mut g = Game::from_fen("8/2p5/3p4/KP5r/1R3p1k/6P1/4P3/8 w - - 0 1").unwrap();

    let mut g = Game::from_fen(fen).unwrap();

    let ts = Tables::new();
    // let mut g = Game::new();

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


