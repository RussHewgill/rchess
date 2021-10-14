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


    // main5() // search + eval position
    // main2();
    // main4(); // perft
    main3(); // read from file and test

    // let b = BitBoard(b0);
    // eprintln!("b = {:?}", b);

    // // let (mut magics, table) = Tables::gen_magics_rook();
    // let t = std::time::Instant::now();
    // // let ((magics, table),_) = Tables::gen_magics();
    // // let (magics_b, table_b) = Tables::_gen_magics(true).unwrap();
    // let (magics_r, table_r) = Tables::_gen_magics(false).unwrap_err();
    // println!("magics done in {} seconds.", t.elapsed().as_secs_f64());

    // let c0: Coord = "A1".into();
    // let sq: u32 = c0.into();
    // let occ = BitBoard::new(&["D4","A5"]);
    // // let b = Tables::attacks_bishop(c0, occ, magics_b, table_b);
    // let b = Tables::attacks_rook(c0, occ, magics_r, table_r);
    // eprintln!("b = {:?}", b);

}

fn main5() {

    // let fen = "8/1N1p4/4k3/2p5/8/3p4/8/7K w - - 0 1";
    // let fen = "8/1N1p4/4k3/r1p5/8/3p4/8/7K w - - 0 1";
    // let fen = "8/3p4/8/2Nk4/8/3p4/8/7K w - - 1 2";
    // let fen = "3p4/1N2k3/8/2p5/8/8/8/7K w - - 0 1";
    // let fen = "8/1N6/8/p1pk4/8/8/8/7K w - - 0 1";
    // let fen = "8/8/8/p1pk4/1P6/8/8/7K w - - 0 1";
    // let fen = "8/1N6/8/p1pk4/8/8/8/7K w - - 0 1";

    // let fen = "8/8/3k4/8/8/4K3/8/Q6R w - - 0 1"; // Mate in 3
    // let fen = "8/8/7R/2k5/8/4K3/8/Q7 w - - 0 1"; // Mate in 2
    // let fen = "8/8/7R/3k4/Q7/4K3/8/8 w - - 0 1"; // Mate in 1
    // let fen = "8/8/7R/3k4/3Q4/4K3/8/8 w - - 0 1"; // Mate

    // let fen = "8/8/7R/3k4/8/4K3/8/Q7 w - - 0 1";
    // let fen = "6k1/p4pp1/7p/1P6/8/P4QKP/5PP1/2qRr2q w - - 0 35";
    // let fen = "1k1r4/pp1b1R2/3q2pp/4p3/2B5/4Q3/PPP2B2/2K5 b - -";

    // let fen = "1k6/6pR/1K6/8/8/8/8/8 w - - 0 1"; // Mate in 1
    // let fen = "1k5R/6p1/1K6/8/8/8/8/8 b - - 1 1"; // Mate

    // let fen = "3k4/2pP4/2P5/4R3/4P3/8/8/6K1 w - - 0 1";
    // let fen = "3k4/2pP4/2P4p/4R1p1/4P3/8/8/6K1 w - - 0 1";
    // let fen = "1k1b3r/p1pPp3/1pP1P2p/4R1p1/4P3/8/8/6K1 w - - 0 1";

    // let fen = "5r1k/4Qpq1/4p3/1p1p2P1/2p2P2/1p2P3/3P4/BK6 b - -"; // Horizon Effect

    let fen = "2bq2k1/5p1n/2p1r2Q/2Pp2N1/3b4/7P/5PP1/6K1 w - - 0 25";

    // let fen = STARTPOS;

    let n = 4;

    let ts = Tables::new();
    // let ts = Tables::_new(false);
    let mut g = Game::from_fen(fen).unwrap();
    let _ = g.recalc_gameinfo_mut(&ts);
    eprintln!("g = {:?}", g);

    let ex = Explorer::new(g.state.side_to_move, g.clone(), n);

    // let b = g.state.checkers;
    // eprintln!("b = {:?}", b);

    // let m = Move::Capture { from: "G5".into(), to: "H7".into() };
    // let g2 = g.make_move_unchecked(&ts, &m).unwrap();
    // eprintln!("g2 = {:?}", g2);
    // let moves = g2.search_all(&ts, g2.state.side_to_move);
    // eprintln!("moves = {:?}", moves);
    // let ex = Explorer::new(g2.state.side_to_move, g2.clone(), n);

    // // let s = g.score_material();
    // let s = g.evaluate(&ts);
    // eprintln!("score = {:?}", s);

    // let moves = vec![
    //     Move::Quiet { from: "E5".into(), to: "G5".into() },
    //     Move::Quiet { from: "E5".into(), to: "D5".into() },
    // ];
    // let s0 = ex.ab_search(&ts, moves[0]);
    // eprintln!("s0 = {:?} : {:?}", s0, moves[0]);
    // let s1 = ex.ab_search(&ts, moves[1]);
    // eprintln!("s1 = {:?} : {:?}", s1, moves[1]);

    // ex.rank_moves(&ts, true);
    // ex.rank_moves_list(&ts, true, moves);

    let t = std::time::Instant::now();
    let m = ex.explore(&ts, ex.depth);
    eprintln!("m = {:?}", m);
    // ex.rank_moves(&ts, true);
    println!("perft done in {} seconds.", t.elapsed().as_secs_f64());

    // let k = g.evaluate(&ts, White);
    // eprintln!("k = {:?}", k);

}

