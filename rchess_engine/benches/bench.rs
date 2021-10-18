#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]

#![feature(iter_partition_in_place)]

use rchess_engine_lib::types::*;
use rchess_engine_lib::tables::*;
use rchess_engine_lib::explore::*;

use criterion::BenchmarkId;
use criterion::{black_box, criterion_group, criterion_main, Criterion};


pub fn crit_bench_1(c: &mut Criterion) {

    // let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - "; // Position 2
    let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - "; // Position 3
    // let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"; // Position 4

    let n = 4;

    let ts    = Tables::new();
    let mut g = Game::from_fen(&ts, fen).unwrap();
    let _     = g.recalc_gameinfo_mut(&ts);

    let stop = Arc::new(AtomicBool::new(false));
    let timesettings = TimeSettings::new_f64(10., 0.1);
    let ex = Explorer::new(g.state.side_to_move, g.clone(), n, stop, timesettings);

    let mut group = c.benchmark_group("group");

    // group.sample_size(25);
    // group.measurement_time(std::time::Duration::from_secs_f64(5.));

    group.bench_function("rank moves", |b| b.iter(|| ex.explore(&ts, ex.depth)));
    // c.bench_function("rank moves", |b| b.iter(|| ex.explore(&ts, ex.depth)));

    // no collect, captures first      = 18.2 ms
    // sort by score                   = 18.8
    // order moves                     = 31.2



    // group.sample_size(100);
    // // group.measurement_time(std::time::Duration::from_secs_f64(5.));
    // // group.bench_with_input(BenchmarkId::new("table getters", c0), &c0, |b, &c| {
    // group.bench_function("table getters", |b| {
    //     b.iter(||
    //            for x in 0..8 {
    //                for y in 0..8 {
    //                    let _ = ts.get_rook(Coord(x,y));
    //                }
    //            }
    //     )
    // });

    // group.bench_function("sliding_old", |b| b.iter(
    //     || g._search_all_test(&ts, White, false)
    // ));
    // group.bench_function("sliding_test", |b| b.iter(
    //     || g._search_all_test(&ts, White, true)
    // ));

    group.finish();

}

criterion_group!(benches, crit_bench_1);
criterion_main!(benches);

