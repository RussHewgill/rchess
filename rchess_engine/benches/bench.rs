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

use rchess_engine_lib::brain::*;
use rchess_engine_lib::brain::matrix::*;
use rchess_engine_lib::brain::types::*;
use rchess_engine_lib::brain::types::nnue::*;

use std::collections::HashSet;
use std::time::{Duration};

use rand::{prelude::{StdRng,SliceRandom},Rng,SeedableRng};

use nalgebra as na;
use na::{DVector,DMatrix};

use ndarray as nd;

use criterion::BenchmarkId;
use criterion::{black_box, criterion_group, criterion_main, Criterion};


pub fn crit_bench_2(c: &mut Criterion) {

    let mut ins: Vec<(DVector<f32>,DVector<f32>)> = {
        let f = std::fs::read("/home/me/code/rust/rchess/temp-mnist.bin").unwrap();
        bincode::deserialize(&f).unwrap()
    };
    // let mut ins = ins.iter().map(|(a,b)| (a,b.clone())).collect::<Vec<_>>();

    let mut nn2: DNetwork<f32,784,10> = DNetwork::new_range(vec![784,16,16,10], (-1.0, 1.0));

    let mut group = c.benchmark_group("group");

    group.warm_up_time(Duration::from_secs_f64(1.0));

    // group.sample_size(20);
    group.measurement_time(Duration::from_secs_f64(4.));

    ins.truncate(200);

    let fen = "6k1/4Q3/8/8/8/5K2/8/8 w - - 6 4"; // Queen endgame, #4
    let ts = &_TABLES;
    let mut g = Game::from_fen(&ts, fen).unwrap();

    // let mut rng: StdRng = SeedableRng::seed_from_u64(1234u64);
    // let mut nn = NNUE::new(&mut rng);

    // nn.init_inputs(&g);

    // let ws0: nd::Array2<f32> = nn.weights_1.clone();
    // let xs0: nd::Array2<f32> = nn.inputs_own.clone();

    // group.bench_function("backprop 1", |b| b.iter(|| {
    //     nn2.backprop_mut_matrix(black_box(&ins), 0.1);
    // }));

    // group.bench_function("mat mul 1: ndarray f32", |b| b.iter(|| {
    //     let result = ws0.dot(&xs0);
    // }));

    // let ws1: nd::Array2<i8> = ws0.clone().map(|x| *x as i8);
    // let xs1: nd::Array2<i8> = xs0.clone().map(|x| *x as i8);

    // group.bench_function("mat mul 1: ndarray i8", |b| b.iter(|| {
    //     let result: nd::Array2<i8> = ws1.dot(&xs1);
    // }));

    // let ws2: na::DMatrix<f32> = ws0.clone().into_nalgebra(); // N,1
    // let xs2: na::DMatrix<f32> = xs0.clone().into_nalgebra(); // N,1

    // group.bench_function("mat mul 1: nalgebra f32 Dynamic", |b| b.iter(|| {
    //     let result: na::DMatrix<f32> = &ws2 * &xs2;
    // }));

    // let ws3: na::DMatrix<i8> = ws2.map(|x| x as i8); // N,1
    // let xs3: na::DMatrix<i8> = xs2.map(|x| x as i8); // N,1

    // group.bench_function("mat mul 1: nalgebra i8 Dynamic", |b| b.iter(|| {
    //     let result: na::DMatrix<i8> = &ws3 * &xs3;
    // }));

    // let ws4 = ws4.rows(0, ws4.shape().0);
    // let ws4: na::SMatrix<i8,40356,256> = ws4.fixed_slice::<40356,256>(0, 0).into();
    // let xs4 = xs4.rows(0, xs4.shape().0);
    // let xs4: na::SMatrix<i8,40356,1> = ws4.fixed_slice::<40356,1>(0, 0).into();

    // group.bench_function("mat mul 1: nalgebra i8 Static", |b| b.iter(|| {
    //     // let result: na::SMatrix<i8,> = &ws3 * &xs3;
    // }));

    // // let k = 1000;
    // const K: usize = 200;
    // let n = 1.0;
    // // let n = 1;

    // // let x = na::DMatrix::<f32>::from_element(K,K,n);
    // // let y = na::DMatrix::<f32>::from_element(K,K,n);
    // // let x = na::SMatrix::<i32,K,K>::from_element(n);
    // // let y = na::SMatrix::<i32,K,K>::from_element(n);
    // let mut result = &x * &y;

    // group.bench_function("mat mul 1", |b| b.iter(|| {
    //     result = &x * black_box(&y);
    // }));

    // let x = nd::Array2::<f32>::from_elem((K,K), n);
    // let y = nd::Array2::<f32>::from_elem((K,K), n);
    // let mut result = x.dot(&y);

    // group.bench_function("mat mul 2", |b| b.iter(|| {
    //     result = x.dot(&y);
    // }));

    group.finish();

}

