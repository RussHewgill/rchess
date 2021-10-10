#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]

use std::collections::HashMap;
use std::str::FromStr;

// use crate::lib::*;
use rchess_engine_lib::types::*;
use rchess_engine_lib::search::*;
use rchess_engine_lib::tables::*;
use rchess_engine_lib::parsing::*;
use rchess_engine_lib::util::*;

use log::{debug, error, log_enabled, info, Level};
use gag::Redirect;

fn main() {

    // let logpath = "log.log";
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


    // main2();
    main4();

    // main3();

}


fn main4() {

    // let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ";
    // let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R4K1R b kq - 1 1";
    // let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R2K3R b kq - 1 1";
    // let fen = "1r2k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K1R1 w KQkq -";

    let n = 1;

    let ts = Tables::new();
    // let mut g = Game::from_fen(fen).unwrap();
    // g.recalc_gameinfo_mut(&ts);
    // eprintln!("g = {:?}", g);

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
    let games = read_json_fens("perft_fens.txt").unwrap();

    let mut games = games;
    games.truncate(1);

    for (depth,nodes,fen) in games.into_iter() {
        let depth = 4;
        println!("FEN: {}", &fen);
        let (done, ((ns0,nodes0),(ns1,nodes1))) = test_stockfish(&fen, depth, false).unwrap();
        println!("perft depth {} done in {}", depth, done);

        if ns0 == ns1 {
            eprintln!("rchess, stockfish = {:>2} / {:>2}", ns0, ns1);
        } else {
            eprintln!("rchess, stockfish = {:>2} / {:>2} / failed ({})",
                      ns0, ns1, ns0 as i64 - ns1 as i64);
        }

    }

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

    g.recalc_gameinfo_mut(&ts);

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

    // // // let moves = g._search_castles(&ts);
    // // let moves = g._search_pawns(None, &ts, White);
    // let moves = g._search_pawns(None, &ts, g.state.side_to_move);
    // let m = moves[1];
    // eprintln!("m = {:?}", m);
    // let mut g = g.make_move_unchecked(&ts, &m).unwrap();
    // g.recalc_gameinfo_mut(&ts);

    // let ep = g.state.en_passant;
    // eprintln!("ep = {:?}", ep);

    // eprintln!("{:?}", g);

    // // let b = g.state.checkers.unwrap();
    // let b = g.find_attackers_to(&ts, "H4".into());
    // eprintln!("b = {:?}", b);

    // let depth = 3;
    // let (ns,cs) = g._perft(&ts, depth, true);
    // eprintln!("\nperft total    = {:?}", ns);
    // eprintln!("perft captures = {:?}", cs);

    // // let moves = g.search_all(&ts, g.state.side_to_move);
    // let moves = g.search_sliding(Bishop, &ts, White);
    // let m = moves[1];
    // eprintln!("m = {:?}", m);

    // let x = g.move_is_legal(&ts, &m);
    // eprintln!("x = {:?}", x);

    // let x = g.find_attacks_by_side(&ts, "C8".into(), White, false);
    // eprintln!("x = {:?}", x);

    // eprintln!("moves.len() = {:?}", moves.len());
    // for m in moves.iter() {
    //     eprintln!("m = {:?}", m);
    // }

    // let (blockers,pinners) = g.find_slider_blockers(&ts, "A3".into());

    // let c0 = g.get(King, White);
    // let c0 = c0.bitscan().into();
    let c0: Coord = "E1".into();
    let (bs_w, ps_b) = g.find_slider_blockers(&ts, c0, White);

    eprintln!("bs_w = {:?}", bs_w);
    eprintln!("ps_b = {:?}", ps_b);

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


