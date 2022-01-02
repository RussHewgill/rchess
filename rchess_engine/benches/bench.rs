#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]

#![feature(iter_partition_in_place)]
#![feature(core_intrinsics)]

use aligned::{Aligned,A2,A64};
use rchess_engine_lib::sf_compat::accumulator::NNDelta;
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

pub fn crit_bench_simd(c: &mut Criterion) {

    let mut group = c.benchmark_group("group");

    group.warm_up_time(Duration::from_secs_f64(1.0));

    // group.sample_size(20);
    // group.measurement_time(Duration::from_secs_f64(3.));

    let mut rng: StdRng = SeedableRng::seed_from_u64(1234u64);

    use rchess_engine_lib::simd_test::*;

    // const IS: usize = 2 * 1024;
    // const OS: usize = 8;
    const IS: usize = 8;
    const OS: usize = 32;

    // let input: [u8; IS]   = array_init::array_init(|_| rng.gen_range(0..2));
    // let weights: Vec<i8>  = (0..OS * IS).map(|_| rng.gen_range(-100..100)).collect();
    // let biases: [i32; OS] = array_init::array_init(|_| rng.gen_range(-100..100));
    // let mut output = [0i32; OS];

    let biases: [i32; OS] = array_init::array_init(|x| (x as i32 % 20) - 10);
    let weights: [i8; IS * OS] = array_init::array_init(|x| (x as i8 % 20) - 10);
    // let weights: [i8; IS * OS] = [1; IS * OS];
    let biases: Aligned<A64,_> = Aligned(biases);
    let weights: Aligned<A64,_> = Aligned(weights);
    // let biases: Aligned<A2,_> = Aligned(bs);
    // let weights: Aligned<A2,_> = Aligned(ws);

    let input: [u8; IS] = array_init::array_init(|x| x as u8 % 2);
    let input: Aligned<A64,_> = Aligned(input);
    // let input: Aligned<A2,_> = Aligned(input);
    let mut output = [0i32; OS];

    use rchess_engine_lib::sf_compat::{Layer0,NNAffine,NNLayer};

    let mut layer0 = Layer0::new();
    // let mut layer1 = NNAffine::<Layer0, OS, IS>::new(layer0);
    let mut layer1 = NNAffine::<Layer0, OS, IS>::new(layer0);
    layer1.weights = Aligned(weights.to_vec());
    layer1.biases = biases.clone();

    // let xs = layer1.propagate(input.as_ref());
    // // let sum: i32 = xs.iter().sum();
    // // eprintln!("sum = {:?}", sum);
    // eprintln!("xs[0..4] = {:?}", &xs[0..4]);

    // simd_mm_1::<IS,OS>(
    //     black_box(input.as_ref()),
    //     black_box(weights.as_ref()),
    //     black_box(biases.as_ref()),
    //     black_box(output.as_mut()));
    // let sum: i32 = output.iter().sum();
    // eprintln!("sum = {:?}", sum);

    // group.bench_function("SIMD mm 0", |b| b.iter(|| {
    //     simd_mm_0::<IS,OS>(
    //         black_box(input.as_ref()),
    //         black_box(weights.as_ref()),
    //         black_box(biases.as_ref()),
    //         black_box(output.as_mut()));
    // }));

    // group.bench_function("SIMD mm 1", |b| b.iter(|| {
    //     simd_mm_1::<IS,OS>(
    //         black_box(input.as_ref()),
    //         black_box(weights.as_ref()),
    //         black_box(biases.as_ref()),
    //         black_box(output.as_mut()));
    // }));

    // group.bench_function("NNAffine mm basic", |b| b.iter(|| {
    //     layer1.propagate(input.as_ref());
    // }));

    // group.bench_function("NNAffine mm simd", |b| b.iter(|| {
    //     layer1.propagate(input.as_ref());
    // }));

    // const N: usize = 2048;
    // let xs1: [u8; N] = array_init::array_init(|x| rng.gen_range(0..2));
    // let xs2: [i8; N] = array_init::array_init(|x| rng.gen_range(-10..10));

    // let ys1: [i32; N] = array_init::array_init(|x| xs1[x] as i32);
    // let ys2: [i32; N] = array_init::array_init(|x| xs2[x] as i32);

    // group.bench_function("dot prod basic", |b| b.iter(|| {
    //     let x = dot_product_basic(black_box(&xs1), &xs2);
    // }));

    // group.bench_function("dot prod 0", |b| b.iter(|| {
    //     dot_product0(black_box(&xs1), &xs2)
    // }));

    // group.bench_function("dot prod 1", |b| b.iter(|| {
    //     dot_product1(black_box(&ys1), &ys2)
    // }));

    // group.bench_function("dot prod 2", |b| b.iter(|| {
    //     dot_product2(black_box(&ys1), &ys2)
    // }));

    // group.bench_function("SIMD mm 1", |b| b.iter(|| {
    //     simd_mm_1::<IS,OS>(black_box(&input), &weights, &biases, &mut output);
    //     // SIMD_01::<IS,OS>::simd_mm(black_box(&input), &weights, &biases, &mut output);
    // }));

    // group.bench_function("SIMD mm 2", |b| b.iter(|| {
    //     simd_mm_2::<IS,OS>(black_box(&input), &weights, &biases, &mut output);
    //     // SIMD_01::<IS,OS>::simd_mm(black_box(&input), &weights, &biases, &mut output);
    // }));

    // group.bench_function("SIMD ndarray mm 0", |b| b.iter(|| {
    //     simd_nd_mm_0::<IS,OS>(
    //         black_box(&input),
    //         &weights,
    //         &biases,
    //         &mut output,
    //     );
    // }));

    // use ndarray as nd;
    // use nd::{Array2,ArrayView2,ArrayViewMut2,ShapeBuilder};

    // let input: nd::Array2<i8> = nd::Array2::from_shape_vec((IS, 1), input.to_vec()).unwrap();
    // let weights: nd::Array2<i8> = nd::Array2::from_shape_vec((IS,OS).f(), weights).unwrap();
    // let weights = weights.reversed_axes();
    // let biases: nd::Array2<i32> = nd::Array2::from_shape_vec((OS, 1), biases.to_vec()).unwrap();

    // let mut result = nd::Array2::<i32>::zeros((OS,1));

    // group.bench_function("SIMD ndarray mm 1", |b| b.iter(|| {
    //     simd_nd_mm_1::<IS,OS>(
    //         black_box(input.view()),
    //         weights.view(),
    //         biases.view(),
    //         &mut result,
    //     );
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

    // let fen = "6k1/4Q3/8/8/8/5K2/8/8 w - - 6 4"; // Queen endgame, #4

    // let fen = "2rr3k/pp3pp1/1nnqbN1p/3pN3/2pP4/2P3Q1/PPB4P/R4RK1 w - - 0 1"; // WAC.001 = Qg6 = g3g6
    // let m0 = Some(Move::Quiet { from: Coord(6,2), to: Coord(6,5) });

    // let n = 35;
    // // let n = 3;
    // let t = 1.0;

    let ts = &_TABLES;
    // let ts    = Tables::new();

    let mut rng: StdRng = SeedableRng::seed_from_u64(1234u64);

    let mut wacs = read_epd("/home/me/code/rust/rchess/testpositions/WAC.epd").unwrap();
    let mut wacs: Vec<Game> = wacs.into_iter().map(|(fen,_)| {
        Game::from_fen(&ts, &fen).unwrap()
    }).collect();
    // games.truncate(10);

    let mut g0 = Game::from_fen(&ts, STARTPOS).unwrap();

    let stop = Arc::new(AtomicBool::new(false));
    // let ex = Explorer::new(g.state.side_to_move, g.clone(), n, stop, timesettings);

    let mut group = c.benchmark_group("group");

    group.warm_up_time(Duration::from_secs_f64(2.0));

    // group.sample_size(20);
    // group.measurement_time(Duration::from_secs_f64(5.));

    // let fen = "1n4k1/2p2rpp/1n6/1q6/8/4QP2/1P3P1P/1N1R2K1 w - - 0 1"; // #3, Qt R d1d8
    let fen = "8/k7/3p4/p2P1p2/P2P1P2/8/8/K7 w - - "; // Lasker-Reichhelm Position, Qt K a1b1
    // let (n,t) = (35,0.5);
    let (n,t) = (12,0.2);
    let timesettings = TimeSettings::new_f64(0.0, t);
    let mut g = Game::from_fen(&ts, fen).unwrap();
    group.bench_function("explore endgame", |b| b.iter(|| {
        let mut ex = Explorer::new(g.state.side_to_move, g.clone(), n, timesettings);
        ex.cfg.clear_table = true;
        let (m,stats) = ex.explore(&ts);
    }));

    // let fen = "r4rk1/4npp1/1p1q2b1/1B2p3/1B1P2Q1/P3P3/5PP1/R3K2R b KQ - 1 1"; // Q cap d6b4
    // let (n,t) = (35,1.0);
    // let timesettings = TimeSettings::new_f64(0.0,t);
    // let mut g = Game::from_fen(&ts, fen).unwrap();
    // group.bench_function("explore", |b| b.iter(|| {
    //     let ex = Explorer::new(g.state.side_to_move, g.clone(), n, timesettings);
    //     let (m,stats) = ex.explore(&ts, None);
    // }));

    use rchess_engine_lib::movegen::*;

    let st = ABStack::new();

    // group.bench_function("movegen all", |b| b.iter(|| {
    //     for g in wacs.iter() {
    //         let mut movegen = MoveGen::new(&ts, &g, None, &st, 0, 0);
    //         let mut x = 0;
    //         while let Some(mv) = movegen.next(&st) {
    //             x += 1;
    //         }
    //     }
    // }));

    // let fen = "1k6/2n5/2p5/3n4/4P3/2N1N3/8/K7 w - - 0 1";
    // let mut g = Game::from_fen(&ts, fen).unwrap();
    // let mv0 = Move::new_capture("e4", "d5", Pawn, Knight);
    // group.bench_function("see", |b| b.iter(|| {
    //     let k0 = g.static_exchange(&ts, mv0);
    // }));
    // group.bench_function("see ge", |b| b.iter(|| {
    //     let val = 0;
    //     let k1 = g.static_exchange_ge(ts, mv0, val);
    // }));

    // let n = 3;
    // let t = 0.1;
    // let timesettings = TimeSettings::new_f64(0.0,t);
    // let mut ex = Explorer::new(g0.state.side_to_move, g0.clone(), n, timesettings);
    // ex.cfg.num_threads = Some(1);
    // // ex.load_nnue("/home/me/code/rust/rchess/nn-63376713ba63.nnue").unwrap();

    // println!("starting");
    // let t0 = std::time::Instant::now();
    // for (n,g) in wacs.iter().enumerate() {
    //     eprintln!("n = {:?}", n);
    //     ex.clear_tt();
    //     ex.update_game(*g);
    //     let (m,stats) = ex.explore(&ts);
    // }
    // let t1 = t0.elapsed().as_secs_f64();
    // println!("finished in {:.3} seconds", t1);

    // group.bench_function("explore wacs", |b| b.iter(|| {
    //     for g in wacs.iter() {
    //         ex.clear_tt();
    //         ex.update_game(g.clone());
    //         let (m,stats) = ex.explore(&ts);
    //     }
    // }));

    // let ev_mid = EvalParams::default();
    // let ev_end = EvalParams::default();

    // group.bench_function("eval arr", |b| b.iter(|| {
    //     let arr = ev_mid.to_arr();
    //     let ev2 = EvalParams::from_arr(&arr);
    // }));

    // let ph_rw = rchess_engine_lib::pawn_hash_table::PHTableFactory::new();
    // let ph_rw = ph_rw.handle();

    // Baseline = 118 us
    // material = 1.78 us
    // psqt     = 9.03 us
    // mobility = 38.61 us
    // pieces   = 28.0 us
    // pawns
    // nohash   = 120 us
    // hash     = 24.3 us
    // purged   = 230 us

    // group.bench_function("eval wacs all", |b| b.iter(|| {
    //     let mut k = 0;
    //     for g in wacs.iter() {
    //         k += g.sum_evaluate_mg(&ts, &ev_mid, &ev_end, Some(&ph_rw));
    //     }
    // }));

    // group.bench_function("eval material2", |b| b.iter(|| {
    //     let mut k = 0;
    //     for g in wacs.iter() {
    //         k += g.score_material2(White) - g.score_material2(Black);
    //     }
    // }));
    // group.bench_function("eval psqt", |b| b.iter(|| {
    //     let mut k = 0;
    //     for g in wacs.iter() {
    //         k += g.score_psqt(&ts, &ev_mid, White) - g.score_psqt(&ts, &ev_mid, Black);
    //     }
    // }));
    // group.bench_function("eval mobility", |b| b.iter(|| {
    //     let mut k = 0;
    //     for g in wacs.iter() {
    //         k += g.score_mobility(&ts, White) - g.score_mobility(&ts, Black);
    //     }
    // }));
    // group.bench_function("eval pieces", |b| b.iter(|| {
    //     let mut k = 0;
    //     for g in wacs.iter() {
    //         k += g.score_pieces_mg(&ts, &ev_mid, White) - g.score_pieces_mg(&ts, &ev_mid, Black);
    //     }
    // }));
    // group.bench_function("eval pawns no hash", |b| b.iter(|| {
    //     let mut k = 0;
    //     for g in wacs.iter() {
    //         let pawns = g.score_pawns(ts, &ev_mid, &ev_end, None, true);
    //         k += pawns[0] - pawns[1];
    //     }
    // }));
    // group.bench_function("eval pawns hashed", |b| b.iter(|| {
    //     ph_rw.purge();
    //     let mut k = 0;
    //     for g in wacs.iter() {
    //         let pawns = g.score_pawns(ts, &ev_mid, &ev_end, Some(&ph_rw), true);
    //         k += pawns[0] - pawns[1];
    //     }
    // }));

    // group.bench_function("eval wacs old", |b| b.iter(|| {
    //     let mut k = 0;
    //     for g in wacs.iter() {
    //         k += g.sum_evaluate2(&ts);
    //     }
    // }));

    // group.bench_function("game_phase", |b| b.iter(|| {
    //     let mut k = 0;
    //     for g in games.iter() {
    //         let ph = g.game_phase();
    //         k += ph;
    //     }
    // }));

    // group.bench_function("search_all", |b| b.iter(|| {
    //     for g in wacs.iter() {
    //         let mvs = g.search_all(&ts);
    //     }
    // }));

    group.finish();

}