pub fn crit_bench_1(c: &mut Criterion) {

    // let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - "; // Position 2
    // let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - "; // Position 3
    // let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"; // Position 4

    // let fen = "Q3b3/4bkp1/1q2np1p/NPp1p3/2P1P3/4BP1P/4B1P1/7K b - - 1 1"; // Correct = e6c7
    // let fen = "rn2kbnr/pppppppp/8/8/6b1/1QP4P/PP1PqPPN/RNB1KB1R w KQkq - 0 2"; // 1 move, then lots

    // let fen = "r4rk1/4npp1/1p1q2b1/1B2p3/1B1P2Q1/P3P3/5PP1/R3K2R b KQ - 1 1"; // Q cap d6b4

    let fen = "6k1/4Q3/8/8/8/5K2/8/8 w - - 6 4"; // Queen endgame, #4

    // let fen = "2rr3k/pp3pp1/1nnqbN1p/3pN3/2pP4/2P3Q1/PPB4P/R4RK1 w - - 0 1"; // WAC.001 = Qg6 = g3g6
    // let m0 = Some(Move::Quiet { from: Coord(6,2), to: Coord(6,5) });

    let n = 35;
    // let n = 3;

    let t = 1.0;

    let ts = &_TABLES;
    // let ts    = Tables::new();
    let mut g = Game::from_fen(&ts, fen).unwrap();

    let mut games = read_epd("/home/me/code/rust/rchess/testpositions/WAC.epd").unwrap();
    let mut games: Vec<Game> = games.into_iter().map(|(fen,_)| {
        Game::from_fen(&ts, &fen).unwrap()
    }).collect();
    // games.truncate(10);

    let stop = Arc::new(AtomicBool::new(false));
    let timesettings = TimeSettings::new_f64(0.0, t);
    // let ex = Explorer::new(g.state.side_to_move, g.clone(), n, stop, timesettings);

    let mut group = c.benchmark_group("group");

    group.warm_up_time(Duration::from_secs_f64(2.0));

    // group.sample_size(20);
    // group.measurement_time(Duration::from_secs_f64(5.));

    use evmap::{ReadHandle,ReadHandleFactory,WriteHandle};

    let (tt_r, tt_w) = evmap::Options::default()
        .with_hasher(FxBuildHasher::default())
        .construct();
    // let tt_rf = tt_w.factory();
    let tt_w: TTWrite = Arc::new(parking_lot::Mutex::new(tt_w));

    let mut rng: StdRng = SeedableRng::seed_from_u64(1234);

    fn add_zb(tt_w: TTWrite, mut zbs: &mut HashSet<Zobrist>, mut rng: &mut StdRng) {
        let zb = Zobrist(rng.gen());
        zbs.insert(zb);
        let mv = Move::Quiet { from: Coord(0,0), to: Coord(0,1), pc: Queen };
        let si = SearchInfo::new(mv, vec![], 1, Node::PV, 0);
        let mut w = tt_w.lock();
        w.update(zb, si);
        w.refresh();
    }

    let mut zbs = HashSet::<Zobrist>::default();

    for _ in 0..1_000_000 {
        add_zb(tt_w.clone(), &mut zbs, &mut rng);
    }

    let zbs2: Vec<Zobrist> = zbs.iter().copied().collect();
    group.bench_function("test evmap 100_000", |b| b.iter(|| {
        let zb = zbs2.choose(&mut rng).unwrap();
        let si = tt_r.get(&zb).unwrap();
    }));

    // group.bench_function("explore endgame", |b| b.iter(|| {
    //     let ex = Explorer::new(g.state.side_to_move, g.clone(), n, stop.clone(), timesettings);
    //     let (m,stats) = ex.explore(&ts, None);
    // }));

    // group.bench_function("explore", |b| b.iter(|| {
    //     let ex = Explorer::new(g.state.side_to_move, g.clone(), n, stop.clone(), timesettings);
    //     let (m,stats) = ex.explore(&ts, None);
    // }));

    // group.bench_function("game_phase", |b| b.iter(|| {
    //     let mut k = 0;
    //     for g in games.iter() {
    //         let ph = g.game_phase();
    //         k += ph;
    //     }
    // }));

    // group.bench_function("search_all", |b| b.iter(|| {
    //     for g in games.iter() {
    //         let mvs = g.search_all(&ts);
    //     }
    // }));

    // group.bench_function("_search_pawns", |b| b.iter(|| {
    //     for g in games.iter() {
    //         let col = White;
    //         let pawns = g.search_pawns(&ts, black_box(col));
    //         let n = pawns.len();
    //     }
    // }));
    // group.bench_function("_search_pawns_iter", |b| b.iter(|| {
    //     for g in games.iter() {
    //         let col = White;
    //         let pawns = g.search_pawns_iter(&ts, black_box(col));
    //         let n = pawns.collect::<Vec<_>>().len();
    //     }
    // }));

    // group.bench_function("search_sliding 1", |b| b.iter(|| {
    //     for g in games.iter() {
    //         let col = White;
    //         let b = g.search_sliding(&ts, black_box(Bishop), black_box(col));
    //         let r = g.search_sliding(&ts, black_box(Rook), black_box(col));
    //         let q = g.search_sliding(&ts, black_box(Queen), black_box(col));
    //         let n = b.len() + r.len() + q.len();
    //     }
    // }));
    // group.bench_function("search_sliding 2", |b| b.iter(|| {
    //     for g in games.iter() {
    //         let col = White;
    //         let b = g.search_sliding_iter(&ts, black_box(Bishop), black_box(col));
    //         let r = g.search_sliding_iter(&ts, black_box(Rook), black_box(col));
    //         let q = g.search_sliding_iter(&ts, black_box(Queen), black_box(col));
    //         // let n = b.len() + r.len() + q.len();
    //         let b = b.collect::<Vec<_>>();
    //         let r = r.collect::<Vec<_>>();
    //         let q = q.collect::<Vec<_>>();
    //     }
    // }));

    // group.sample_size(10);
    // group.measurement_time(std::time::Duration::from_secs_f64(5.));

    // group.bench_function("rank moves lazy_smp", |b| b.iter(|| {
    //     let (m,stats,_) = ex.lazy_smp(&ts, false, true);
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

    // group.bench_function("perft", |b| b.iter(
    //     || g.perft(&ts, black_box(4))
    // ));
    // group.bench_function("perft2", |b| b.iter(
    //     || g.perft2(&ts, black_box(4))
    // ));

    // group.bench_function("sliding_old", |b| b.iter(
    //     || g._search_all_test(&ts, White, false)
    // ));
    // group.bench_function("sliding_test", |b| b.iter(
    //     || g._search_all_test(&ts, White, true)
    // ));

    group.finish();

}

criterion_group!(benches, crit_bench_1);
// criterion_group!(benches, crit_bench_2);
criterion_main!(benches);

