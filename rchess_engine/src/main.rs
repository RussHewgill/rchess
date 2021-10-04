#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]

pub mod types;
pub mod bitboard;
pub mod coords;

pub mod search;

use crate::types::*;

use log::{debug, error, log_enabled, info, Level};
use gag::Redirect;

fn main() {

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

    // let b = BitBoard::new(&vec![Coord(1,1)]);
    // eprintln!("{:?}", b);

    let c0 = Coord(1,1);

    let c1 = c0.shift(D::N);

    eprintln!("c1 = {:?}", c1);

    // eprintln!("b = {:64b}", b.0);
    // debug_bitboard(b);

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