pub fn crit_bench_nnue(c: &mut Criterion) {
    let mut rng: StdRng = SeedableRng::seed_from_u64(1234u64);

    let mut group = c.benchmark_group("group");

    group.warm_up_time(Duration::from_secs_f64(2.0));

    group.sample_size(100);
    group.measurement_time(Duration::from_secs_f64(5.));

    let ts = Tables::read_from_file_def().unwrap();

    use rchess_engine_lib::sf_compat::*;

    let path = "../nn-63376713ba63.nnue";
    let mut nn = NNUE4::read_nnue(path).unwrap();

    let mut wacs = read_epd("/home/me/code/rust/rchess/testpositions/WAC.epd").unwrap();
    let mut wacs: Vec<Game> = wacs.into_iter().map(|(fen,_)| {
        Game::from_fen(&ts, &fen).unwrap()
    }).collect();

    // let wacs2: Vec<(Game,NNIndex)> = wacs.into_iter().flat_map(|g| {
    //     let moves = g.search_all(&ts).get_moves_unsafe();
    //     let mv = moves.choose(&mut rng).unwrap();
    //     if mv.piece() == Some(King) { None } else {
    //         let d = nn.ft._make_move(&g, *mv);
    //         Some((g,d[0].get().0))
    //     }
    // }).collect();

    let mut ft = nn.ft.clone();

    // group.bench_function("nnue _accum_rem", |b| b.iter(|| {
    //     for (g,idx) in wacs2.iter() {
    //         // ft.reset_accum(black_box(&g));
    //         ft._accum_rem(g.state.side_to_move, *idx);
    //     }
    // }));

    // group.bench_function("nnue eval", |b| b.iter(|| {
    //     for g in wacs.iter() {
    //         // nn.ft.accum.needs_refresh = [true; 2];
    //         nn.ft.reset_accum(&g);
    //         let v = nn.evaluate(black_box(&g), false);
    //     }
    // }));

    // group.bench_function("nnue reset accum", |b| b.iter(|| {
    //     for g in wacs.iter() {
    //         nn.ft.reset_accum(&g);
    //     }
    // }));

    // group.bench_function("nnue reset accum simd", |b| b.iter(|| {
    //     for g in wacs.iter() {
    //         nn.ft._update_accum_simd(&g, White);
    //         nn.ft._update_accum_simd(&g, Black);
    //     }
    // }));

    // // let mut wacs2: Vec<(Game,Move,NNFeatureTrans)> = wacs.into_iter().map(|g| {
    // let mut wacs2: Vec<(Game,Move)> = wacs.into_iter().map(|g| {
    //     let moves = g.search_all(&ts).get_moves_unsafe();
    //     let mv = moves.choose(&mut rng).unwrap();
    //     // let mut ft = nn.ft.clone();
    //     // ft.reset_accum(&g);
    //     (g,*mv)
    // }).collect();
    // // let wacs3 = wacs2.clone().into_iter().map(|(g,mv,ft)| {
    // //     (g,*mv,ft)
    // // }).collect::<Vec<_>>();
    // let wacs3: Vec<(Game,Move)> =
    //     wacs2.clone().into_iter().filter(|(_,mv)| mv.piece() != Some(King)).collect();
    // let mut ft = nn.ft.clone();

    // group.bench_function("nnue _update_accum", |b| b.iter(|| {
    //     let mut wacs22 = wacs2.clone();
    //     for (g,mv) in wacs22.iter_mut() {
    //         ft.reset_accum(black_box(&g));
    //     }
    // }));

    // group.bench_function("nnue make_move", |b| {
    //     b.iter(|| {
    //         ft.accum.stack_copies.clear();
    //         ft.accum.stack_delta.clear();
    //         for (g,mv) in wacs3.iter() {
    //             ft.make_move(g, *mv);
    //         }
    //     })});

    group.finish();
}

// criterion_group!(benches, crit_bench_nnue);
// criterion_group!(benches, crit_bench_simd);
criterion_group!(benches, crit_bench_1);
// criterion_group!(benches, crit_bench_2);
criterion_main!(benches);

