#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]

use rchess_engine_lib::types::*;
use rchess_engine_lib::tables::*;
use rchess_engine_lib::explore::*;

use criterion::{black_box, criterion_group, criterion_main, Criterion};


pub fn crit_bench_1(c: &mut Criterion) {
    let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";

    let n = 3;

    let ts    = Tables::new();
    let mut g = Game::from_fen(fen).unwrap();
    let _     = g.recalc_gameinfo_mut(&ts);

    let stop = Arc::new(AtomicBool::new(false));
    let timesettings = TimeSettings::new_f64(10., 0.1);
    let ex = Explorer::new(g.state.side_to_move, g.clone(), n, stop, timesettings);

    let mut group = c.benchmark_group("group");
    group.sample_size(25);

    group.bench_function("rank moves perft_pos_4", |b| b.iter(|| ex.explore(&ts, ex.depth)));
    group.finish();

}

criterion_group!(benches, crit_bench_1);
criterion_main!(benches);

