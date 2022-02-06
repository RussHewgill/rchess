#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]

#![allow(clippy::all)]

mod sprt;

use rchess_engine_lib::alphabeta::ABResult;
use rchess_engine_lib::types::*;
use rchess_engine_lib::tables::*;
use rchess_engine_lib::explore::*;
use rchess_engine_lib::evaluate::*;

// use sprt::*;

use chrono::{Datelike,Timelike};
use std::io::Write;
use log::{debug, error, log_enabled, info, Level};
use simplelog::*;
use gag::Redirect;

fn main() {
    init_logger();


}

fn init_logger() {
    let now = chrono::Local::now();
    let mut logpath = format!(
        "/home/me/code/rust/rchess/logs/log_tuner_{:0>4}-{:0>2}-{:0>2}_{:0>2}:{:0>2}:{:0>2}-1.log",
        now.year(), now.month(), now.day(),
        now.hour(), now.minute(), now.second());
    if std::path::Path::new(&logpath).exists() {
        logpath = format!(
            "/home/me/code/rust/rchess/logs/log_tuner_{:0>4}-{:0>2}-{:0>2}_{:0>2}:{:0>2}:{:0>2}-2.log",
            now.year(), now.month(), now.day(),
            now.hour(), now.minute(), now.second());
    };
    let mut logfile = std::fs::OpenOptions::new()
        .truncate(true)
        .read(true)
        .create(true)
        .write(true)
        .open(logpath)
        .unwrap();

    let cfg = ConfigBuilder::new()
        .set_time_level(LevelFilter::Off)
        .set_target_level(LevelFilter::Off)
        // .set_thread_level(LevelFilter::Info)
        .set_thread_level(LevelFilter::Off)
        .set_location_level(LevelFilter::Off)
        .build();

    WriteLogger::init(LevelFilter::Debug, cfg, logfile).unwrap();
    // WriteLogger::init(LevelFilter::Trace, cfg, logfile).unwrap();

    let mut errfile = std::fs::File::create("/home/me/code/rust/rchess/panic.log").unwrap();
    let err_redirect = Redirect::stderr(errfile).unwrap();
}

