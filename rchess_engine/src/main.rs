#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]

pub mod types;
pub mod bitboard;
pub mod coords;
pub mod tables;
pub mod game;

pub mod search;

use crate::types::*;
use crate::search::*;
use crate::tables::*;

use log::{debug, error, log_enabled, info, Level};
use gag::Redirect;

fn main() {

    println!("start");
    let now = std::time::Instant::now();

    // let game = Game::new();

    // let mut a: u8 = 0b0000_1000;
    // eprintln!("a = {:?}", a);
    // eprintln!("a = 0b{:08b}", a);
    // a <<= 1;
    // eprintln!("a = 0b{:08b}", a);

    // let mut b = BitBoard(0);
    // b.flip(Coord(2,0));
    // // let b: u8 = 0b0000_0010;
    // eprintln!("board = {:08b}", &b.0);

    // for k in 0..3 {
    //     println!("====");
    //     let k0 = b.get(Coord(k,0));
    //     eprintln!("k0 = {:?}", k0);
    // }

    // let s = BitBoard::index_square(Coord(0, 0));
    // // let s = 2;
    // let r = BitBoard::index_rank(s);
    // let f = BitBoard::index_file(s);

    // eprintln!("r = {:?}", r);
    // eprintln!("f = {:?}", f);

    let mut g = Game::new();

    let b = g.search_king(White);

    // let b = g.get(Pawn, Black);

    // let k = std::mem::size_of::<Coord>();
    // eprintln!("k = {:?}", k);

    // let b = g.get(King, White);
    // let b = BitBoard::new(&vec![Coord(1,1)]);
    // assert_eq!(!BitBoard::mask_file(7).0, 0x7f7f7f7f7f7f7f7fu64);

    // let b = Tables::gen_knight_move(Coord(2,2));

    // let b = BitBoard(0xfcfcfcfcfcfcfcfcu64);
    // let b = BitBoard(!(BitBoard::mask_file(6).0 | BitBoard::mask_file(7).0));

    eprintln!("{:?}", b);

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



