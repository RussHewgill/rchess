#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]

pub mod types;
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

    let mut b = BitBoard(0);
    b.flip(Coord(2,0));
    // let b: u8 = 0b0000_0010;
    eprintln!("board = {:08b}", &b.0);

    for k in 0..3 {
        println!("====");

        // let x = 1 << k;
        // eprintln!("x = {:08b}", x);
        // let y = b & x;
        // eprintln!("y = {:08b}", y);

        let k0 = b.get(Coord(k,0));
        eprintln!("k0 = {:?}", k0);

        // let k = 1 << p;
        // eprintln!("k0 = {:08b}", k);

        // let k = k & b.0;
        // eprintln!("k1 = {:08b}", k);

        // eprintln!("k0 = {:?}", k0);
    }


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



