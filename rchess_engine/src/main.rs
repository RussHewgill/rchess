#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]

pub mod types;
pub mod bitboard;
pub mod coords;
pub mod tables;
pub mod game;

pub mod evaluate;
pub mod search;

use crate::types::*;
use crate::search::*;
use crate::tables::*;

use log::{debug, error, log_enabled, info, Level};
use gag::Redirect;

fn main() {

    // println!("start");
    let now = std::time::Instant::now();

    // let game = Game::new();

    // let mut g = Game::new();
    let mut g = Game::empty();

    // g.insert_piece_mut_unchecked(Coord(1,1), Pawn, White);
    // g.insert_piece_mut_unchecked(Coord(2,1), Pawn, White);
    // g.insert_piece_mut_unchecked(Coord(3,1), Pawn, White);
    // g.insert_piece_mut_unchecked(Coord(2,2), Pawn, Black);

    g.insert_piece_mut_unchecked(Coord(1,1), Rook, White);
    // g.insert_piece_mut_unchecked(Coord(3,1), Pawn, White);
    // g.insert_piece_mut_unchecked(Coord(1,3), Pawn, Black);

    // eprintln!("{:?}", g);

    // let c = Coord(3,3);

    let ts = Tables::new();
    // let b = ts.rook_moves.get(&Coord(3,3)).unwrap();
    // let b = Tables::gen_rook_move(c);

    let ms = g.search_rooks(&ts, White);
    // let ms = g.search_pawns(White);

    // for m in ms.iter() {
    //     eprintln!("m = {:?}", m);
    // }

    g.insert_piece_mut_unchecked(Coord(1,0), Pawn, White);
    g.insert_piece_mut_unchecked(Coord(3,0), Pawn, Black);

    let b = g.get_piece(Pawn);
    let (x,b) = b.bitscan_rev_reset();

    eprintln!("x = {:?}", x);
    eprintln!("{:?}", b);

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



