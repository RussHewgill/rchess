#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]

#![feature(iter_partition_in_place)]
#![feature(core_intrinsics)]

use rchess_engine_lib::types::*;
use rchess_engine_lib::tables::*;
use rchess_engine_lib::explore::*;
use rchess_engine_lib::util::*;
use rchess_engine_lib::search::*;

use std::time::{Duration};

use criterion::BenchmarkId;
use criterion::{black_box, criterion_group, criterion_main, Criterion};


pub fn crit_bench_1(c: &mut Criterion) {

    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - "; // Position 2
    // let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - "; // Position 3
    // let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"; // Position 4

    // let fen = "Q3b3/4bkp1/1q2np1p/NPp1p3/2P1P3/4BP1P/4B1P1/7K b - - 1 1"; // Correct = e6c7
    // let fen = "rn2kbnr/pppppppp/8/8/6b1/1QP4P/PP1PqPPN/RNB1KB1R w KQkq - 0 2"; // 1 move, then lots

    // let fen = "2rr3k/pp3pp1/1nnqbN1p/3pN3/2pP4/2P3Q1/PPB4P/R4RK1 w - - 0 1"; // WAC.001 = Qg6 = g3g6
    // let m0 = Some(Move::Quiet { from: Coord(6,2), to: Coord(6,5) });

    let n = 3;

    let ts = &_TABLES;
    // let ts    = Tables::new();
    let mut g = Game::from_fen(&ts, fen).unwrap();
    let _     = g.recalc_gameinfo_mut(&ts);

    let mut games = read_epd("/home/me/code/rust/rchess/testpositions/WAC.epd").unwrap();
    let mut games: Vec<Game> = games.into_iter().map(|(fen,_)| {
        Game::from_fen(&ts, &fen).unwrap()
    }).collect();
    // games.truncate(10);

    let stop = Arc::new(AtomicBool::new(false));
    let timesettings = TimeSettings::new_f64(0.0, 0.2);
    let ex = Explorer::new(g.state.side_to_move, g.clone(), n, stop, timesettings);

    let mut group = c.benchmark_group("group");

    group.warm_up_time(Duration::from_secs_f64(1.0));

    // // group.sample_size(50);
    // // group.measurement_time(Duration::from_secs_f64(5.));
    // group.bench_function("search_all", |b| b.iter(|| {
    //     for g in games.iter() {
    //         let mvs = g.search_all(&ts, black_box(None));
    //     }
    // }));

    // group.sample_size(10);
    // group.measurement_time(std::time::Duration::from_secs_f64(5.));

    // group.bench_function("rank moves lazy_smp", |b| b.iter(|| {
    //     let (m,stats,_) = ex.lazy_smp(&ts, false, true);
    // }));

    // group.bench_function("rank moves iter", |b| b.iter(|| {
    //     // let (m,stats) = ex.explore(&ts, None);
    //     let (m,stats) = ex.iterative_deepening(&ts, false, true);
    // }));

    // group.bench_function("search_all", |b| b.iter(|| {
    //     let mvs = g.search_all(&ts, black_box(None));
    // }));

    // let moves = g.search_all(&ts, None).get_moves_unsafe();
    // group.bench_function("move_is_legal", |b| b.iter(|| {
    //     for m in moves.iter() {
    //         let k = g.move_is_legal(&ts, *m);
    //     }
    // }));

    group.bench_function("perft", |b| b.iter(
        || g.perft(&ts, black_box(4))
    ));

    group.bench_function("perft2", |b| b.iter(
        || g.perft2(&ts, black_box(4))
    ));

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