fn main4() {

    // let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    // let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -";
    // let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ";
    // let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
    let fen = STARTPOS;
    let fen = "1k1r4/pp1b1R2/3q2pp/4p3/2B5/4Q3/PPP2B2/2K5 b - - 0 1";
    let fen = "1kbr4/pp3R2/3q2pp/4p3/2B5/4Q3/PPP2B2/2K5 w - - 1 2";

    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -";
    let fen = "r3k2r/p1ppqpb1/bn2pQp1/3PN3/1p2P3/2N4p/PPPBBPPP/R3K2R b KQkq - 0 1";
    let fen = "r2qk2r/p1pp1pb1/bn2pQp1/3PN3/1p2P3/2N4p/PPPBBPPP/R3K2R w KQkq - 1 2";

    let n = 4;

    let ts = Tables::new();
    let mut g = Game::from_fen(fen).unwrap();
    let _ = g.recalc_gameinfo_mut(&ts);
    eprintln!("g = {:?}", g);

    let (t,(_,_)) = test_stockfish(fen, n, true).unwrap();
    // let (t,(_,_)) = test_stockfish(fen, n, false).unwrap();
    println!("perft done in {} seconds.", t);

    // let (bad_move, bad_fen) = find_move_error(&ts, fen, n, None).unwrap().unwrap();
    // eprintln!("bad_fen  = {:?}", bad_fen);
    // eprintln!("bad_move = {:?}", bad_move);
    // let g = Game::from_fen(&bad_fen).unwrap();
    // eprintln!("g = {:?}", g);

}

fn main3() {
    let mut games = read_ccr_onehour("ccr_onehour.txt").unwrap();

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

        let ex = Explorer::new(g.state.side_to_move, g.clone(), n);
        let m0 = ex.explore(&ts, ex.depth);

        eprintln!("m0 = {:?}", m0.map(|x| x.to_long_algebraic()));
        // eprintln!("m0 = {:?}", m0.map(|x| x.to_algebraic(&g)));
        eprintln!("m0 = {:?}", m0.unwrap().to_algebraic(&g));
        eprintln!("correct: {}", m.join(", "));
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
    let moves = g.search_all(&ts, g.state.side_to_move);
    // let moves = g.search_sliding(&ts, Rook, g.state.side_to_move);
    // let moves = g._search_sliding_single(&ts, Rook, "D4".into(), g.state.side_to_move);
    // eprintln!("moves = {:?}", moves);

    // // let m = Move::Quiet { from: "C1".into(), to: "D1".into() };
    // let m = Move::Quiet { from: "C1".into(), to: "B1".into() };
    // let b = g.move_is_legal(&ts, &m);
    // eprintln!("b = {:?}", b);

    eprintln!("moves.len() = {:?}", moves.get_moves_unsafe().len());
    // eprintln!("moves.len() = {:?}", moves.len());
    for m in moves.into_iter() {
        eprintln!("m = {:?}", m);
        // let b = g.move_is_legal(&ts, &m);
        // eprintln!("b = {:?}", b);
    }

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


