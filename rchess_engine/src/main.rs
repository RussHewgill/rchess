#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]

use std::str::FromStr;

// use crate::lib::*;
use rchess_engine_lib::types::*;
use rchess_engine_lib::search::*;
use rchess_engine_lib::tables::*;
use rchess_engine_lib::parsing::*;

use log::{debug, error, log_enabled, info, Level};
use gag::Redirect;

fn main() {

    // println!("start");
    let now = std::time::Instant::now();

    // let game = Game::new();

    // let mut g = Game::new();
    // let mut g = Game::empty();

    // g.insert_piece_mut_unchecked(Coord(1,1), Pawn, White);
    // g.insert_piece_mut_unchecked(Coord(2,1), Pawn, White);
    // g.insert_piece_mut_unchecked(Coord(3,1), Pawn, White);
    // g.insert_piece_mut_unchecked(Coord(2,2), Pawn, Black);

    // g.insert_piece_mut_unchecked(Coord(1,1), Rook, White);
    // g.insert_piece_mut_unchecked(Coord(3,1), Pawn, White);
    // g.insert_piece_mut_unchecked(Coord(1,3), Pawn, Black);

    // g.insert_piece_mut_unchecked(Coord(0,0), Rook, White);
    // g.insert_piece_mut_unchecked(Coord(2,0), Pawn, White);
    // g.insert_piece_mut_unchecked(Coord(0,2), Pawn, Black);

    // g.insert_piece_mut_unchecked(Coord(2,2), King, White);
    // g.insert_piece_mut_unchecked(Coord(3,2), Pawn, White);
    // g.insert_piece_mut_unchecked(Coord(2,3), Pawn, Black);

    // g.insert_piece_mut_unchecked(Coord(3,3), Bishop, White);
    // g.insert_piece_mut_unchecked(Coord(5,1), Pawn, White);
    // g.insert_piece_mut_unchecked(Coord(4,4), Pawn, White);
    // g.insert_piece_mut_unchecked(Coord(2,4), Pawn, White);
    // g.insert_piece_mut_unchecked(Coord(1,1), Pawn, Black);

    let mut g = Game::empty();

    // g.insert_pieces_mut_unchecked(&vec![
    //     ("C7", Pawn, Black),
    //     ("D6", Pawn, Black),
    //     ("F4", Pawn, Black),
    //     ("H4", Rook, Black),
    //     ("H3", King, Black),
    //     ("A5", King, White),
    //     ("B5", Pawn, White),
    //     ("B4", Rook, White),
    //     ("B5", Pawn, White),
    // ]);

    // g.insert_pieces_mut_unchecked(&vec![
    //     ("D4", Queen, White),
    //     ("B2", Pawn, White),
    //     ("D2", Pawn, White),
    //     ("F2", Pawn, White),
    //     ("B4", Pawn, White),
    //     ("F4", Pawn, White),
    //     ("B6", Pawn, White),
    //     ("D6", Pawn, White),
    //     ("F6", Pawn, White),
    // ]);

    // g.insert_pieces_mut_unchecked(&vec![
    //     ("H2", Rook, Black),
    //     ("A2", King, White),
    //     ("B2", Pawn, White),
    //     // ("A8", Rook, Black),
    //     // ("A1", King, White),
    //     // ("A2", Pawn, White),
    // ]);

    g.insert_pieces_mut_unchecked(&vec![
        ("A1", King, White),
        // ("C3", Pawn, White),
        // ("C3", Pawn, Black),
        ("D4", Pawn, White),
        ("H8", Bishop, Black),
    ]);

    let ts = Tables::new();
    // let g = Game::new();

    // let g = Game::from_fen("8/8/8/8/1nb5/1p6/7r/K7 w - - 0 1").unwrap();
    // let g = Game::from_fen("8/8/8/8/8/1p6/8/K7 w - - 0 1").unwrap();
    // let g = Game::from_fen("8/8/8/8/8/1p6/7r/K7 w - - 0 1").unwrap();
    // let g = Game::from_fen("8/8/8/8/8/k7/8/K7 w - - 0 1").unwrap();
    // let g = Game::from_fen("8/8/8/8/2b5/k7/8/K7 w - - 0 1").unwrap();

    // let g = Game::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -").unwrap();

    // let mut g = Game::from_fen("8/2p5/3p4/KP5r/1R3p1k/6P1/4P3/8 w - - 0 1").unwrap();
    // g.state.side_to_move = Black;

    eprintln!("{:?}", g);

    let b = g.find_pins_absolute(&ts, White);

    eprintln!("b = {:?}", b);

    // let moves = g.search_all(&ts, g.state.side_to_move);

    // let moves = g.find_attacks_to(&ts, "A2".into(), Black);
    // let moves: Vec<Move> = moves.collect();

    // let k = g.find_attacks_by_side(&ts, "B1".into(), Black);
    // eprintln!("k = {:?}", k);

    // let moves = g.search_king(&ts, White);

    // let moves = g.search_sliding(Rook, &ts, White);

    // let g2 = g.make_move_unchecked(&m).unwrap();
    // eprintln!("{:?}", g2);

    // eprintln!("moves.len() = {:?}", moves.len());
    // for m in moves.iter() {
    //     eprintln!("m = {:?}", m);
    // }

    // let b = g.find_xray_bishop(&ts, "H8".into(), Black);

    // let c1: Coord = "A1".into();
    // let c0: Coord = "H8".into();

    // let b = g.obstructed(&ts, c0, c1);

    // let b = g.find_xray_rook(&ts, "A2".into(), Black, Some(White));
    // let b = g.find_xray_rook(&ts, "A2".into(), Black, None);

    // let depth = 1;
    // let (ns,cs) = g.perft(&ts, depth, true);
    // eprintln!("\nperft total    = {:?}", ns);
    // eprintln!("perft captures = {:?}", cs);

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

fn main2() {

    let logpath = "log.log";
    use std::fs::OpenOptions;
    let logfile = OpenOptions::new()
        .truncate(true)
        .read(true)
        .create(true)
        .write(true)
        .open(logpath)
        .unwrap();

    let err_redirect = Redirect::stderr(logfile).unwrap();


    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
    // .format(|buf, record| {
    //     writeln!(buf, "{}")
    // })
        .format_timestamp(None)
        .init();


}



