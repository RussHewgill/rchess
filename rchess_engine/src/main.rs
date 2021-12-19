#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_doc_comments)]

// #![feature(core_intrinsics)]
#![feature(backtrace,backtrace_frames)]
// #![feature(generic_const_exprs)]
#![feature(portable_simd)]
#![feature(array_chunks)]
#![feature(asm)]

#![allow(clippy::all)]

// #![allow(
//     clippy::all,
//     clippy::restriction,
//     clippy::pedantic,
//     clippy::nursery,
//     clippy::cargo,
// )]

// extern crate blas_src;
// extern crate openblas_src;

use std::collections::{HashMap,HashSet,VecDeque};
use std::slice::SliceIndex;
use std::str::FromStr;

use itertools::Itertools;

use rchess_engine_lib::{timer,timer_loop,eprint_self};
use rchess_engine_lib::explore::*;
use rchess_engine_lib::opening_book::*;
use rchess_engine_lib::qsearch::*;
use rchess_engine_lib::types::*;
use rchess_engine_lib::search::*;
use rchess_engine_lib::tables::*;
use rchess_engine_lib::parsing::*;
use rchess_engine_lib::util::*;
use rchess_engine_lib::evaluate::*;
use rchess_engine_lib::explore::*;
use rchess_engine_lib::tuning::*;
use rchess_engine_lib::alphabeta::*;
use rchess_engine_lib::{stats,not_stats,stats_or};
#[cfg(feature = "syzygy")]
use rchess_engine_lib::syzygy::SyzygyTB;
use rchess_engine_lib::brain::trainer::*;

use log::{debug, error, log_enabled, info, Level};
use gag::Redirect;
use simplelog::{
    CombinedLogger,TermLogger,WriteLogger,ConfigBuilder,ColorChoice,TerminalMode,LevelFilter,
};
use chrono::Timelike;
use rand::{prelude::{StdRng,SliceRandom},Rng,SeedableRng};
use rand::distributions::{Uniform,uniform::SampleUniform};

use std::time::{Instant,Duration};

// #[cfg(feature = "nope")]
fn main() {

    let args: Vec<String> = std::env::args().collect();
    if args.len() <= 1 { main9(); return; }
    let arg1: &str = &args[1];
    match &arg1[..] {
        "tt"        => main_tt(),
        "nn"        => main_nn(),
        "nnue"      => main_nnue(),
        "train"     => main_nnue_train(),
        "movegen"   => main_movegen(),
        "simd"      => main_simd(),
        "eval"      => main_eval(),
        "gensfen"   => {
            let count = u64::from_str(&args[2]).unwrap();
            let time = f64::from_str(&args[3]).unwrap();
            let path = &args[4];

            main_gensfen(count, path);
        },
        "tuning"    => main_tuning(),
        "wac"       => match args.get(2).map(|x| u64::from_str(x).ok()) {
            Some(n) => main_wac(n, false),
            _       => main_wac(None, false),
        },
        "perft"     => match args.get(2).map(|x| u64::from_str(x).ok()) {
            Some(n) => main_perft(n),
            _       => main_perft(None),
        },
        _           => main9(),
    }

}

#[allow(unreachable_code)]
fn _main() {

    // let logpath = "./log.log";
    // use std::fs::OpenOptions;
    // let logfile = OpenOptions::new()
    //     .truncate(true)
    //     .read(true)
    //     .create(true)
    //     .write(true)
    //     .open(logpath)
    //     .unwrap();

    // let err_redirect = Redirect::stderr(logfile).unwrap();

    // env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
    // // env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
    //     .format_timestamp(None)
    //     .format_module_path(false)
    //     .format_target(false)
    //     // .format_level(false)
    //     .init();

    // let now = chrono::Local::now();
    // let logpath = format!(
    //     "/home/me/code/rust/rchess/logs/log-{:0>2}:{:0>2}:{:0>2}.log",
    //     now.hour(), now.minute(), now.second());
    // let mut logfile = std::fs::OpenOptions::new()
    //     .truncate(true)
    //     .read(true)
    //     .create(true)
    //     .write(true)
    //     .open(logpath)
    //     .unwrap();
    // WriteLogger::init(LevelFilter::Debug, Config::default(), logfile).unwrap();
    // // WriteLogger::init(LevelFilter::Trace, Config::default(), logfile).unwrap();

    // #[cfg(not(feature = "par"))]
    // rayon::ThreadPoolBuilder::new()
    //     .num_threads(1)
    //     // .stack_size(16 * 1024 * 1024)
    //     .build_global()
    //     .unwrap();

    // #[cfg(feature = "par")]
    // rayon::ThreadPoolBuilder::new()
    //     .num_threads(1)
    //     // .stack_size(16 * 1024 * 1024)
    //     .build_global()
    //     .unwrap();

    // let ts = Tables::new();
    // ts.write_to_file_def().unwrap();
    // // let ts = Tables::read_from_file("tables.bin").unwrap();

    // let from: Coord = "A1".into();
    // let to: Coord = "B2".into();
    // let mut mvs = vec![
    //     Move::Quiet { from, to, pc: Pawn },
    //     Move::Quiet { from, to, pc: Queen },
    //     Move::PawnDouble { from, to },
    //     Move::Capture { from, to, pc: Pawn, victim: Pawn },
    //     Move::EnPassant { from, to, capture: from }
    // ];
    // mvs.sort();
    // for m in mvs.iter() {
    //     eprintln!("m = {:?}", m);
    // }

    // let ts = &_TABLES;
    // let mut games = read_epd("/home/me/code/rust/rchess/testpositions/WAC.epd").unwrap();
    // let mut games: Vec<(Game,String)> = games.into_iter().map(|(fen,_)| {
    //     (Game::from_fen(&ts, &fen).unwrap(), fen)
    // }).collect();
    // let col = White;


    // #[derive(Debug,PartialEq,PartialOrd,Clone)]
    // pub enum Wat {
    //     Wat1(ABResult),
    //     // Wat2(Vec<ABResult>),
    //     Wat2(ABResult, Vec<ABResult>),
    //     Wat3
    // }

    // // let s = std::mem::size_of::<Eval>();
    // // let s = std::mem::size_of::<rchess_engine_lib::types::Color>();
    // // let s = std::mem::size_of::<Game>();
    // let s0 = std::mem::size_of::<[i16; 256 * 40320]>();
    // let s1 = std::mem::size_of::<[i16; 256]>();
    // // let s = s0 * 2 + s0 * 2 + s1 * 2 + s1;
    // let s = s0 * 1 + s0 * 2 + s1 * 2 + s1;
    // // let s = std::mem::size_of::<GameState>();
    // eprintln!("size  = {:?}", s / 1024 / 1024);
    // // let a = std::mem::align_of::<GameState>();
    // // eprintln!("align = {:?}", a);
    // // let s = u16::MAX;
    // // eprintln!("s = {:#8x}", s);
    // return;

    // // main_nnue();
    // // main_nn();
    // main_mnist();
    // return;

    // let mut args: Vec<String> = std::env::args().collect();
    // match args.len() {
    //     1 => main9(),
    //     2 => match args[2].as_str() {
    //         "nn" => main_nn(),
    //         _    => unimplemented!(),
    //     }
    // }

    // let mut args: Vec<String> = std::env::args().collect();
    // match args.get(1) {
    //     Some(s) => match s.as_str() {
    //         // "wac"   => main3(false), // read from file and test
    //         "wac"   => match args.get(2).map(|x| u64::from_str(x).ok()) {
    //             Some(Some(n)) => main3(Some(n),false),
    //             _             => main3(None, false),
    //         }
    //         "wac2"  => main3(None, true), // read from file and test, send URL to firefox
    //         "perft" => match args.get(2).map(|x| u64::from_str(x).ok()) {
    //             Some(n) => main_perft(n),
    //             _       => main_perft(None),
    //         }
    //         "main7" => main7(),
    //         "sts"   => match args.get(2).map(|x| u64::from_str(x).ok()) {
    //             Some(n) => main_sts(n),
    //             _       => main_sts(None),
    //         }
    //         // "nn"    => main_nn(),
    //         "nnue"  => main_nnue(),
    //         "nn"    => main_nn(),
    //         "simd"  => main_simd(),
    //         _       => {},
    //     },
    //     // None    => main7(),
    //     None    => main9(),
    // }

    // main6();
    // main5(); // search + eval position
    // main2();
    // main4(); // perft

    // // main8(); // eval testing
    // main7();
    // // main3(); // read from file and test

}

#[allow(unreachable_code)]
fn main_movegen() {
}

#[allow(unreachable_code)]
fn main_tt() {
    use rchess_engine_lib::lockless_map::*;

    use jemalloc_ctl::{stats, epoch};
    // epoch::advance().unwrap();
    // let allocated = stats::allocated::read().unwrap();
    // let resident = stats::resident::read().unwrap();
    // println!("{} bytes allocated/{} bytes resident", allocated, resident);

    let mut rng: StdRng = SeedableRng::seed_from_u64(1234);
    // let zb = Zobrist(rng.gen());

    // // let s0 = std::mem::size_of::<TTEntry>();
    // let s0 = std::mem::size_of::<Score>();
    // eprintln!("s0 = {:?}", s0);

    let tt2 = TransTable::new_mb(32);

    // let mut b0 = Bucket::new();

    // let idx = tt.calc_index(zb);
    // let ver = tt.calc_verification(zb);

    // let mut x = 0;

    use std::cell::UnsafeCell;
    use std::ptr::NonNull;
    use std::mem;
    use std::alloc::{Layout, handle_alloc_error, self};

    // Bucket     = 36
    // SearchInfo = 8
    // TTEntry    = 12

    // let k0: u128 = 0;

    // let k0 = std::mem::size_of::<rchess_engine_lib::lockless_map::Bucket>();
    // eprintln!("k0 = {:?}", k0);

    // let k1 = std::mem::size_of::<TTEntry>();
    // eprintln!("k1 = {:?}", k1);

    return;

    let zb1 = Zobrist(0xcdab13aceaa91520);
    let zb2 = Zobrist(0x6b2bc7c01dffde39);

    // unsafe {
    //     let idx0 = tt2.calc_index(zb1);
    //     eprintln!("idx0 = {:?}", idx0);
    //     let b0 = tt2.bucket(idx0);
    //     eprintln!("b0 = {:?}", (*b0).x);
    //     let idx1 = tt2.calc_index(zb2);
    //     eprintln!("idx1 = {:?}", idx1);
    //     let b1 = tt2.bucket(idx1);
    //     eprintln!("b1 = {:?}", (*b1).x);
    // }

    // return;

    let mut si0 = SearchInfo::empty();
    let mut si1 = SearchInfo::empty();

    si0.depth_searched = 22;
    si1.depth_searched = 33;

    let si2 = tt2.probe(zb1);
    eprintln!("si2 = {:?}", si2);
    let si2 = tt2.probe(zb2);
    eprintln!("si2 = {:?}", si2);

    tt2.insert(zb1, si0);
    tt2.insert(zb2, si1);

    // unsafe {
    //     let b0 = *tt2.bucket(201644);
    //     eprintln!("b0 = {:?}", b0);
    //     let ver = 3936949536;
    //     let si2 = b0.find(ver);
    //     eprintln!("si2 = {:?}", si2);
    // }

    let si2 = tt2.probe(zb1);
    eprintln!("si2 = {:?}", si2);
    let si2 = tt2.probe(zb2);
    eprintln!("si2 = {:?}", si2);

    return;

    // old
    if !true {

    if !true {
        use std::alloc::{Layout, handle_alloc_error, self};
        use std::ptr::NonNull;
        use std::cell::UnsafeCell;

        // let size = 6;
        // let size = size * std::mem::size_of::<[u8; 2]>();

        // let layout = Layout::from_size_align(size, 2).unwrap();

        // let xs = UnsafeCell::new(unsafe {
        //     let ptr: *mut u8 = alloc::alloc_zeroed(layout);
        //     let ptr2: NonNull<[u8; 2]> = match NonNull::new(ptr) {
        //         Some(p) => p.cast(),
        //         _       => handle_alloc_error(layout),
        //     };
        //     ptr2
        // });

        // unsafe {
        //     let arr: *mut [u8; 2] = 
        //     // let init_entry: *
        // }

        let mut xs: [u8; 6] = [0, 1, 2, 3, 4, 5];
        eprintln!("xs = {:?}", xs);

        unsafe {
            let ptr: *mut u8 = xs.as_mut_ptr();
            *ptr += 1;
        }
        eprintln!("xs = {:?}", xs);

        return;
    }

    // let tt = TransTable::new(10);
    // let tt = TransTable::new_num_clusters(1);
    // let k0 = tt.num_entries();
    // let k1 = tt.num_clusters();
    // eprintln!("k0 = {:?}", k0);
    // eprintln!("k1 = {:?}", k1);

    // let mv0         = Move::new_quiet("E2", "E4", Pawn);
    // let p0          = PackedMove::convert(mv0);
    // let p1: [u8; 2] = p0.pack().unwrap();

    // let zb0 = Zobrist(rng.gen());
    // let zb1 = Zobrist(rng.gen());
    // let zb2 = Zobrist(rng.gen());
    // let zb3 = Zobrist(rng.gen());

    // let (found, entry) = tt.probe(&zb0);
    // eprintln!("found = {:?}", found);
    // entry.place(zb, p1, 1, Node::PV, 2);
    // let (found, entry) = tt.probe(&zb1);
    // eprintln!("found = {:?}", found);
    // entry.place(zb, p1, 1, Node::PV, 2);
    // let (found, entry) = tt.probe(&zb2);
    // eprintln!("found = {:?}", found);
    // entry.place(zb, p1, 1, Node::PV, 2);
    // let (found, entry) = tt.probe(&zb3);
    // eprintln!("found = {:?}", found);
    // entry.place(zb, p1, 1, Node::PV, 2);

    }

    init_logger();
    let ts = Tables::read_from_file_def().unwrap();

    let fen = STARTPOS;
    // let fen = "8/k7/3p4/p2P1p2/P2P1P2/8/8/K7 w - - "; // Lasker-Reichhelm Position, Qt K a1b1

    let mut g = Game::from_fen(&ts, fen).unwrap();

    let n = 40;

    let t   = 10.0;
    let inc = 0.0;

    let t0 = std::time::Instant::now();
    let timesettings = TimeSettings::new_f64(t,inc);
    let mut ex = Explorer::new(g.state.side_to_move, g.clone(), n, timesettings);
    ex.load_syzygy("/home/me/code/rust/rchess/tables/syzygy/").unwrap();
    ex.cfg.return_moves = true;
    ex.cfg.clear_table = false;
    // ex.cfg.num_threads = Some(6);
    ex.cfg.num_threads = Some(1);
    // ex.cfg.num_threads = None;

    let (t_opt,t_max) = ex.timer.allocate_time(g.state.side_to_move, 1);
    eprintln!("t_opt = {:?}", t_opt);
    eprintln!("t_max = {:?}", t_max);

    return;

    debug!("ex.cfg.num_threads = {:?}", ex.cfg.num_threads);

    let (res,moves,stats0) = ex.lazy_smp_2(&ts);
    let t1 = t0.elapsed();
    let t2 = t1.as_secs_f64();

    let best   = res.get_result().unwrap();
    let scores = res.get_scores().unwrap_or_default();

    // for m in best.moves.iter() { eprintln!("\t{:?}", m); }
    // eprintln!("\nBest move = {:>8} {:?}", best.score, best.moves[0]);
    eprintln!("\nBest move = {:>8} {:?}", best.score, best.mv);
    println!("explore lazy_smp_negamax (depth: {}) done in {:.3} seconds.",
             stats0.max_depth, t2);

    stats0.print(t1);

}

#[cfg(feature = "syzygy")]
#[allow(unreachable_code)]
fn main_syzygy() {
    use rchess_engine_lib::syzygy::*;
    use rchess_engine_lib::syzygy::sz_format::*;

    // init_logger();

    let dir = "/home/me/code/rust/rchess/tables/syzygy/";

    // SyzygyBase::load_dir(&ts, &dir).unwrap();
    // SyzygyBase::load_dir(&dir).unwrap();

    // let m = Material::from_str("KBNvK");
    // eprintln!("m = {:?}", m);

    let mut tb = SyzygyTB::new();
    tb.add_directory(&dir).unwrap();
    // tb.add_file("/home/me/code/rust/rchess/tables/syzygy/KBNvK.rtbw").unwrap();
    // tb.add_file("/home/me/code/rust/rchess/tables/syzygy/KBNvK.rtbz").unwrap();

    // let fen = "8/8/8/8/B7/N7/K2k4/8 b - - 0 1";
    // let fen = "5BrN/8/8/8/8/2k5/8/2K5 b - -";
    // let fen = "5B2/8/6N1/8/8/2k5/8/2K5 b - - 0 2";
    // let fen = "5B1N/8/8/8/8/2k5/8/2K3r1 w - - 1 2";

    // let fen = "3qk3/8/8/8/8/8/8/4K3 w - - 0 1";

    // let fen = "8/6B1/8/8/B7/8/K1pk4/8 b - - 0 1";
    // let fen = "8/6B1/8/8/B7/8/3k4/K1n5 b - - 1 2";

    let fen = "8/8/8/2k5/7R/7P/7K/6R1 w - - 0 1"; // syzygy error
    // let fen = "8/8/8/2k5/7R/7P/7K/R7 w - - 0 1"; // works?

    let ts = Tables::read_from_file_def().unwrap();
    let g = Game::from_fen(&ts, &fen).unwrap();

    // let mut g = g.flip_sides(&ts);
    // let _ = g.init_gameinfo_mut(&ts).unwrap();
    // let _ = g.recalc_gameinfo_mut(&ts).unwrap();

    // eprintln!("g = {:?}", g);

    // let k0 = tb.probe_ab_no_ep(&ts, &g, Wdl::Loss, Wdl::Loss);
    // eprintln!("k0 = {:?}", k0);

    // tb.fathom(&ts, &g).unwrap();

    // let mv = Move::Promotion { from: "C2".into(), to: "C1".into(), new_piece: Knight };
    // // let mv = Move::Promotion { from: "C2".into(), to: "C1".into(), new_piece: Queen };
    // let g2 = g.make_move_unchecked(&ts, mv).unwrap();
    // eprintln!("g2 = {:?}", g2);
    // let g = g2;

    let k0 = tb.probe_wdl(&ts, &g);
    eprintln!("k0 = {:?}", k0);
    let k1 = tb.probe_dtz(&ts, &g);
    eprintln!("k1 = {:?}", k1);
    let k2 = tb.best_move(&ts, &g).unwrap();
    eprintln!("k2 = {:?}", k2.map(|x| x.0));

    // let wdl = tb.probe(&ts, &g).unwrap();
    // // eprintln!("wdl = {:?}", wdl.wdl);
    // // eprintln!("wdl = {:?}", wdl.state);

    // let k2 = tb.probe_dtz_table(&g, wd);
    // eprintln!("k2 = {:?}", k2);

    // // let c0 = Coord(1,2);
    // let c0 = Coord::from("F1");
    // let c1 = c0.flip_diagonal();
    // // let c2 = Coord::from(usize::from(c0) ^ 0b000_111);
    // let c2: u8 = c0.into();
    // // let c2: u8 = Coord::flip_horiz_int(c2);
    // let c2: u8 = Coord::flip_diagonal_int(c2);
    // let c2: Coord = c2.into();
    // eprintln!("c0 = {:?}", c0);
    // eprintln!("c1 = {:?}", c1);
    // // eprintln!("c2 = {:?}", c2);

}

#[allow(unreachable_code)]
fn main_simd() {
    use nalgebra as na;
    use ndarray as nd;
    use rand::{prelude::{StdRng,SliceRandom},Rng,SeedableRng};
    use ndarray_rand::RandomExt;
    use rand::distributions::{Uniform,uniform::SampleUniform};

    use rchess_engine_lib::sf_compat::layers::ceil_to_multiple;

    let mut rng: StdRng = SeedableRng::seed_from_u64(1234u64);
    // let dist0 = Uniform::new(0,1);
    // let dist1 = Uniform::new(i16::MIN,i16::MAX);

    // const OS: usize = 8;
    // const IS: usize = 1024 * 2;
    const IS: usize = 8;
    const OS: usize = 32;
    const ISP: usize = 32;

    use rchess_engine_lib::sf_compat::{Layer0,Layer1,NNAffine,NNLayer,NNInput};

    // type L0 = NNInput<IS>;
    // type L1 = NNAffine::<L0, OS, IS>;
    // let mut layer0 = L0::new();
    // let mut layer1 = NNAffine::<L0, OS, IS>::new(layer0);

    type L2 = NNAffine::<Layer1, OS, IS>;

    let mut layer0 = Layer0::new();
    let mut layer1 = Layer1::new(NNAffine::new(layer0));
    let mut layer2 = L2::new(layer1);

    // layer1.biases  = Aligned(array_init::array_init(|_| rng.gen_range(-10..10)));
    // let ws: [i8; IS * OS] = array_init::array_init(|_| rng.gen_range(-127..127));
    // layer1.weights = Aligned(ws.to_vec());

    layer2.biases = Aligned(array_init::array_init(|x| (x as i32 % 20) - 10));
    let ws: [i8; ISP * OS] = array_init::array_init(|x| (x as i8 % 20) - 10);
    layer2.weights = Aligned(ws.to_vec());

    // for i in 0..layer1.weights.len() {
    //     let i2 = L1::_get_weight_index(i);
    //     layer1.weights[i2] = ws[i];
    // }

    // eprintln!("layer2.weights.len() = {:?}", layer2.weights.len());
    // eprintln!("layer2.biases.len() = {:?}", layer2.biases.len());
    // eprintln!("layer2.buffer.len() = {:?}", layer2.buffer.len());

    // let input: [u8; IS * 2] = array_init::array_init(|_| rng.gen_range(0..2));
    // let input: [u8; IS] = array_init::array_init(|x| x as u8 % 2);
    let input: [u8; 2048] = array_init::array_init(|x| x as u8 % 2);
    // let input: Aligned<A64,_> = Aligned(input.clone());
    let mut output = [0i32; OS];

    // let v = layer2._propagate_avx2_small(input.as_ref());
    // eprintln!("v[0..5] = {:?}", &v[0..5]);

    // // let v = layer2._propagate_avx2_large(input.as_ref());
    // let v = layer2._propagate_avx2_small_nosimd(input.as_ref());
    // eprintln!("v[0..5] = {:?}", &v[0..5]);

    // simd_mm_0::<IS,OS>(input.as_ref(), &layer2.weights, layer2.biases.as_ref(), &mut output);
    // eprintln!("&output[0..5] = {:?}", &output[0..5]);
    // // let sum: i32 = output.iter().sum();
    // // eprintln!("sum = {:?}", sum);

    // simd_mm_1::<IS,OS>(input.as_ref(), &layer1.weights, layer1.biases.as_ref(), &mut output);
    // let sum: i32 = output.iter().sum();
    // eprintln!("sum = {:?}", sum);

    // return;

    // // XXX: stockfish = 3.75 M / sec
    // timer_loop!(1_000_000, {
    //     // layer1.propagate(input.as_ref());
    //     layer2._propagate_avx2_small(input.as_ref());
    //     // simd_mm_0::<IS,OS>(&input, &layer1.weights, layer1.biases.as_ref(), &mut output);
    //     // simd_mm_1::<IS,OS>(&input, &layer1.weights, layer1.biases.as_ref(), &mut output);
    // });

    // timer_loop!(1_000_000, {
    //     layer2._propagate_avx2_small_nosimd(input.as_ref());
    // });

    // return;

    let input: [u8; IS]   = array_init::array_init(|_| rng.gen_range(0..2));
    let weights: Vec<i8>  = (0..OS * IS).map(|_| rng.gen_range(-10..10)).collect();
    // let input: [i32; IS]   = array_init::array_init(|_| rng.gen_range(0..2));
    // let weights: Vec<i32>  = (0..OS * IS).map(|_| rng.gen_range(-10..10)).collect();

    let weights2 = {
        let mut weights2 = [[0; IS]; OS];
        for i in 0..OS {
            let offset = i * IS;
            weights2[i].copy_from_slice(&weights[offset..offset+IS]);
        };
        weights2
    };

    let weights3 = {
        let mut ws3 = [[0; OS]; IS];
        for x in 0..OS {
            for y in 0..IS {
                let k = weights2[x][y];
                ws3[y][x] = k;
            }
        }
        ws3
    };

    // let input: [i32; IS] = array_init::array_init(|k| input[k] as i32);
    // let weights: Vec<i32> = weights.into_iter().map(|x| x as i32).collect();

    // let biases: [i32; OS] = array_init::array_init(|_| rng.gen_range(-100..100));
    let biases     = [0i32; OS];
    let mut output = [0i32; OS];

    // let input: [i32; ISP] = array_init::array_init(|_| rng.gen_range(0..2));
    // let weights: Vec<i32> = (0..OS * IS).map(|_| rng.gen_range(-10..10)).collect();
    // let biases: [i32; OS] = array_init::array_init(|_| rng.gen_range(-100..100));

    use rchess_engine_lib::simd_test::*;

    use std::arch::x86_64::{__m256i,__m128i};
    use safe_arch::*;
    use rchess_engine_lib::simd_utils::safe_arch::*;

    const N: usize = 2048;
    const K: usize = 1_000_000;

    let xs1: [u8; N] = array_init::array_init(|x| rng.gen_range(0..2));
    let xs2: [i8; N] = array_init::array_init(|x| rng.gen_range(-10..10));
    // let xs1: [u8; 32] = array_init::array_init(|x| rng.gen_range(0..2));
    // let xs2: [i8; 32] = array_init::array_init(|x| rng.gen_range(-10..10));

    let ys1: [i32; N] = array_init::array_init(|x| xs1[x] as i32);
    let ys2: [i32; N] = array_init::array_init(|x| xs2[x] as i32);


    // let xs1: [u8; N] = array_init::array_init(|x| rng.gen_range(0..10));
    // let xs1: [u8; 16] = array_init::array_init(|x| rng.gen_range(0..10));

    let xs1: [u8; 2048] = array_init::array_init(|x| rng.gen_range(0..10));

    // let xs1 = xs1.to_vec();

    // let xs1: [u8; 512 + 256 + 64 + 32 + 0] = array_init::array_init(|x| rng.gen_range(0..10));
    // // XXX: slice: 865 = segfault, 864 fine

    // let xs1: [u8; 512 + 128 + 64 + 32 + 1] = array_init::array_init(|x| rng.gen_range(0..10));
    // // // XXX: array: 737 = segfault, 736 fine

    use aligned::{Aligned,A8,A16,A64};

    // let xs: &[u8] = &xs1;

    // let mut res1 = [0; 16];
    // res1.copy_from_slice(&xs1[0..16]);
    // let res1 = m128i::from(res1);
    // eprintln!("res1 = {:?}", bytemuck::cast::<m128i,[i8;16]>(res1));

    // let ws: Aligned<A64,_> = Aligned(xs1);
    let ws = xs1.clone();

    // // let a: &[m128i] = unsafe {
    // let a = unsafe {

    //     // let xs = Wrapper(xs);
    //     // std::mem::transmute(xs)

    //     // let ptr = xs.as_ptr() as *const m128i;
    //     // std::slice::from_raw_parts(ptr, 1)

    //     // let (a,b,c) = xs.align_to::<m128i>();
    //     let (a,b,c) = xs.align_to::<__m128i>();
    //     eprintln!("a.len() = {:?}", a.len());
    //     eprintln!("b.len() = {:?}", b.len());
    //     eprintln!("c.len() = {:?}", c.len());
    //     b

    // };

    // let k0 = std::mem::align_of_val(&xs);
    // eprintln!("k0 = {:?}", k0);

    let ws1: &[u8] = ws.as_ref();

    // let a: &[m128i] = unsafe {
    //     let ptr = ws1.as_ptr() as *const m128i;
    //     std::slice::from_raw_parts(ptr, ws1.len() / 16)
    // };

    // let a: &[m256i] = unsafe {
    //     let ptr = ws1.as_ptr() as *const m256i;
    //     std::slice::from_raw_parts(ptr, ws1.len() / 32)
    // };

    // let a = unsafe { cast_slice_to_m256i(ws1.as_ref()) };
    let a = unsafe { cast_slice_to_m128i(ws1.as_ref()) };

    // let res0 = m256_haddx4(sum0, sum0, sum1, sum1, bias);

    // let mut res0 = m256i::from([0i32; 8]);
    // m256_add_dpbusd_epi32x2(&mut res0, a0, b0, a1, b1);

    let mut res0 = [0u8; 16];
    res0.copy_from_slice(&ws1[0..16]);
    let res0 = m128i::from(res0);

    // let res0 = a[0];
    eprintln!("res0 = {:?}", bytemuck::cast::<m128i,[i8;16]>(res0));
    // eprintln!("res1 = {:?}", bytemuck::cast::<m128i,[i8;16]>(res1));
    // eprintln!("res0 = {:?}", bytemuck::cast::<m256i,[i16;16]>(res0));
    // eprintln!("res1 = {:?}", bytemuck::cast::<m256i,[i16;16]>(res1));

    return;

    let mut res = 0;
    timer!({
        for _ in 0..K {
            res = dot_product_basic(&xs1, &xs2);
        }});
    eprintln!("res = {:?}", res);

    let t0 = Instant::now();
    for _ in 0..K {
        res = dot_product2(&ys1, &ys2);
    }
    eprintln!("res = {:?}", res);
    let t1 = t0.elapsed().as_secs_f64();
    eprintln!("finished in {:.3} seconds", t1);

    let t0 = Instant::now();
    for _ in 0..K {
        res = dot_product0(&xs1, &xs2);
    }
    eprintln!("res = {:?}", res);
    let t1 = t0.elapsed().as_secs_f64();
    eprintln!("finished in {:.3} seconds", t1);

    return;

    let xs: [u8; 32] = array_init::array_init(|x| x as u8);
    let xs2: [i8; 32] = array_init::array_init(|x| x as i8);
    // let xs2: [i8; 32] = array_init::array_init(|x| 32 + x as i8);

    // let mut xs: [u8; 32] = array_init::array_init(|x| 0);
    // let mut xs2: [i8; 32] = array_init::array_init(|x| 0);

    // let mut sum = 0;
    // for i in 0..32 {
    //     sum += xs2[i] as i32 * xs[i] as i32;
    // }
    // eprintln!("sum = {:?}", sum);

    let k0 = m256i::from(xs);
    // let k1 = m256i::from(xs2);

    // let res0 = unpack_low_i8_m256i(k0, k0);
    // // eprintln!("res0 = {:?}", bytemuck::cast::<m256i,[i8;32]>(res0));
    // let res0 = shr_imm_i16_m256i::<8>(res0);
    // let res1 = mul_i16_keep_low_m256i(res0,res0);
    // eprintln!("res1 = {:?}", bytemuck::cast::<m256i,[i16;16]>(res1));

    // return;

    // let res0 = mul_u8i8_add_horizontal_saturating_m256i(k0, k1);
    // let res1 = mul_i16_horizontal_add_m256i(res0, set_splat_i16_m256i(1));

    // let res2 = add_horizontal_i32_m256i(res0, m256i::default());

    // let res1 = m128i::from([123,0,0,0]);
    // // let res1 = set_i32_m128i_s(123);
    // eprintln!("res1 = {:?}", bytemuck::cast::<m128i,[i32;4]>(res1));

    // eprintln!("res1 = {:?}", bytemuck::cast::<m256i,[i32;8]>(res1));
    // eprintln!("res2 = {:?}", bytemuck::cast::<m256i,[i32;8]>(res2));

    // let k1 = m256i::default();
    // let k2 = set_splat_i16_m256i(1);

    // let res0 = mul_u8i8_add_horizontal_saturating_m256i(k0, k1);

    // let res: [i16; 16] = res0.into();
    // eprintln!("res0 = {:?}", res);

    // let res1 = mul_i16_horizontal_add_m256i(res0, set_splat_i16_m256i(1));

    // let res: [i16; 16] = res1.into();
    // eprintln!("res1 = {:?}", res);

    // xs0:  a, b, c, d     u8, input
    // xs1:  e, f, g, h     i8, weights

    // 1:    i0 = a * e, i1 = b * f   i16
    // 2:    i0 + i1

    // res:  j0 = (a*e) + (b*f), j1 = (c*g) + (d*h)

    // res:  (j0 * j1) + (j2 * j3)


    // let k4: [i8;32] = result1.into();
    // eprintln!("k4 = {:?}", k4);

    // return;

    // const NUM_RUNS: usize = 1_000_000;
    // const NUM_RUNS: usize = 500_000;
    const NUM_RUNS: usize = 500_000;

    let t0 = Instant::now();
    for _ in 0..NUM_RUNS {
        simd_mm_0::<IS,OS>(&input, &weights, &biases, &mut output);
    }
    let t1 = t0.elapsed().as_secs_f64();
    eprintln!("finished in {:.3} seconds", t1);
    eprintln!("sum = {:?}", output.iter().sum::<i32>()); // -2055
    output.fill(0);

    // simd_mm_0::<IS,OS>(&input, &weights, &biases, &mut output);
    // eprintln!("output = {:?}", output);
    // eprintln!("sum = {:?}", output.iter().sum::<i32>()); // -2055

    // simd_mm_2::<IS,OS>(&input, &weights2, &biases, &mut output);

    let t0 = Instant::now();
    for _ in 0..NUM_RUNS {
        simd_mm_2::<IS,OS>(&input, &weights2, &biases, &mut output);
        // simd_mm_2::<IS,OS>(&input, &weights3, &biases, &mut output);
    }
    let t1 = t0.elapsed().as_secs_f64();
    eprintln!("finished in {:.3} seconds", t1);

    eprintln!("output = {:?}", output);
    eprintln!("sum = {:?}", output.iter().sum::<i32>()); // -2055

    return;

    // let x: [i32; 4] = array_init::array_init(|_| rng.gen_range(0..2));
    // let a: [i32; 4*4] = array_init::array_init(|_| rng.gen_range(-10..10));
    // let mut a2: [[i32; 4]; 4] = [[0; 4]; 4];
    // for x in 0..4 {
    //     for y in 0..4 {
    //         // let k0 = x + y * 4;
    //         let k1 = y + x * 4;
    //         a2[x][y] = a[k1]
    //     }
    // }
    // let b: [i32; 4] = [0; 4];
    // let mut result = [0i32; 4];

    // // eprintln!("x = {:?}", x);
    // // eprint_self!(&a[0..4]);
    // // eprint_self!(&a[4..8]);
    // // eprint_self!(&a[8..12]);
    // // eprint_self!(&a[12..16]);
    // // eprintln!();
    // // eprint_self!(&a2[0]);
    // // eprint_self!(&a2[1]);
    // // eprint_self!(&a2[2]);
    // // eprint_self!(&a2[3]);
    // // eprintln!();
    // // correct = [-10, 11, -9, -19]
    // simd_mm_0::<4,4>(&x,&a,&b,&mut result);
    // eprintln!("result 0 = {:?}", result);
    // simd_mm_2::<4,4>(&x,&a2,&b,&mut result);
    // eprintln!("result 1 = {:?}", result);
    // return;

    // let t0 = Instant::now();
    // for _ in 0..NUM_RUNS {
    //     simd_mm_0::<IS,OS>(&input, &weights, &biases, &mut output);
    // }
    // let t1 = t0.elapsed().as_secs_f64();
    // eprintln!("finished in {:.3} seconds", t1);
    // let sum: i32 = output.iter().sum();
    // eprintln!("sum = {:?}", sum); // -2055

    // return;

    // eprintln!("weights[0..8] = {:?}", &weights[0..8]);
    // eprintln!("weights[-8..] = {:?}", &weights[weights.len() - 8..]);
    // eprint_self!(&weights2[0][0..8]);
    // eprint_self!(&weights2[OS-1][IS - 8 ..]);

    // simd_mm_0::<IS,OS>(&input, &weights, &biases, &mut output);
    // let sum: i32 = output.iter().sum();
    // eprintln!("sum = {:?}", sum); // -2055

    // simd_mm_2::<IS,OS>(&input, &weights2, &biases, &mut output);
    // let sum: i32 = output.iter().sum();
    // eprintln!("sum = {:?}", sum); // -2055

    // simd_nd_mm_3::<IS,OS>(&input, &weights, &biases, &mut output);

    // let t0 = Instant::now();
    // for _ in 0..NUM_RUNS {
    //     simd_mm_2::<IS,OS>(&input, &weights2, &biases, &mut output);
    //     // simd_mm_2::<IS,OS>(&input, &weights, &biases, &mut output);
    // }
    // let t1 = t0.elapsed().as_secs_f64();
    // eprintln!("finished in {:.3} seconds", t1);

    // eprintln!("output[0..8] = {:?}", &output[0..8]); // -482, -165, -278
    // let sum: i32 = output.iter().sum();
    // eprintln!("sum = {:?}", sum); // -2055

    use std::simd::*;

    use nd::ShapeBuilder;

    // for i in 0..OS {
    //     let offset = i * ISP;
    //     let mut sum = biases[i];
    //     // let mut sum = 0;
    //     for j in 0..IS {
    //         let x = input[j] as i32;
    //         sum += weights[offset + j] as i32 * x;
    //     }
    //     buffer[i] = sum;
    // }
    // eprintln!("buffer[0] = {:?}", buffer[0]);
    // let s0: i32 = buffer.iter().sum();
    // eprintln!("s0 = {:?}", s0);
    // eprintln!("buffer = {:?}", buffer);
    // eprintln!();
    // eprintln!("biases = {:?}", biases);

    // let input: nd::Array2<u8> = nd::Array2::from_shape_vec((ISP, 1), input.to_vec()).unwrap();
    // let weights: nd::Array2<i8> = nd::Array2::from_shape_vec((IS,OS).f(), weights).unwrap();
    // let biases: nd::Array2<i32> = nd::Array2::from_shape_vec((OS, 1), biases.to_vec()).unwrap();
    // let weights = weights.reversed_axes();

    // let input   = input.map(|x| *x as i32);
    // let weights = weights.map(|x| *x as i32);
    // let biases  = biases.map(|x| *x as i32);
    // let result = weights.dot(&input);
    // let result = result + &biases;

    // eprintln!("result = {}", result.t());
    // eprintln!("result.shape() = {:?}", result.shape());
    // eprintln!("result[(0,0)] = {:?}", result[(0,0)]);
    // eprintln!("result.sum() = {:?}", result.sum());

    // use std::collections::hash_map::DefaultHasher;
    // use std::hash::{Hash, Hasher};
    // let mut s = DefaultHasher::new();
    // (&buffer).hash(&mut s);
    // let hash = s.finish();
    // eprintln!("hash = {:?}", hash);

    // let a = f32x4::splat(10.0);
    // let b = f32x4::from_array([1.0, 2.0, 3., 4.]);
    // eprintln!("a + b = {:?}", a + b);

}

#[allow(unreachable_code)]
fn main_mnist() {

    use nalgebra::{SMatrix,SVector,Matrix,Vector,matrix,vector,DMatrix,DVector};
    use nalgebra as na;
    use rand::prelude::{StdRng,SliceRandom};
    use rand::{Rng,SeedableRng};

    use rchess_engine_lib::brain::*;
    use rchess_engine_lib::brain::nnue::*;
    use rchess_engine_lib::brain::types::*;

    use mnist::*;

    let data = MnistBuilder::new()
        .base_path("mnist")
        .label_format_digit()
        // .training_set_length(50_000)
        // .validation_set_length(10_000)
        // .test_set_length(10_000)
        .finalize()
        .normalize();
    let mut trn_imgs: Vec<SVector<f32,784>> = data.trn_img
        .chunks_exact(28 * 28)
        .map(|x| SVector::<f32,784>::from_column_slice(x))
        .collect::<Vec<_>>();
    let mut test_imgs: Vec<SVector<f32,784>> = data.tst_img
        .chunks_exact(28 * 28)
        .map(|x| SVector::<f32,784>::from_column_slice(x))
        .collect::<Vec<_>>();
    let mut val_imgs: Vec<SVector<f32,784>> = data.val_img
        .chunks_exact(28 * 28)
        .map(|x| SVector::<f32,784>::from_column_slice(x))
        .collect::<Vec<_>>();

    let mut corrects = data.trn_lbl.iter()
    // let mut corrects = data.tst_lbl.iter()
        .map(|x| {
            let mut v = SVector::<f32,10>::zeros();
            v[*x as usize] = 1.0;
            v
        }).collect::<Vec<_>>();

    // trn_imgs.truncate(1000);
    // corrects.truncate(100);
    let mut trn_lbl = data.trn_lbl.clone();
    trn_lbl.truncate(100);

    // let mut nn0 = MNNetwork::new_range(2, (-1.0, 1.0));
    // let mut nn1 = MNNetwork::new_range(2, (-1.0, 1.0));

    // nn.write_to_file("mnist.bin").unwrap();

    let test_data = test_imgs.into_iter().zip(data.tst_lbl.into_iter()).collect::<Vec<_>>();

    // let val_data  = &test_data[..test_data.len() / 2].to_vec();
    // let test_data = &test_data[test_data.len() / 2..].to_vec();

    let mut trn_imgs2: Vec<DVector<f32>> = data.trn_img
        .chunks_exact(28 * 28)
        .map(|x| DVector::<f32>::from_vec(x.to_vec()))
        .collect::<Vec<_>>();
    let mut corrects2 = data.trn_lbl.iter()
        .map(|x| {
            let mut v = DVector::<f32>::zeros(10);
            v[*x as usize] = 1.0;
            v
        }).collect::<Vec<_>>();

    // let mut ins = trn_imgs.iter().zip(trn_lbl.clone()).collect::<Vec<_>>();
    // let mut ins: Vec<_> = trn_imgs.clone().into_iter().zip(corrects.clone()).collect();
    let ti = trn_imgs.clone();
    let mut ins: Vec<_> = ti.iter().zip(corrects.clone()).collect();

    // return;

    // let mut rng = rand::thread_rng();
    let mut rng: StdRng = SeedableRng::seed_from_u64(1234u64);

    let ins2: Vec<(DVector<f32>,DVector<f32>)> = trn_imgs2.into_iter().zip(corrects2.clone()).collect();

    let mm = (-1.0, 1.0);
    // let mm = (0.0, 1.0);

    let mut nn2: DNetwork<f32,784,10> = DNetwork::new_range(vec![784,16,16,10], mm);
    // let mut nn2: DNetwork<f32,784,10> = DNetwork::new_range(vec![784,30,10], mm);

    let lr = 0.001;
    // let lr = 0.5;
    // let lr = 10.0;

    const BATCH_SIZE: usize = 100;
    const EPOCHS: usize = 400;
    let ksize = 50;

    // test_mnist2(&nn2, &test_data, true);

    // let s0 = nn2.run(&ins2[0].0);
    // eprintln!("s0 = {:?}", s0);
    // return;

    let ws0 = nn2.weights[0].clone();

    let t0 = Instant::now();
    println!("Starting dyn...");
    for k in 0..EPOCHS {

        // ins.shuffle(&mut rng);
        // let mut ins3 = ins2.clone();
        // ins3.truncate(BATCH_SIZE);

        let ins3 = ins2.choose_multiple(&mut rng, BATCH_SIZE).cloned().collect::<Vec<_>>();

        nn2.backprop_mut_matrix(&ins3, lr);
        // break;

        // let acc = test_mnist2(&nn2, &val_data, false);
        // eprintln!("acc = {:.2}", acc);

        if k % ksize == 0 {
            test_mnist2(&nn2, &test_data, true);
        }

    }
    let t1 = t0.elapsed().as_secs_f64();
    println!("finished {} runs in {:.3} seconds, avg {:.3} s/run", EPOCHS, t1, t1 / EPOCHS as f64);

    let ws1 = nn2.weights[0].clone();

    eprintln!("ws0 == ws1 = {:?}", ws0 == ws1);

    test_mnist2(&nn2, &test_data, true);


    // let (img,lbl) = &ins2[4];
    // let pred = nn2.run(&img);
    // let (c0,c1) = lbl.iter().enumerate()
    //     .max_by(|a,b| a.1.partial_cmp(&b.1).unwrap())
    //     .unwrap();
    // let (k0,k1) = pred.iter().enumerate()
    //     .max_by(|a,b| a.1.partial_cmp(&b.1).unwrap())
    //     .unwrap();
    // eprintln!("correct = {:?}", c0);
    // eprintln!("pred    = {:?}", k0);

    // eprintln!("pred[4] = {:?}", pred[4]);
    // eprintln!("pred[7] = {:?}", pred[7]);

    // use image::{DynamicImage,Luma};
    // let img = img.iter()
    //     .map(|x| (x * 255.0).clamp(0.0, 255.0) as u8)
    //     .collect::<Vec<u8>>();
    // let i: image::ImageBuffer<image::Luma<u8>,Vec<u8>> = image::ImageBuffer::from_vec(28, 28, img).unwrap();
    // i.save("char.png").unwrap();

    return;

    // let t0 = Instant::now();
    // println!("Starting old...");
    // for k in 0..EPOCHS {
    //     // ins.shuffle(&mut rng);
    //     // let xs: &[(&SVector<f32,784>,SVector<f32,10>)] = &ins[0..BATCH_SIZE];
    //     // let xs = xs.to_vec();
    //     // let (imgs,lbls): (Vec<&SVector<f32,784>>,Vec<SVector<f32,10>>) = xs.into_iter().unzip();
    //     let xs = ins.choose_multiple(&mut rng, BATCH_SIZE).cloned();
    //     let (imgs,lbls): (Vec<&SVector<f32,784>>,Vec<SVector<f32,10>>) = xs.unzip();
    //     nn0.backprop_mut(imgs, lbls, lr);
    //     if k % ksize == 0 {
    //         // let t1 = t0.elapsed().as_secs_f64();
    //         // println!("finished {} runs in {:.3} seconds, avg {:.3} s/run", k, t1, t1 / k as f64);
    //         // t0 = Instant::now();
    //         eprint!("old:    ");
    //         test_mnist(&nn0, test_data.clone(), Some(1000));
    //         // nn.write_to_file("mnist.bin",Some("mnist-2.bin")).unwrap();
    //         // nn.write_to_file("mnist.bin",None).unwrap();
    //     }
    // }
    // let t1 = t0.elapsed().as_secs_f64();
    // println!("finished {} runs in {:.3} seconds, avg {:.3} s/run", EPOCHS, t1, t1 / EPOCHS as f64);

    // let mut ins2 = ins.clone();
    // ins2.truncate(BATCH_SIZE);
    // nn1.backprop_mut_matrix::<BATCH_SIZE>(ins2, lr);

    return;

    // let mut t0 = Instant::now();
    // println!("Starting matrix...");
    // for k in 0..EPOCHS {
    //     ins.shuffle(&mut rng);
    //     let mut ins2 = ins.clone();
    //     ins2.truncate(BATCH_SIZE);
    //     nn1.backprop_mut_matrix::<BATCH_SIZE>(&ins2, lr);
    //     if k % ksize == 0 {
    //         // let t1 = t0.elapsed().as_secs_f64();
    //         // println!("finished {} runs in {:.3} seconds, avg {:.3} s/run", k, t1, t1 / k as f64);
    //         // t0 = Instant::now();
    //         eprint!("matrix: ");
    //         test_mnist(&nn1, test_data.clone(), Some(1000));
    //     }
    // }
    // let t1 = t0.elapsed().as_secs_f64();
    // println!("finished {} runs in {:.3} seconds, avg {:.3} s/run", EPOCHS, t1, t1 / EPOCHS as f64);

    // nn.write_to_file("mnist.bin").unwrap();

    // // test_mnist(&nn, test_data.clone(), Some(1000));
    // test_mnist(&nn, test_data.clone(), None);
    // // test_mnist(&nn, test_imgs.clone(), data.tst_lbl.clone(), Some(1000));
    // // test_mnist(&nn, trn_imgs.clone(), trn_lbl.clone(), Some(1000));

    return;
}

#[allow(unreachable_code)]
fn main_tuning() {
    use rchess_engine_lib::texel::*;
    use rchess_engine_lib::qsearch::*;
    use rchess_engine_lib::pawn_hash_table::*;
    use rchess_engine_lib::pgn::*;

    let ts = Tables::read_from_file_def().unwrap();

    let ob = OpeningBook::read_from_file(&ts, "tables/Perfect_2021/BIN/Perfect2021.bin").unwrap();

    // let path = "/home/me/code/rust/rchess/training_data/test_5.bin";
    // let path = "/home/me/code/rust/rchess/training_data/set1/depth5_games500_4.bin";
    // let path = "/home/me/code/rust/rchess/training_data/depth5_games100_1.bin";
    // let path = "/home/me/code/rust/rchess/training_data/depth5_test_1.bin";
    // let path = "/home/me/code/rust/rchess/training_data/set2/depth5_games500_1.bin";

    let evpath = "/home/me/code/rust/rchess/evparams.bin";

    if !true {

        // let mut ev = EvalParams::default();
        // ev.psqt.print_table(Rook).unwrap();
        // println!();

        let (ev_mid,ev_end) = EvalParams::read_evparams(evpath).unwrap();

        for pc in Piece::iter_pieces() {
            eprintln!("{:?}", pc);
            // ev_mid.psqt.print_table(pc).unwrap();
            ev_end.psqt.print_table(pc).unwrap();
            println!();
        }
        // ev_mid.psqt.print_table(Rook).unwrap();

        // eprintln!("ev == ev_mid = {:?}", ev == ev_mid);

        return;
    }

    // // let mut ev_mid = EvalParams::default();
    // // let mut ev_end = EvalParams::default();
    // let mut ev_mid = EvalParams::empty();
    // let mut ev_end = EvalParams::empty();
    // ev_mid.mid = true;
    // ev_end.mid = false;

    // EvalParams::save_evparams(&ev_mid, &ev_end, evpath).unwrap();
    // return;

    // let ev0 = EvalParams::default();
    // ev0.psqt.print_table(Knight).unwrap();
    // println!();

    let (ev_mid,ev_end) = EvalParams::read_evparams(evpath).unwrap();

    // ev_mid.psqt.print_table(Knight).unwrap();
    // return;

    // filter quiet TrainingData
    if !true {
        let path = "/home/me/code/rust/rchess/training_data/set2/depth5_games500_1.bin";
        let path2 = &format!("{}-tmp", path);

        eprintln!("path = {:?}", path);
        eprintln!("path2 = {:?}", path2);

        let t0 = std::time::Instant::now();
        let tds = TrainingData::load_all(path, None).unwrap();
        eprintln!("finished in {:.3} seconds", t0.elapsed().as_secs_f64());

        eprintln!("tds.len() = {:?}", tds.len());

        let t0 = std::time::Instant::now();
        let tds2 = TrainingData::filter_quiet(&ts, &(ev_mid,ev_end), tds);
        eprintln!("finished in {:.3} seconds", t0.elapsed().as_secs_f64());

        eprintln!("tds2.len() = {:?}", tds2.len());

        return;
    }

    // let pgn_path = "./training_data/ficsgamesdb_2020_standard_nomovetimes_233317.pgn";
    // let pgn_path = "./training_data/fics_test2.pgn";
    // let pgn_path = "./training_data/fics_test.pgn";

    // let ps = load_pgns_tx(&ts, &mut exhelper, count, pgn_path).unwrap();

    let ph_factory = PHTableFactory::new();
    let ph_rw = ph_factory.handle();

    let mut g = Game::from_fen(&ts, STARTPOS).unwrap();
    let mut exhelper = exhelper_once(&g, g.state.side_to_move, &ev_mid, &ev_end, Some(&ph_rw), None);

    let fen_path = "./training_data/tuner/quiet-labeled.epd";

    // let count = Some(100000);
    let count = None; // 725 k

    let ps = load_labeled_fens(&ts, &mut exhelper, count, fen_path).unwrap();

    eprintln!("ps.len() = {:?}", ps.len());

    let k = 1.0;
    // let k: f64 = -0.1111f64;
    // let k = 1.3;
    // let k = find_k(&ts, &ps, &exhelper, true);
    eprintln!("k = {:?}", k);

    let t0 = std::time::Instant::now();
    let error = average_eval_error(&ts, &ps, &exhelper, Some(k));
    eprintln!("error = {:.5}", error);
    let t1 = t0.elapsed().as_secs_f64();
    eprintln!("finished one eval_error in {:.3} seconds", t1);

    // return;

    // if std::path::Path::new(&evpath).exists() {
    //     std::fs::rename(&evpath, &format!("{}.bak", evpath)).unwrap();
    //     eprintln!("evparams.bin already exists, renaming to evparams.bin.bak");
    //     // return;
    // }

    let count = 10;

    let (ev_mid2,ev_end2) = texel_optimize(
        &ts,
        &ps,
        &mut exhelper,
        &vec![],
        Some(count),
        Some(k),
        evpath);

    let error = average_eval_error(&ts, &ps, &exhelper, Some(k));
    eprintln!("error = {:.5}", error);

}

fn main_gensfen(count: u64, path: &str) {
    let mut rng: StdRng = SeedableRng::seed_from_u64(1234u64);

    let ts = Tables::read_from_file_def().unwrap();
    let ob = OpeningBook::read_from_file(&ts, "tables/Perfect_2021/BIN/Perfect2021.bin").unwrap();

    let t0 = Instant::now();

    // let count = 100;

    // let path = "/home/me/code/rust/rchess/training_data/test_6.bin";

    if std::path::Path::new(&path).exists() {
        eprintln!("path exists, exiting");
        return;
    }

    let ts = TDBuilder::new()
        .max_depth(5)
        .time(0.2)
        .num_threads(num_cpus::get())
        // .num_threads(12)
        // .num_positions(Some(1000))
        .num_positions(None)
        .do_explore(&ts, &ob, count, true, rng, true, path)
        .unwrap();
    println!("finished in {:.3} seconds", t0.elapsed().as_secs_f64());

}

#[allow(unreachable_code)]
fn main_nnue_train() {
    use rchess_engine_lib::brain::*;
    use rchess_engine_lib::brain::nnue::*;
    use rchess_engine_lib::brain::types::nnue::*;

    let ts = Tables::read_from_file_def().unwrap();
    let mut rng: StdRng = SeedableRng::seed_from_u64(1234u64);

    let nn_path = "./nnue.bin";
    let td_path = "./training_data/ficsgamesdb_2020_standard_tds.bin";

    let count = Some(10);

    let tds = TrainingData::load_all(td_path, count).unwrap();
    eprintln!("tds.len() = {:?}", tds.len());

    let mut nn = NNUE::new(White, &mut rng);

    let err = nn.mean_sq_error(&ts, &tds);
    eprintln!("err 0 = {:?}", err);

    let params = NNTrainingParams::new(10);

    for n in 0..1000 {
        let t0 = std::time::Instant::now();
        nn.train(&ts, &params, &mut rng, &tds);
        let t1 = t0.elapsed().as_secs_f64();
        eprintln!("finished {} in {:.3} seconds", n, t1);

        if n > 0 && n % 50 == 0 {
            let err = nn.mean_sq_error(&ts, &tds);
            eprintln!("err {:>3} = {:?}", n, err);
        }
    }

}

#[allow(unreachable_code)]
fn main_nnue() {
    use rchess_engine_lib::brain::*;
    use rchess_engine_lib::brain::nnue::*;
    use rchess_engine_lib::brain::types::nnue::*;
    use rchess_engine_lib::brain::types::*;

    let ts = Tables::read_from_file_def().unwrap();
    let mut rng: StdRng = SeedableRng::seed_from_u64(1234u64);

    if !true {
        use rchess_engine_lib::brain::networks2::*;
        use rchess_engine_lib::brain::autodiff::*;

        // fn f(x: f32) -> f32 {
        //     // 3.0 * x + 2.0
        //     5. * x * x + 4. * x + 1.
        // }

        // let k0 = f(2.0);
        // eprintln!("k0 = {:?}", k0);

        // let d = Dual::new(4.0, 1.0);
        // let d1 = d * Dual::constant(3.0);
        // let d2 = d1 + Dual::constant(2.0);

        // eprintln!("d1 = {:?}", d1);

        // eprintln!("d2 = {:?}", d2);

        // let x0 = Dual::new(3, 4);
        // let x1 = Dual::new(1, 2);
        // let k = x0 * x1;
        // eprintln!("k = {:?}", k);

        return;
    }

    let nn_path = "./nnue.bin";

    // let fen = "r1bqk2r/ppp1nppp/3p1b2/3P4/2B1R3/5N2/PP3PPP/R1BQ2K1 w kq - 0 12";
    // let fen = "8/k7/3p4/p2P1p2/P2P1P2/8/8/K7 w - - "; // Lasker-Reichhelm Position, Qt K a1b1
    // let mut g = Game::from_fen(&ts, fen).unwrap();

    let ob = OpeningBook::read_from_file(&ts, "tables/Perfect_2021/BIN/Perfect2021.bin").unwrap();

    // let mut s = OBSelection::new_random_seeded(12345);
    // let (mut g,_) = ob.start_game(&ts, None, &mut s).unwrap();

    // // let fen = "7k/4pp2/8/8/8/8/4PP2/7K w - - 0 1";
    // let fen = "4k3/4p3/8/8/8/8/4P3/4K3 w - - 0 1";
    // let mut g = Game::from_fen(&ts, fen).unwrap();
    // eprintln!("g.to_fen() = {:?}", g.to_fen());
    // eprintln!("g = {:?}", g);

    // let fen1 = "8/k7/3p4/p2P1p2/P2P1P2/8/8/K7 w - - "; // Lasker-Reichhelm Position, Qt K a1b1
    // let fen2 = "r1bqk2r/ppp1nppp/3p1b2/3P4/2B1R3/5N2/PP3PPP/R1BQ2K1 w kq - 0 12";

    // let fen1 = "r1bqk2r/ppp1nppp/3p1b2/3P4/2B1R3/5N2/PP3PPP/R1BQ2K1 w kq - 0 12"; // base
    // let fen2 = "r1bqk2r/ppp1nppp/3p1b2/3P4/2B1R3/8/PP3PPP/R1BQ2K1 w kq - 0 12"; // no knight
    // let fen3 = "r1bqk2r/ppp1nppp/3p1b2/3P4/2B1R3/2N2N2/PP3PPP/R1BQ2K1 w kq - 0 12"; // extra knight

    let fen1 = "4k3/3ppp2/8/8/8/8/3PPP2/4K3 w - - 0 1"; // base
    let fen2 = "4k3/2nppp2/8/8/8/8/3PPP2/4K3 w - - 0 1"; // enemy knight
    let fen3 = "4k3/3ppp2/8/8/8/8/2NPPP2/4K3 w - - 0 1"; // + knight

    // let corrects = vec![0.0, -0.5, 0.5];
    let corrects = vec![0, -500, 500];

    let mut g1 = Game::from_fen(&ts, fen1).unwrap();
    let mut g2 = Game::from_fen(&ts, fen2).unwrap();
    let mut g3 = Game::from_fen(&ts, fen3).unwrap();

    let n = 35;
    let t = 0.2;

    // let timesettings = TimeSettings::new_f64(0.0,t);
    // let mut ex1 = Explorer::new(g1.state.side_to_move, g1.clone(), n, timesettings);
    // ex1.load_syzygy("/home/me/code/rust/rchess/tables/syzygy/").unwrap();
    // let mut ex2 = ex1.clone();
    // ex2.update_game(g2.clone());

    let mut nn = NNUE::new(White, &mut rng);
    // let mut nn = NNUE::new(Black, &mut rng);

    // nn.save(nn_path).unwrap();

    // init_logger();

    // nn.init_inputs(&g);
    let e1 = nn.run_fresh(&g1);
    let e2 = nn.run_fresh(&g2);
    let e3 = nn.run_fresh(&g3);
    eprintln!("e1,e2,e3 = {:>8}, {:>8}, {:>8}", e1, e2, e3);

    // let mut m0 = Score::MIN;
    // let mut m1 = Score::MAX;

    let eta = 10;

    let t0 = std::time::Instant::now();
    for n in 0..500 {
        nn.backprop(Some(&g1), corrects[0], eta);
        nn.backprop(Some(&g2), corrects[1], eta);
        nn.backprop(Some(&g3), corrects[2], eta);

        if n % 50 == 0 {
            let e1 = nn.run_fresh(&g1);
            let e2 = nn.run_fresh(&g2);
            let e3 = nn.run_fresh(&g3);
            eprintln!("e1,e2,e3 = {:>8?}, {:>8?}, {:>8?}", e1, e2, e3);
        }

    }
    let t1 = t0.elapsed().as_secs_f64();
    eprintln!("finished in {:.3} seconds", t1);

    let e1 = nn.run_fresh(&g1);
    let e2 = nn.run_fresh(&g2);
    let e3 = nn.run_fresh(&g3);
    eprintln!("e1,e2,e3 = {:>8?}, {:>8?}, {:>8?}", e1, e2, e3);

    // loop {
    //     if let (Some((mv,res)),_) = ex.explore(&ts) {
    //         if let Ok(g2) = g.make_move_unchecked(&ts, mv) {
    //             g = g2;
    //             ex.update_game(g.clone());
    //             eprintln!("g = {:?}", g);
    //             let nn_eval = nn.update_move(&g, true).unwrap();
    //             m0 = m0.max(nn_eval);
    //             m1 = m1.min(nn_eval);
    //             eprintln!("res.score = {:?}", res.score);
    //             eprintln!("nn_eval   = {:?}", nn_eval);
    //             // eprintln!("(m0,m1) = {:?}", (m0,m1));
    //             // let err = (res.score - nn_eval).checked_pow(2).unwrap();
    //             // eprintln!("err = {:?}", err);
    //         } else {
    //             println!("wat 0: {:?}", mv);
    //             break;
    //         }
    //     } else {
    //         println!("wat 1");
    //         break;
    //     }
    // }

}

#[allow(unreachable_code)]
fn main_nnue2() {
    use nalgebra as na;
    use na::{SMatrix,SVector,Matrix,Vector,matrix,vector,dmatrix,dvector,DVector,DMatrix};

    use ndarray as nd;
    use nd::Array2;
    use ndarray_rand::RandomExt;
    use ndarray_rand::rand_distr::Distribution;

    use rchess_engine_lib::brain::*;
    use rchess_engine_lib::brain::types::*;
    use rchess_engine_lib::brain::types::nnue::*;
    use rchess_engine_lib::brain::nnue::*;
    use rchess_engine_lib::brain::matrix::*;
    // use rchess_engine_lib::brain::accumulator::*;

    // init_logger();

    let mut rng: StdRng = SeedableRng::seed_from_u64(1234u64);

    // let ts = Tables::read_from_file_def().unwrap();
    // use rchess_engine_lib::brain::binpack::*;
    // let path = "/home/me/code/rust/rchess/training_data/generated_kifu.binpack";
    // return;

    let fen = STARTPOS;
    // let fen = "4k3/8/8/8/8/8/4P3/4K3 w - - 0 1";
    // let fen = "4k3/4p3/8/8/8/8/4P3/3QK3 w - - 0 1";

    // let dist = Uniform::new(i16::MIN,i16::MAX);
    // let dist = Uniform::new(-10,10);

    // let inputs_own       = nd::Array2::<i16>::random((NNUE_INPUT, 1), dist);
    // let inputs_other     = nd::Array2::<i16>::random((NNUE_INPUT, 1), dist);
    // let weights_in_own   = nd::Array2::<i16>::random((NNUE_L2,NNUE_INPUT), dist);
    // let weights_in_other = nd::Array2::<i16>::random((NNUE_L2,NNUE_INPUT), dist);

    // let dist0 = Uniform::new(0i16,2);

    // // let mut inputs = nd::Array2::<i16>::random_using((3,1), dist0, &mut rng);
    // let mut inputs = nd::array![[1],[0]];
    // let ws         = Array2::<i16>::random_using((2,2), dist, &mut rng);

    let fen = STARTPOS;
    // let fen     = "rnbqkb1r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    // let fen = "4k3/3pp3/8/8/8/8/3PP3/4K3 w - - 0 1";
    // let fen = "4k3/3pp3/8/8/8/8/3P1P2/4K3 w - - 0 1";
    let correct = 10;

    // let dnn = DNetwork::<f32,512,1>::_new_rng(vec![512,32,32,1], (-1.,1.), &mut rng);

    let ts = Tables::read_from_file_def().unwrap();

    let ob = OpeningBook::read_from_file(&ts, "tables/Perfect_2021/BIN/Perfect2021.bin").unwrap();
    // let ob = OpeningBook2::read_from_file("tables/Perfect_2021/ABK/Perfect2021.abk").unwrap();

    // let fen = "rn1qkb1r/1p3pp1/p2pbn2/4p2p/4P3/1NN1BP2/PPP3PP/R2QKB1R w KQkq - 0 9";
    // let fen = "rn1qkb1r/1p3pp1/p2pbn2/4p2p/4P3/1NN1BP2/PPP3PP/R2QKB1R w KQkq - 0 2";
    // let fen = "r2qkb1r/1p1n1p2/p2p1np1/3Pp2p/8/1N2BP2/PPPQ2PP/R3KB1R w KQkq - 0 2";

    let fen = "6k1/4Q3/8/8/8/5K2/8/8 w - - 6 4"; // Queen endgame, #4

    let mut g = Game::from_fen(&ts, fen).unwrap();
    // let mut g = Game::from_fen(&ts, STARTPOS).unwrap();
    // let mut nn = NNUE::new(White, &mut rng, dnn);
    // g.state.en_passant = Some("H6".into());

    // let path = "/home/me/code/rust/rchess/training_data/test_3.bin";

    // let tds: Vec<TrainingData> = TrainingData::load_all(path).unwrap();

    // eprintln!("tds.len() = {:?}", tds.len());

    // for td in tds.into_iter() {
    //     eprintln!("td.result = {:?}", td.result);
    //     eprintln!("td.moves.len() = {:?}", td.moves.len());
    //     let b: Vec<u8> = bincode::serialize(&td).unwrap();
    //     eprintln!("b.len() = {:?}", b.len());
    // }

    // let (_,opening) = ob.start_game(&ts, Some(6), &mut s).unwrap();

    // let k0 = TDBuilder::new()
    //     .with_opening(opening)
    //     .with_branch_factor(5)
    //     .with_max_depth(5)
    //     .with_time(0.5)
    //     .generate_single(&ts)
    //     .unwrap();

    // let k0 = builder.generate_single(&ts, opening);
    // let k0 = TrainingData::generate_single(&ts, vec![]);

    // eprintln!("k0.result = {:?}", k0.result);

    return;

    let mv0 = Move::Quiet { from: "E3".into(), to: "D3".into(), pc: Queen };
    let mv1 = Move::Quiet { from: "D6".into(), to: "E6".into(), pc: Queen };
    let mv2 = Move::Quiet { from: "D3".into(), to: "E3".into(), pc: Queen };
    let mv3 = Move::Quiet { from: "E6".into(), to: "D6".into(), pc: Queen };

    // for _ in 0..2 {
    //     g = g.make_move_unchecked(&ts, mv0).unwrap();
    //     g = g.make_move_unchecked(&ts, mv1).unwrap();
    //     g = g.make_move_unchecked(&ts, mv2).unwrap();
    //     g = g.make_move_unchecked(&ts, mv3).unwrap();
    // }

    // for (n,(zb,mv)) in g.history.iter().enumerate() {
    //     eprintln!("{}: {:?} = {:?}", n, zb, mv);
    // }

    // let g2 = g.clone().make_move_unchecked(&ts, mv0).unwrap();


    // let zb0 = g.zobrist;
    // let zb1 = g2.zobrist;

    // eprintln!("zb0 = {:?}", zb0);
    // eprintln!("zb1 = {:?}", zb1);

    // let n = g.history.len();
    // let mut k = 0;

    // for (pz,pmv) in g.history.iter() {
    //     if *pz == g.zobrist {
    //         panic!("{}, {:?}", k, pmv);
    //     }
    //     k += 1;
    // }

    // loop {
    //     let idx = 3 + k * 2;
    //     eprintln!("idx = {:?}", idx);
    //     if idx >= n { break; }
    //     // let (z,pmv) = g.history[n - idx];
    //     let (z,pmv) = g.history[idx];
    //     if z == g.zobrist {
    //         panic!("{}, {:?}", k, pmv);
    //     } else {
    //         eprintln!("nop {}, {:?}", k, pmv);
    //     }
    //     k += 1;
    // }

    return;

    // let mv = ob.best_move(&g, &mut s).unwrap();
    // let g = g.make_move_unchecked(&ts, mv).unwrap();

    // eprintln!("g = {:?}", g);

    // let mvs = ob.best_moves(&g).unwrap();
    // for mv in mvs {
    //     eprintln!("mv = {:?}", mv);
    // }
    // println!();

    // loop {
    //     if let Some(mv) = ob.best_move(&g, &mut s) {
    //         eprintln!("mv = {:?}", mv);
    //     } else { break; }
    // }

    // let mut gs = vec![];

    // println!("wat 0");
    // loop {
    //     if let Some(mv) = ob.best_move(&g, &mut s) {
    //         let g2 = g.make_move_unchecked(&ts, mv).unwrap();
    //         gs.push(g2);
    //     } else { break; }
    // }

    // loop {
    //     if let Some(g) = ob.start_game(&ts, &mut s) {
    //         gs.push(g);
    //     } else { break; }
    // }

    // eprintln!("gs.len() = {:?}", gs.len());

    // eprintln!("gs.len() = {:?}", gs.len());
    // for (n,g) in gs.into_iter().enumerate() {
    //     eprintln!("g {} = {:?}", n, g);
    // }

    // let g = ob.start_game(&ts, s);

    // let key = OpeningBook::gen_key(&g);
    // let mvs = ob.map.get(&key).unwrap();

    // let mvs = ob.best_moves(&g).unwrap();

    // for mv in mvs.into_iter() {
    //     eprintln!("mv = {:?}", mv);
    // }

    // let mv = mvs.into_iter().max_by_key(|(_,k)| *k).unwrap();
    // let g2 = g.make_move_unchecked(&ts, mv.0).unwrap();

    // eprintln!("g2 = {:?}", g2);

    // let _ = rchess_engine_lib::brain::trainer::generate_training_data(&ts, &ob);

    return;

    let mut nn = NNUE::new(White, &mut rng);
    // nn.init_inputs(&g);
    // nn.dirty = false;

    // let mut nn2 = nn.clone();
    // let mv = Move::Quiet { from: "E2".into(), to: "E3".into(), pc: Pawn };
    // let g2 = g.make_move_unchecked(&ts, mv).unwrap();
    // nn2.update_move(&g2);
    // let s1 = nn2.run_partial();
    // eprintln!("s0 = {:?}", s0);
    // eprintln!("s1 = {:?}", s1);

    // nn._init_inputs(&g, xs);

    // let g2 = g.flip_sides(&ts);
    // eprintln!("g = {:?}", g);
    // eprintln!("g2 = {:?}", g2);

    // trace!("wat 0");
    // let s0 = nn.run_fresh(&g);
    // eprintln!("s0 = {:?}", s0);
    // trace!("wat 1");
    // nn.side = Black;
    // let s1 = nn.run_fresh(&g2);
    // eprintln!("s1 = {:?}", s1);

    let eta = 100;

    let correct = 10;

    let s0 = nn.run_fresh(&g);
    eprintln!("s0 = {:?}", s0);

    // nn.backprop(&g, correct, eta);

    // let s1 = nn.run_fresh(&g);
    // eprintln!("s1 = {:?}", s1);

    // return;

    println!("starting...");
    let t0 = Instant::now();
    for k in 0..1000 {
        let pred = nn.backprop(Some(&g), correct, eta);

        if k % 50 == 0 {
            let delta = pred - correct;
            eprintln!("({:>4}): {:>8}, delta = {:?}", k, pred, delta);
        }
        // nn.run_partial();
    }
    println!("finished in {:.3} seconds", t0.elapsed().as_secs_f64());

    let s1 = nn.run_fresh(&g);
    eprintln!("s1 = {:?}", s1);

    // let mut vs0 = vec![];
    // println!("starting...");
    // let t0 = Instant::now();
    // for _ in 0..100 {
    //     let moves = g2.search_all(&ts).get_moves_unsafe();
    //     let mut moves = moves.into_iter().filter(|m| m.piece() != Some(King));
    //     let mv = moves.next().unwrap();
    //     g2 = g2.make_move_unchecked(&ts, mv).unwrap();
    //     let score = nn2.run_fresh(&g2);
    //     vs0.push(score);
    // }
    // println!("finished in {:.3} seconds", t0.elapsed().as_secs_f64());

}

#[allow(unreachable_code)]
fn main_nn() {
    use rchess_engine_lib::brain::networks2::*;

    let mut rng: StdRng = SeedableRng::seed_from_u64(1234u64);

    // const IS: usize = 769;
    // const OS: usize = 1;
    // const HL: usize = 1;

    // let hidden_sizes = vec![128];
    // let mut nn = NNUE3::<f32,IS,OS,HL>::new(hidden_sizes, &mut rng);
    // // eprintln!("nn.weights.len() = {:?}", nn.weights.len());
    // eprintln!("nn.activations[0].shape() = {:?}", nn.activations[0].shape());

    use rchess_engine_lib::sf_compat::*;
    // let mut nn = read_nnue(&path);

    _main_nn().unwrap();

}

#[allow(unreachable_code)]
fn _main_nn() -> std::io::Result<()> {

    use std::io;
    use io::Read;
    use std::path::Path;

    use byteorder::{ReadBytesExt, LittleEndian};

    use rchess_engine_lib::sf_compat::*;
    use rchess_engine_lib::sf_compat::accumulator::*;

    // return Ok(());

    // let path = "nn-cdf1785602d6.nnue";
    // let path = "nn-13406b1dcbe0.nnue";
    let path = "nn-63376713ba63.nnue";

    let path2 = "test_nn.nnue";
    // let path2 = "nn_sf.nnue";

    use std::io::{BufReader,Write,BufWriter};

    let mut nn = NNUE4::read_nnue(path).unwrap();

    if !true {
        let persp = White;
        let king_sq = Coord::from("E1");
        // let o_king_sq = NNUE4::orient(king_sq, persp, king_sq);
        // eprintln!("o_king_sq = {:?}", o_king_sq);
        {
            let side = White;
            let pc = King;
            let sq = Coord::from("E1");
            let idx = NNUE4::make_index_half_ka_v2(king_sq, persp, pc, side, sq);
            let x = 22468;
            eprintln!("idx 1 = {:?}, idx == {}, {}", idx, x, x == idx.0);
        }
        {
            let side = White;
            let pc = Pawn;
            let sq = Coord::from("E2");
            let idx = NNUE4::make_index_half_ka_v2(king_sq, persp, pc, side, sq);
            let x = 21836;
            eprintln!("idx 0 = {:?}, idx == {}, {}", idx, x, x == idx.0);
        }
        {
            let side = Black;
            let pc = King;
            let sq = Coord::from("E8");
            let idx = NNUE4::make_index_half_ka_v2(king_sq, persp, pc, side, sq);
            let x = 22524;
            eprintln!("idx 2 = {:?}, idx == {}, {}", idx, x, x == idx.0);
        }
    }

    if !true {
        let persp = Black;
        let king_sq = Coord::from("E8");
        // let o_king_sq = NNUE4::orient(king_sq, persp, king_sq);
        // eprintln!("o_king_sq = {:?}", o_king_sq);
        {
            let side = White;
            let pc = King;
            let sq = Coord::from("E1");
            let idx = NNUE4::make_index_half_ka_v2(king_sq, persp, pc, side, sq);
            let x = 22524;
            eprintln!("idx 1 = {:?}, idx == {}, {}", idx, x, x == idx.0);
        }
        {
            let side = White;
            let pc = Pawn;
            let sq = Coord::from("E2");
            let idx = NNUE4::make_index_half_ka_v2(king_sq, persp, pc, side, sq);
            let x = 21940;
            eprintln!("idx 0 = {:?}, idx == {}, {}", idx, x, x == idx.0);
        }
        {
            let side = Black;
            let pc = King;
            let sq = Coord::from("E8");
            let idx = NNUE4::make_index_half_ka_v2(king_sq, persp, pc, side, sq);
            let x = 22468;
            eprintln!("idx 2 = {:?}, idx == {}, {}", idx, x, x == idx.0);
        }
    }

    {
        let ts = Tables::read_from_file_def().unwrap();

        // // let fen = STARTPOS;
        // let fen1 = "4k3/3ppp2/8/8/8/8/3PPP2/4K3 w - - 0 1"; // base
        // let fen2 = "4k3/3ppp2/8/8/8/8/2NPPP2/4K3 w - - 0 1"; // +1 knight
        // let fen3 = "4k3/2nppp2/8/8/8/8/3PPP2/4K3 w - - 0 1"; // -1 knight
        // let mut g1 = Game::from_fen(&ts, fen1).unwrap();
        // let mut g2 = Game::from_fen(&ts, fen2).unwrap();
        // let mut g3 = Game::from_fen(&ts, fen3).unwrap();

        // // let fen = "4k3/3ppp2/8/8/8/8/2NPPP2/4K3 w - - 0 1"; // +1 knight
        // // let fen = "4k3/8/8/8/8/8/4P3/4K3 w - - 0 1";
        // let fen = "8/8/8/8/8/8/8/8 w - - 0 1";
        // let mut g = Game::from_fen(&ts, fen).unwrap();
        // let v = nn.evaluate(&g, false);
        // eprintln!("v = {:?}", v);

        // let fen = "4k3/3ppp2/8/8/8/8/2NPPP2/4K3 w - - 0 1"; // +1 knight
        // let fen = "4k3/8/8/8/8/8/4P3/4K3 w - - 0 1";
        // let fen = "8/8/8/8/8/8/8/8 w - - 0 1";
        let fen = "r4rk1/4npp1/1p1q2b1/1B2p3/1B1P2Q1/P3P3/5PP1/R3K2R b KQ - 1 1"; // Q cap d6b4
        // let fen = "r4rk1/4n1p1/1p1q1pb1/1B2p3/1B1PP1Q1/P7/5PP1/R3K2R b KQ - 0 2";
        // let fen = "rnbqk2r/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R w KQkq - 0 1";
        let mut g = Game::from_fen(&ts, fen).unwrap();

        // type LayerX = NNAffine<Layer0, 8, {1024 * 2}>;
        // // let layer1: &LayerX = &nn.layers[0].prev.prev.prev.prev;
        // let layer1 = &nn.layers[0].prev.prev;

        // eprintln!("layer1.weights.len() = {:?}", layer1.weights.len());

        // let max = layer1.weights.iter().max();
        // let min = layer1.weights.iter().min();
        // eprintln!("(min,max) = {:?}", (min,max));

        let mut ft = nn.ft.clone();

        use aligned::{Aligned,A64};
        let mut transformed: Aligned<A64,_> = Aligned([0; HALF_DIMS * 2]);

        let bucket = 0;

        // ft.reset_accum(&g);

        // return Ok(());

        let mut nn2 = nn.clone();

        nn2.ft.reset_accum(&g);

        let v0 = nn2.evaluate(&g, false);
        eprintln!("v0 = {:?}", v0);
        eprintln!("v0 == -599 = {:?}", v0 == -599);

        // return Ok(());

        // let mv1 = Move::new_capture("a8", "a3", Rook, Pawn);
        // let mv2 = Move::new_capture("e5", "d4", Pawn, Pawn);
        // let mv2 = Move::new_capture("e3", "d4", Pawn, Pawn);
        let mv2 = Move::new_quiet("e5", "e4", Pawn);
        // let mv2 = Move::new_quiet("d4", "d5", Pawn);

        // let mv2 = Move::new_quiet("e1", "f1", King);

        // let mv2 = Move::Castle {
        //     from:      "e1".into(),
        //     to:        "g1".into(),
        //     rook_from: "h1".into(),
        //     rook_to:   "f1".into(),
        // };

        let g2 = g.make_move_unchecked(&ts, mv2).unwrap();
        // nn2.ft.make_move(&g2, mv2);

        let mut nn3 = nn2.clone();

        let i_w = NNIndex(21924);
        let i_b = NNIndex(20444);
        // let i_w = NNIndex(21916);
        // let i_b = NNIndex(20452);

        nn2.ft._accum_inc_simd::<true>(White, i_w);
        // nn2.ft._accum_inc_simd::<true>(Black, i_b);

        nn3.ft._accum_add(White, i_w);
        // nn3.ft._accum_add(Black, i_b);

        eprintln!("nn2.accum.accum == nn3.accum.accum = {:?}", nn2.ft.accum.accum == nn3.ft.accum.accum);
        eprintln!("nn2.accum.psqt == nn3.accum.psqt = {:?}", nn2.ft.accum.psqt == nn3.ft.accum.psqt);

        return Ok(());

        let mut transformed: Aligned<A64,_> = Aligned([0; HALF_DIMS * 2]);
        let psqt = nn2.ft.transform(&g2, transformed.as_mut(), 0);
        eprintln!("psqt = {:?}", psqt);
        eprintln!("psqt == 2 = {:?}", psqt == 2);

        let v1 = nn2.evaluate(&g2, false);
        eprintln!("v1 = {:?}", v1);
        eprintln!("v1 == 755 = {:?}", v1 == 755);

        let fen = "r4rk1/4npp1/1p1q2b1/1B6/1B1Pp1Q1/P3P3/5PP1/R3K2R w KQ - 0 2";
        let mut g = Game::from_fen(&ts, fen).unwrap();

        nn2.ft.reset_accum(&g);
        let v0 = nn2.evaluate(&g, false);
        eprintln!("v0 = {:?}", v0);

        // // nn2.ft._update_accum(&g2, White);
        // // nn2.ft._update_accum(&g2, Black);
        // nn2.ft.make_move(&g2, mv1);
        // // nn2.ft.reset_accum(&g2);

        // nn2.ft.accum_pop();

        return Ok(());

        let g3 = g.make_move_unchecked(&ts, mv2).unwrap();
        nn2.ft.make_move(&g3, mv2);

        if !true {
            let g = &g3;
            let side = !g.state.side_to_move; // XXX: should be after make_move g -> g2
            eprintln!("side = {:?}", side);
            let ksqs = [g.get(King,White).bitscan(),g.get(King,Black).bitscan()];
            eprintln!("ksqs[White] = {:?}", ksqs[White]);
            eprintln!("ksqs[Black] = {:?}", ksqs[Black]);

            // let [i_w,i_b] = NNUE4::make_index_2(ksqs, pc, side, sq);

            let deltas = nn2.ft.make_move_move(ksqs, Pawn, Black, "e5".into(), "e4".into());
            let (a0,a1) = deltas[0].get();
            let (a2,a3) = deltas[1].get();

            eprintln!("a0.get_index() = {:?}", a0.get_index(White));
            eprintln!("a1.get_index() = {:?}", a1.get_index(White));
            eprintln!("a2.get_index() = {:?}", a2.get_index(White));
            eprintln!("a3.get_index() = {:?}", a3.get_index(White));

            // eprintln!("delta = {:?}", delta);
        }

        // let ks = &nn2.ft.accum.stack_delta;
        // for k in ks.iter() {
        //     // eprintln!("k = {:?}", k);
        //     if let NNDeltas::CopyCastle(persp, (from,to), (rook_from,rook_to)) = k {
        //         eprintln!("from.get_index() = {:?}", from.get_index(Black));
        //         eprintln!("to.get_index() = {:?}", to.get_index(Black));
        //         eprintln!("rook_from.get_index() = {:?}", rook_from.get_index(Black));
        //         eprintln!("rook_to.get_index() = {:?}", rook_to.get_index(Black));
        //     }
        // }

        nn2.ft.accum_pop();
        let v3 = nn2.evaluate(&g, false);
        eprintln!("v3 = {:?}", v3);

        // let g3 = g2.make_move_unchecked(&ts, mv2).unwrap();
        // nn2.ft.make_move(&g3, mv2);

        // nn2.ft.accum.needs_refresh = [false; 2];
        // let v2 = nn2.evaluate(&g2, false, false);
        // eprintln!("v2 = {:?}", v2);

        // let fen0 = "r4rk1/4npp1/1p1q2b1/1B6/1B1P2Q1/P3P3/5PP1/R3K2R b KQ - 1 1";
        // let g0 = Game::from_fen(&ts, fen0).unwrap();
        // nn2.ft.reset_accum(&g0);
        // let v0 = nn2.evaluate(&g0, false, false);
        // eprintln!("v0 = {:?}", v0);

        // for x in nn2.ft.accum.stack_delta.iter() {
        //     eprintln!("x = {:?}", x);
        // }

        // nn2.ft.reset_accum(&g);

        // nn2.ft.accum.stack_delta.clear();
        // use rchess_engine_lib::sf_compat::accumulator::NNDelta;
        // let mut xs = arrayvec::ArrayVec::<NNDelta,8>::new();
        // let idx0 = NNUE4::make_index_half_ka_v2("E1".into(), White, Pawn, Black, "E5".into());
        // // let idx1 = NNUE4::make_index_half_ka_v2("E1".into(), White, Pawn, White, "D4".into());
        // // let idx2 = NNUE4::make_index_half_ka_v2("E1".into(), White, Pawn, Black, "D4".into());
        // xs.push(NNDelta::Remove(idx0,White));
        // // xs.push(NNDelta::Remove(idx1));
        // // xs.push(NNDelta::Add(idx2));
        // let idx0 = NNUE4::make_index_half_ka_v2("G8".into(), Black, Pawn, Black, "E5".into());
        // // let idx1 = NNUE4::make_index_half_ka_v2("G8".into(), Black, Pawn, White, "D4".into());
        // // let idx2 = NNUE4::make_index_half_ka_v2("G8".into(), Black, Pawn, Black, "D4".into());
        // xs.push(NNDelta::Remove(idx0,Black));
        // // xs.push(NNDelta::Remove(idx1));
        // // xs.push(NNDelta::Add(idx2));
        // nn2.ft.accum.stack_delta.push(xs);

        // let idx0 = NNUE4::make_index_half_ka_v2("E1".into(), White, Pawn, Black, "E5".into());
        // nn2.ft.accum_rem(White, idx0, false);
        // let idx0 = NNUE4::make_index_half_ka_v2("G8".into(), Black, Pawn, Black, "E5".into());
        // nn2.ft.accum_rem(Black, idx0, false);

        // // nn2.ft.accum.needs_refresh = [false; 2];
        // // nn2.ft.accum_pop();
        // // nn2.ft.accum_pop();
        // // nn2.ft.accum.needs_refresh = [false; 2];
        // let v3 = nn2.evaluate(&g3, false, false);
        // eprintln!("v3 = {:?}", v3);
        // // eprintln!("v3 == -599 = {:?}", v3 == -599);

        // let g3 = g2.make_move_unchecked(&ts, mv2).unwrap();
        // nn2.ft.make_move(&g3, mv2);
        // nn2.ft.accum.needs_refresh = [false; 2];
        // let v3 = nn2.evaluate(&g3, false, false);
        // eprintln!("v3 = {:?}", v3);

        // nn2.make_mov

        // let fen0 = "r4rk1/4npp1/1p1q2b1/1B6/1B1p2Q1/P3P3/5PP1/R3K2R w KQ - 0 2";
        // let mut g0 = Game::from_fen(&ts, fen0).unwrap();

        // nn.ft.reset_accum(&g0);
        // let v = nn.evaluate(&g0, false);
        // eprintln!("v = {:?}", v);

        return Ok(());

        // d6b4 -> -599
        let v = nn.evaluate(&g, false);
        eprintln!("v = {:?}", v);
        eprintln!("v == -599 = {:?}", v == -599);

        // type LayerX = NNAffine<Layer0, 8>;
        // let ws: &LayerX = &nn.layers[0].prev.prev.prev.prev;

        return Ok(());

        // baseline =  50_000 = 0.42
        // baseline = 100_000 = 0.962

        // println!("starting");
        // let t0 = std::time::Instant::now();
        // for n in 0..100000 {
        //     // for bucket in 0..8 {
        //     //     nn.layers[bucket].propagate(&tfs[bucket]);
        //     // }
        //     // let _ = nn.trace_eval(&g, false);
        //     nn.ft.accum.needs_refresh = [true; 2];
        //     let _ = nn.evaluate(&g, false, true);
        // }
        // let t1 = t0.elapsed().as_secs_f64();
        // eprintln!("finished in {:.3} seconds", t1);

        // return Ok(());

        // eprintln!("g.to_fen() = {:?}", g.to_fen());
        // eprintln!("g = {:?}", g);
        // eprintln!("evaling");

        // let eval1 = nn.evaluate(&g1, false);
        // eprintln!("eval 1 = {:?}", eval1);

        // let eval2 = nn.evaluate(&g1, false);
        // eprintln!("eval 2 = {:?}", eval2);

        // let eval3 = nn.evaluate(&g1, false);
        // eprintln!("eval 3 = {:?}", eval3);


        const HALF_DIMS: usize = 1024;
        let mut transformed = [0; HALF_DIMS * 2];

        // let mut exhelper = exhelper_once(
        //     &g, g.state.side_to_move, &ev_mid, &ev_end, Some(&ph_rw), None);
        // let eval = exhelper.cfg.evaluate(&ts, &g, None);

        // let (ev_mid,ev_end) = EvalParams::new_mid_end();
        // let eval = g.sum_evaluate(&ts, &ev_mid, &ev_end, None);

        // eprintln!("\neval = {:?}", eval);

        // let v = nn.evaluate(&g, false);
        // eprintln!("\nv = {:?}", v);
        // return Ok(());

        let (psqt,positional,bucket) = nn.trace_eval(&g, false);

        // let mut vs = HashMap::<(Piece,Coord),Score>::default();
        // let base = nn.evaluate2(&g, false);
        // println!("NNUE Piece Values");
        // for (pc,c0) in g.iter_side_pieces(White) {
        //     if pc == King { continue; }
        //     let g2 = g.delete_piece_mut_unchecked(&ts, c0, pc, White, true);
        //     let eval = nn.evaluate2(&g, false);
        //     let v = base - eval;
        //     eprintln!("{:?} at {:?} = {:?}", pc, c0, v);
        // }

        for i in 0..8 {
            let x0 = psqt[i];
            let x1 = positional[i];

            // PawnValueEg   = 208,
            let x0 = (x0 as f64).abs() / 208.0;
            let x1 = (x1 as f64).abs() / 208.0;

            if i == bucket {
                // eprintln!("{} = {:>3}, {:>3}  <- this bucket used", i, x0, x1);
                eprintln!("{} = {:.2}, {:.2}  <- this bucket used", i, x0, x1);
            } else {
                eprintln!("{} = {:.2}, {:.2}", i, x0, x1);
                // eprintln!("{} = {:>3}, {:>3}", i, x0, x1);
            }
        }

        return Ok(());
    }

    Ok(())
}

#[allow(unreachable_code)]
fn main_nn2() {
    // use ndarray::prelude::*;

    use nalgebra::{SMatrix,SVector,Matrix,Vector,matrix,vector,dmatrix,dvector,DVector,DMatrix};

    use rand::thread_rng;
    use rand::prelude::{StdRng,SliceRandom};
    use rand::{Rng,SeedableRng};

    use rchess_engine_lib::brain::*;
    use rchess_engine_lib::brain::nnue::*;
    use rchess_engine_lib::brain::types::*;

    // let t0 = Instant::now();
    // for _ in 0..n {
    //     wat_ndarray::<K>();
    // }
    // println!("ndarray:  finished in {:.3} seconds", t0.elapsed().as_secs_f64());

    // main_mnist();
    // // main_nnue();
    // return;

    // let h = std::thread::Builder::new()
    //     .stack_size(24 * 1024 * 1024)
    //     .spawn(move || {
    //     main_mnist();
    // }).unwrap();
    // h.join().unwrap();

    // // main_mnist2();
    // return;

    let inputs = vec![
        dvector![0.0,0.0],
        dvector![1.0,0.0],
        dvector![0.0,1.0],
        dvector![1.0,1.0],
    ];
    let corrects = vec![
        dvector![0.0],
        dvector![1.0],
        dvector![1.0],
        dvector![0.0],
    ];
    // let corrects = inputs.clone();

    // let inputs = vec![
    //     dvector![0.0,0.0,0.0],
    //     dvector![1.0,0.0,0.0],
    //     dvector![0.0,1.0,0.0],
    //     dvector![1.0,1.0,0.0],
    //     dvector![0.0,0.0,1.0],
    //     dvector![1.0,0.0,1.0],
    //     dvector![0.0,1.0,1.0],
    //     dvector![1.0,1.0,1.0],
    // ];
    // let corrects = vec![
    //     dvector![0.0, 0.0],
    //     dvector![1.0, 1.0],
    //     dvector![1.0, 1.0],
    //     dvector![0.0, 0.0],
    //     dvector![0.0, 1.0],
    //     dvector![1.0, 0.0],
    //     dvector![1.0, 0.0],
    //     dvector![0.0, 1.0],
    // ];
    // // let corrects = inputs.clone();

    let inputs0 = vec![
        vector![0.0,0.0],
        vector![1.0,0.0],
        vector![0.0,1.0],
        vector![1.0,1.0],
    ];
    let corrects0 = vec![
        vector![0.0],
        vector![1.0],
        vector![1.0],
        vector![0.0],
    ];
    // let corrects = inputs.clone();

    // let xs = inputs.iter().zip(corrects.iter()).collect::<Vec<_>>();
    let xs = inputs.into_iter().zip(corrects.into_iter());
    // let xs = inputs.into_iter().zip(corrects.into_iter());

    let xs0 = inputs0.into_iter().zip(corrects0.clone().into_iter());

    // na:   (Rows, Cols)
    // Multiply:
    // LHS:     M x N
    // RHS:     N x K
    // Result:  M x K

    let lr = 0.1;
    // let mut nn = Network::new(2);
    use rchess_engine_lib::brain::types::g_networks::*;
    let mut nn0 = Network::new_range(1, (0.,1.));

    // let mut nn = DNetwork::<f32,2,1>::new_range(vec![2,3,1], (0.,1.));

    // for (input,correct) in xs.clone() {
    //     let (pred,pred_z,acts) = nn._run(input);
    //     let delta = pred - correct; // OS,1
    //     let delta = delta.component_mul(&pred_z.map(sigmoid_deriv));
    //     let error = 2.0 * (pred - correct);
    //     let error = error.component_mul(&pred.map(sigmoid_deriv));
    //     eprintln!("{:?} = {:?}, {:?}", correct, error, delta);
    //     // predictions.push((pred,pred_z));
    //     // activations.push(acts);
    // }

    // nn.backprop_mut(inputs.clone(), corrects.clone(), 0.1);

    // let (ins,cors) = Network::fill_input_matrix::<4>(xs.clone().collect());

    // nn.backprop_mut_matrix::<4>(ins, cors, lr);
    // return;

    // println!("===");
    // eprintln!("{} = {}", 0, nn0.weights_in);
    // for (k,ws) in nn0.weights.iter().enumerate() {
    //     let k = k + 1;
    //     eprintln!("{} = {}", k, ws);
    // }
    // eprintln!("{} = {}", 0, nn0.weights_out);
    // return;

    // return;

    let mut rng: StdRng = SeedableRng::seed_from_u64(1234u64);

    // let mut nn = DNetwork::<f32,2,1>::new_range(vec![2,3,3,1], (0.,1.));
    // let mut nn = DNetwork::<f32,2,1>::new_range(vec![2,3,3,1], (-1.,1.));
    // let mut nn = DNetwork::<f32,2,1>::new_range(vec![2,3,1], (0.,1.));

    // let mut nn = DNetwork::<f32,3,2>::new_range(vec![3,3,3,2], (0.,1.));

    // let mut nn = DNetwork::<f32,3,2>::new_range(vec![3,512,32,32,2], (0.0,1.0));
    let mut nn = DNetwork::<f32,3,1>::new_range(vec![3,512,32,32,1], (0.0,1.0));

    let ins = vec![
        (dvector![0.0, 0.0, 1.0],dvector![0.0]),
        // (dvector![1.0, 0.0, 1.0],dvector![1.0]),
        // (dvector![0.0, 1.0, 1.0],dvector![1.0]),
        // (dvector![1.0, 1.0, 1.0],dvector![0.0]),
    ];

    // eprintln!("nn.weights.len() = {:?}", nn.weights.len());

    // let ws0 = nn.weights[0].clone();
    // eprintln!("ws0.shape() = {:?}", ws0.shape());
    // for _ in 0..100 {
    //     nn.backprop_mut_matrix(&ins, 0.1);
    // }
    // let ws1 = nn.weights[0].clone();
    // eprintln!("ws0 == ws1 = {:?}", ws0 == ws1);

    nn.backprop_mut_matrix(&ins, 0.1);
    return;

    let mut xs2: Vec<_> = xs.clone().collect();
    // let mut xs2: Vec<_> = xs0.clone().collect();

    println!("Starting...");
    let t0 = Instant::now();
    for k in 0..10000 {
        // eprintln!("k = {:?}", k);
        xs2.shuffle(&mut rng);

        let mut xs3 = xs2.clone();
        // xs3.truncate(2);
        xs3.truncate(2);

        // nn0.backprop_mut_matrix::<4>(&xs3, lr);
        nn.backprop_mut_matrix(&xs3, 0.1);

        // for (i,c) in inputs0.iter().zip(corrects0.iter()) {
        //     nn0.backprop_mut(vec![i], vec![*c], 0.1);
        // }

        // for (input, c) in xs.clone().into_iter().take(1) {
        //     nn.backprop_mut(vec![input], vec![c.clone()], 0.1);
        // }

        // break;

        // let (inputs,corrects): (Vec<&SVector<f32,2>>,Vec<SVector<f32,1>>) = xs3.into_iter().unzip();
        // nn0.backprop_mut(inputs.clone(), corrects.clone(), 0.1);

        // let (inputs,corrects): (Vec<&DVector<f32>>,Vec<DVector<f32>>) = xs3.into_iter().unzip();
        // nn0.backprop_mut(inputs.clone(), corrects.clone(), 0.1);

    }
    println!("finished in {:.3} seconds.", t0.elapsed().as_secs_f64());

    println!();
    println!("X     Y     Cor    Ans   err");
    for (input, c) in xs.clone() {
    // for (input, c) in xs0.clone() {
        let pred = nn.run(&input);
        // let pred = nn0.run(&input);

        // let (pred,pred_a,acts) = nn._run(&input);
        // eprintln!("pred_a = {:?}", pred_a);
        println!("{:.2}  {:.2}  {}      {:.3}", input[0], input[1], c[0], pred[0]);
        // eprintln!("acts = {:?}", acts);
    }

    let a0 = matrix![
        1, 5;
        3, 7;
    ];
    let a1 = matrix![
        2, 6;
        4, 8;
    ];

    // // let k = a0 * a1;
    // let k0 = a1.transpose() * a0;
    // let k1 = a1 * a0.transpose();
    // // let k = a0.component_mul(&a1);

    // eprintln!("k0 = {:?}", k0);
    // eprintln!("k1 = {:?}", k1);

    // let mut nn = Network2::<f32>::new(
    //     // 3,4,1,1
    //     2,3,1,1
    // );

    // let i = ArrayView1::from(&inputs[0]).to_owned();
    // nn._run(i);
    // return;

    // println!();
    // println!("X     Y     Cor    Ans   err");
    // for (input, c) in xs.clone() {
    //     let i = ArrayView1::from(input);
    //     // let ans = nn.run(i);
    //     let (pred, acts) = nn._run(i);
    //     println!("{:.2}  {:.2}  {}      {:.2}", input[0], input[1], *c as u32, pred[0]);
    //     eprintln!("acts = {:?}", acts);
    // }


    // let inputs = vec![
    //     array![0.0, 0.0],
    //     array![1.0, 0.0],
    //     array![0.0, 1.0],
    //     array![1.0, 1.0],
    // ];
    // let corrects = vec![
    //     array![0.0],
    //     array![1.0],
    //     array![1.0],
    //     array![1.0],
    // ];
    // let errs = nn.backprop(inputs, corrects);


    // for _ in 0..10000 {
    //     for (input, c) in xs.clone() {
    //         let i = ArrayView1::from(input).to_owned();
    //         let (ans,acts) = nn._run(i);
    //     }
    // }

    // let a0 = array![
    //     [1,1,1],
    //     [1,1,1],
    //     [1,1,1],
    //     [1,1,1],
    // ];
    // let a1: Array1<u16> = array![
    //     2,2,2
    // ];
    // let a2 = array![
    //     1,1,1,1
    // ];
    // eprintln!("a0.shape() = {:?}", a0.shape());
    // eprintln!("a1.shape() = {:?}", a1.shape());
    // let k = a0.dot(&a1);
    // let k2 = &k + a2;
    // eprintln!("k = {:?}", k);
    // eprintln!("k2 = {:?}", k2);


    // for _ in 0..10000 {
    //     for (input, c) in xs.clone() {
    //         let i = ArrayView1::from(input).to_owned();
    //         let (hidden_out,ans) = nn.run(i.clone());
    //         let err = nn.backprop(*c, ans, i, hidden_out);
    //     }
    // }

    // println!();
    // println!("X     Y     Cor    Ans  err");
    // for (input, c) in xs.clone() {
    //     let i = ArrayView1::from(input).to_owned();
    //     let (hidden_out,ans) = nn.run(i.clone());
    //     let err = nn.backprop(*c, ans, i, hidden_out);
    //     println!("{:.2}  {:.2}  {}      {:.2},  {:.2}", input[0], input[1], *c as u32, ans, err);
    // }

    // let x0 = array![0.0, 1.0];
    // eprintln!("x0 = {:?}", x0);

    // let input: [f32; 2] = [0.0, 0.0];

    // // let i = ArrayView1::from(&input).to_owned();
    // let i = ArrayView1::from(&input).to_owned().insert_axis(Axis(0)).reversed_axes();

    // eprintln!("i = {:?}", i);

    // let x = array![[1.0], [1.0]];
    // let y = array![[2.0, 3.0]];
    // let k = x.dot(&y);
    // eprintln!("k = {:?}", k);

    // eprintln!("nn = {:?}", nn);
    // println!();
    // let k = nn.run(array![0.0,0.0]);
    // eprintln!("\nk = {:?}", k);

}

#[allow(unreachable_code)]
fn main_eval() {
    let ts = Tables::read_from_file_def().unwrap();
    let fen = STARTPOS;

    // let fen1 = "rnbqkbnr/ppp3pp/4p3/3pNp2/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1"; // outpost knight
    // let fen1 = "rnbqkbnr/ppp3pp/4p3/3pBp2/3P4/8/PPP1PPPP/RNBQK1NR w KQkq - 0 1"; // outpost bishop
    // let fen1 = "rnbqkbnr/ppp3pp/4p3/3p1p2/3P4/5N2/PPP1PPPP/RNBQKB1R w KQkq - 0 1"; // reachable knight
    // let fen1 = "rnbqkbnr/ppp3pp/4p3/3p1p2/3P4/3N1N2/PPP1PPPP/R1BQKB1R w KQkq - 0 1"; // reachable knight x2
    // let fen1 = "rnbqkbnr/ppp5/4p1p1/3p1p1p/3P3P/5N2/PPP1PPP1/RNBQKB1R w KQkq - 0 1"; // 2x reachable knight

    // let fen = "4k3/8/8/8/1P6/2PPPPP1/P7/4K3 w - - 0 1"; // pawns supported = 1
    // let fen = "4k3/8/8/8/1P6/P1PPPPP1/8/4K3 w - - 0 1"; // pawns supported = 2
    // let fen = "4k3/8/8/8/1P1P4/P1P1PPP1/8/4K3 b - - 0 1"; // pawns supported = 4
    // let fen = "4k3/8/8/8/1P1P1PP1/P1P1P3/8/4K3 b - - 0 1"; // pawns phalanx 2
    // let fen = "4k3/8/8/8/1P1P1PPP/P1P5/8/4K3 b - - 0 1"; // pawns phalanx 3

    // let fen = "4k3/8/8/8/1P1PP2P/PP3P2/8/4K3 b - - 0 1"; // isolated

    // let fen = "4k3/pp1ppp2/8/3p4/PP1PP2P/2P2P2/8/4K3 b - - 0 2"; // backward
    // let fen = "4k3/pp1ppp2/8/3p4/P2PP2P/1PP2P2/8/4K3 b - - 0 2"; // not backward

    // let fen = "4k3/pppppp2/8/1P1P4/P2P3P/2P2P2/8/4K3 b - - 0 2"; // doubled
    // let fen = "4k3/pppppp2/8/1P1P4/P1PP3P/5P2/8/4K3 b - - 0 2"; // doubled but supported

    // let fen = "4k3/2p1ppp1/1pP5/pP6/P6P/4PPP1/8/4K3 w - - 0 3"; // blocked 1x 5 + 1x 6


    // connected:   b5, c6, d4, e4
    // supported:   b5, c6, d4
    // phalanx:     d4, e4
    // passed:      
    // candidate:   
    // blocked 5:   b5
    // blocked 6:   c6
    // doubled:     e4
    // isolated:    h4
    // backward:    a4
    let fen = "4k3/2p1ppp1/1pP5/pP6/P2PP2P/4P3/8/4K3 w - - 0 3"; // catch all pawns

    let fen = "4k3/8/8/2PP4/8/8/8/4K3 w - - 0 3";

    eprintln!("fen = {:?}", fen);
    let g1 = Game::from_fen(&ts, fen).unwrap();
    let g2 = g1.clone().flip_sides(&ts);

    let ev_mid = EvalParams::default();
    let ev_end = EvalParams::default();
    let side = White;

    // eprintln!("g = {:?}", g);
    // eprintln!("g2 = {:?}", g2);

    // eprintln!("g1.to_fen() = {:?}", g1.to_fen());
    // eprintln!("g2.to_fen() = {:?}", g2.to_fen());

    // let score = g.sum_evaluate(&ts, &ev_mid, &ev_end, None);
    // eprintln!("score = {:?}", score);

    // let k0 = g1.score_pawns(&ts, &ev_mid, None, g1.state.side_to_move)
    //     - g1.score_pawns(&ts, &ev_mid, None, !g1.state.side_to_move);
    // let k1 = g2.score_pawns(&ts, &ev_mid, None, g2.state.side_to_move)
    //     - g2.score_pawns(&ts, &ev_mid, None, !g2.state.side_to_move);

    // let k0 = g1.score_pawns(&ts, &ev_mid, None, g1.state.side_to_move);

    // eprintln!("k0 = {:?}", k0);


    // let k = g.pawns_supported(side);
    // let k = g.pawns_phalanx(side);

    let g = g1;

    // let ph = g.gen_ph_entry(&ts);

    // g.get(Pawn, White).into_iter().for_each(|sq| {
    //     let c0 = Coord::from(sq);
    //     // if g._pawn_backward(&ts, c0, g.state.side_to_move) {
    //     //     eprintln!("backward = {:?}", c0);
    //     // }
    //     // if g._pawn_isolated(&ts, c0, g.state.side_to_move) {
    //     //     eprintln!("isolated = {:?}", c0);
    //     // }
    //     // if g._pawn_doubled(&ts, c0, g.state.side_to_move) {
    //     //     eprintln!("doubled = {:?}", c0);
    //     // }
    //     // if g._pawn_opposed(c0, g.state.side_to_move) {
    //     //     eprintln!("opposed = {:?}", c0);
    //     // }
    //     let c0 = Coord::from(sq);
    //     let score = g.connected_bonus(&ev_mid, &ph, c0, g.state.side_to_move);
    //     eprintln!("score {:?} = {:?}", c0, score);
    //     // eprintln!("k = {:?}", k);
    // });

    // let c0 = Coord::from("D5");
    // // let c0 = Coord::from("C2");
    // let k = g._pawn_connected_bonus(&ev_mid, &ph, c0, g.state.side_to_move);

    // eprintln!("k = {:?}", k);

    return;

}

#[allow(unreachable_code)]
fn main9() {
    let fen = STARTPOS;
    init_logger();

    // let ts = Tables::new();
    // ts.write_to_file_def().unwrap();
    let ts = Tables::read_from_file_def().unwrap();
    // let ts = &_TABLES;

    fn games_wac(i: usize) -> String {
        let mut games = read_epd("testpositions/WAC.epd").unwrap();
        let mut games = games.into_iter();
        let games = games.map(|x| x.0).collect::<Vec<_>>();
        games[i - 1].clone()
    }

    fn games_iq(i: usize) -> (String,Vec<String>) {
        let mut games = read_epd("testpositions/iq6.epd").unwrap();
        // let mut games = games.into_iter();
        // let games = games.map(|x| x.0).collect::<Vec<_>>();
        games[i - 1].clone()
    }

    fn games_sts(i: usize, sts: u8) -> String {
        let mut games = read_epd(&format!("testpositions/STS/STS{}.epd", sts)).unwrap();
        let mut games = games.into_iter();
        let games = games.map(|x| x.0).collect::<Vec<_>>();
        games[i - 1].clone()
    }

    // fn go(ts: &Tables, n: Depth, g: Game, t: f64)
    //       -> ((ABResult, Vec<ABResult>),SearchStats,(TTRead,TTWrite)) {
    //     let timesettings = TimeSettings::new_f64(0.0,t);
    //     let mut ex = Explorer::new(g.state.side_to_move, g.clone(), n, timesettings);
    //     ex.load_syzygy("/home/me/code/rust/rchess/tables/syzygy/").unwrap();
    //     ex.lazy_smp_negamax(&ts, false, false)
    // }

    let fen = "5rk1/ppR1Q1p1/1q6/8/8/1P6/P2r1PPP/5RK1 b - - 0 1"; // b6f2, #-4
    // let fen = "6k1/6pp/3q4/5p2/QP1pB3/4P1P1/4KPP1/2r5 w - - 0 2"; // a4e8, #3
    // let fen = "r1bq2rk/pp3pbp/2p1p1pQ/7P/3P4/2PB1N2/PP3PPR/2KR4 w Kq - 0 1"; // WAC.004, #2, Q cap h6h7
    let fen = "r4rk1/4npp1/1p1q2b1/1B2p3/1B1P2Q1/P3P3/5PP1/R3K2R b KQ - 1 1"; // Q cap d6b4

    // let fen = "5rk1/pp3pp1/8/4q1N1/6b1/4r3/PP3QP1/5K1R w - - 0 2"; // R h1h8, #4, knight move order?
    // let fen = "r4r1k/2Q5/1p5p/2p2n2/2Pp2R1/PN1Pq3/6PP/R3N2K b - - 0 1"; // #4, Qt N f5g3, slow

    // let fen = "1n4k1/2p2rpp/1n6/1q6/8/4QP2/1P3P1P/1N1R2K1 w - - 0 1"; // #3, Qt R d1d8

    // let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - "; // Position 2

    // let fen = "4r1k1/4nppp/2b5/1r2N3/5P2/3Q4/6PP/5RK1 w - - 0 1"; // QS test,
    // let fen = "4r1k1/4nppp/2N5/1r6/5P2/3Q4/6PP/5RK1 b - - 0 1"; // After N e5c6
    // // let fen = "4r1k1/5ppp/2n5/1r6/5P2/3Q4/6PP/5RK1 w - - 0 2"; // After N e7c6, recapture, WRONG
    // // let fen = "4r1k1/1r2nppp/2N5/8/5P2/3Q4/6PP/5RK1 w - - 1 2"; // After R b5b7, Correct

    // let fen = "5rk1/4npp1/1p4b1/1B2p3/1P1P2Q1/4P3/5PP1/r3K2R w K - 0 3"; // base
    // // let fen = "5rk1/4npp1/1p4b1/1B2p3/1P1P2Q1/4P3/4KPP1/r6R b - - 1 3"; // evade with K e1e2
    // // let fen = "5rk1/4npp1/1p4b1/1B2p3/1P1P4/4P3/5PP1/r2QK2R b K - 1 3"; // block with Q g4d1
    // // let fen = "5rk1/4npp1/1p4b1/1B2p3/1P1P2Q1/4P3/4KPP1/7r w - - 0 4"; // after evade, -320
    // // let fen = "5rk1/4npp1/1p4b1/1B2p3/1P1P4/4P3/5PP1/3K3R b - - 0 4"; // after block, -220

    // // let fen = "7k/8/8/8/8/8/4Q3/7K w - - 0 1"; // Queen endgame, #7
    // // let fen = "7k/4Q3/8/8/8/8/8/7K w - - 4 3"; // Queen endgame, #6
    // let fen = "7k/4Q3/8/8/8/8/6K1/8 w - - 4 3"; // Queen endgame, #5
    // // let fen = "6k1/4Q3/8/8/8/5K2/8/8 w - - 6 4"; // Queen endgame, #4
    // // let fen = "7k/4Q3/8/4K3/8/8/8/8 w - - 8 5"; // Queen endgame, #2
    // // let fen = "7k/8/8/8/8/8/4R3/7K w - - 0 1"; // Rook endgame,

    // let fen = &games(8); // Qt R e7f7, #7

    // let fen = &games(2); // STS2 002, Qt R a7E7
    // let fen = &games(2); // STS15 001, Qt Q d3d1

    let fen = "r3rbk1/1pq2ppp/p1n3b1/3BpNP1/4P3/P1Q1B2P/1PP2P2/3RR1K1 b - - 0 1"; // repetition
    // let fen = "r3rbk1/1pq2ppp/p1n5/3BpNPb/4P3/P1Q1B2P/1PP2P2/3RR1K1 w - - 1 2"; // repetition
    // let fen = "r3rbk1/1pq2ppp/p1n5/3BpNPb/4P3/P1QRB2P/1PP2P2/4R1K1 b - - 2 2"; // repetition
    // let fen = "r3rbk1/1pq2ppp/p1n3b1/3BpNP1/4P3/P1QRB2P/1PP2P2/4R1K1 w - - 3 3"; // repetition

    // let fen = "r1b2rk1/1pq1bppp/p2ppn2/2n3B1/3NP3/2N2Q2/PPP1BPPP/R4RK1 w - - 8 12"; // ??
    // let fen = "8/1p1b1pq1/3Npk2/2Q1p3/P4rp1/1PP5/K6p/4R3 w - - 2 45"; // Q cap c5e5

    // let fen = "7k/6pp/8/8/8/8/8/RK6 w - - 0 1"; // #1, Qt R a1a8

    // let fen = "r1bqk2r/ppp2ppp/2np1n2/4p3/1PP1P3/P1NPbN2/5PPP/R2QKB1R w KQkq -";

    // let fen = "8/k7/3p4/p2P1p2/P2P1P2/8/8/K7 w - - "; // Lasker-Reichhelm Position, Qt K a1b1

    // let fen = "rnbqkb1r/p4p2/2p1pn1p/1p2P1p1/2pP3B/2N2N2/PP3PPP/R2QKB1R w KQkq g6"; // rand opening

    // let fen = "rnb2r2/p1q2pQk/2p1pB1p/1p2P3/1bpPN3/2N5/PP3PPP/R3KB1R b KQ -"; // Mate

    // let fen = "4k3/r2bbprp/3p1p1N/2qBpP2/ppP1P1P1/1P1R3P/P7/1KR1Q3 w - - "; // STS 1 56

    // let fen = "8/5R2/1r3npk/1p2p1Np/p3P2P/P2K2P1/1P6/8 w - - 1 35"; // Qt K d3c3

    // let fen = "8/8/p1p5/1p5p/1P5p/8/PPP2K1p/4R1rk w - - 0 1";    // Qt R e1f1
    // let fen = "1q1k4/2Rr4/8/2Q3K1/8/8/8/8 w - - 0 1";            // Kh6
    // let fen = "7k/5K2/5P1p/3p4/6P1/3p4/8/8 w - - 0 1";           // g5
    // let fen = "8/6B1/p5p1/Pp4kp/1P5r/5P1Q/4q1PK/8 w - - 0 32";   // Qxh4
    // let fen = "8/8/1p1r1k2/p1pPN1p1/P3KnP1/1P6/8/3R4 b - - 0 1"; // Nxd5

    // let fen = "8/8/8/8/8/7p/3k1pr1/7K w - -"; // non legal ??

    // let fen = "2r3r1/pp1b4/1bn2pk1/3pP2p/1P5P/5NP1/P1NB1P2/2RKR3 b - - 0 23"; // ??

    // let fen = "6k1/5pp1/3p1n2/3P3Q/5Rn1/P5P1/1PPq2KP/8 w - - 0 1"; // ??
    // let fen = "6k1/5pp1/3p1n2/3P3r/5Rn1/P5PQ/1PP2q1P/7K w - - 0 33"; // ??

    // let fen = "r1b1k2r/ppppnppp/2n2q2/2b5/3NP3/2P1B3/PP3PPP/RN1QKB1R w KQkq - 0 1";
    // let fen = "6k1/3q1pp1/pp5p/1r5n/8/1P3PP1/PQ4BP/2R3K1 w - - 0 1"; // killer test

    // let fen = &games_sts(2, 8);
    // let fen = &games_sts(1, 15);

    // eprintln!("correct = {:?}", correct);

    eprintln!("fen = {:?}", fen);
    let mut g = Game::from_fen(&ts, fen).unwrap();
    // let g = g.flip_sides(&ts);

    // g.last_move = Some(Move::new_capture("D1", "D2", King, Queen));

    // let mvs = vec![
    //     "h3h5",
    // ];
    // g = g.run_moves(&ts, mvs);

    eprintln!("g.to_fen() = {:?}", g.to_fen());
    eprintln!("g = {:?}", g);

    // let hook = std::panic::take_hook();
    // std::panic::set_hook(Box::new(move |panicinfo| {
    //     let loc = panicinfo.location();
    //     debug!("Panicking, Location: {:?}", loc);
    //     hook(panicinfo)
    // }));

    // let mv = Move::Capture { from: "H5".into(), to: "G4".into(), pc: Pawn, victim: Pawn };

    // let t = 10.0;
    let t = 6.0;
    // let t = 4.0;
    // let t = 2.0;
    // let t = 0.5;
    // let t = 0.3;

    // let n = 35;
    let n = 7;
    // let n = 2;

    let t0 = std::time::Instant::now();
    let timesettings = TimeSettings::new_f64(0.0,t);
    let mut ex = Explorer::new(g.state.side_to_move, g.clone(), n, timesettings);
    ex.load_syzygy("/home/me/code/rust/rchess/tables/syzygy/").unwrap();
    ex.cfg.return_moves = true;
    ex.cfg.clear_table = false;
    // ex.cfg.num_threads = Some(6);
    ex.cfg.num_threads = Some(1);
    // ex.cfg.num_threads = None;

    ex.load_nnue("/home/me/code/rust/rchess/nn-63376713ba63.nnue").unwrap();

    // let (t_opt,t_max) = ex.timer.allocate_time(g.state.side_to_move, 1);
    // eprintln!("t_opt = {:?}", t_opt);
    // eprintln!("t_max = {:?}", t_max);
    // return;

    // let mut only_moves = HashSet::default();
    // only_moves.insert(Move::new_quiet("F5", "F1", Rook));
    // only_moves.insert(Move::new_double("b7", "b5"));
    // ex.cfg.only_moves = Some(only_moves);

    // let s0 = std::mem::size_of::<OrdMove>();
    // eprintln!("s0 = {:?}", s0);
    // return;

    // let (mv,stats) = ex.explore(&ts, None);
    // eprintln!("mv = {:?}", mv);
    // return;

    // let tt_r = ex.handle();

    ex.update_game(g.clone());
    let (res,moves,stats0) = ex.lazy_smp_2(&ts);
    let t1 = t0.elapsed();
    let t2 = t1.as_secs_f64();

    // let fen = "r4rk1/4npp1/1p4b1/1B2p3/1q1P2Q1/P3P3/5PP1/R3K2R w KQ - 0 2";
    // let g2 = Game::from_fen(&ts, fen).unwrap();
    // let zb2 = g2.zobrist;
    // eprintln!("zb2 = {:?}", zb2);

    // let zb1 = Zobrist(0xcdab13aceaa91520);
    // let zb2 = Zobrist(0x6b2bc7c01dffde39);
    // let tt2 = ex.ptr_tt.clone();
    // let si2 = ex.ptr_tt.probe(zb1);
    // eprintln!("si2 = {:?}", si2);
    // let si2 = ex.ptr_tt.probe(zb2);
    // eprintln!("si2 = {:?}", si2);

    // return;

    // println!("wat 0");
    // let fen = games_wac(1);
    // let g = Game::from_fen(&ts, &fen).unwrap();
    // ex.update_game(g.clone());
    // let (res,moves,stats0) = ex.lazy_smp_2(&ts);
    // println!("wat 1");

    // let fen = games_wac(2);
    // let g = Game::from_fen(&ts, &fen).unwrap();
    // ex.update_game(g.clone());
    // let (res,moves,stats0) = ex.lazy_smp_2(&ts);
    // println!("wat 2");

    // return;

    let best   = res.get_result().unwrap();
    let scores = res.get_scores().unwrap_or_default();

    // for m in best.moves.iter() { eprintln!("\t{:?}", m); }
    // eprintln!("\nBest move = {:>8} {:?}", best.score, best.moves[0]);
    eprintln!("\nBest move = {:>8} {:?}", best.score, best.mv);
    println!("explore lazy_smp_negamax (depth: {}) done in {:.3} seconds.",
             stats0.max_depth, t2);

    // let tt_r = ex.tt_rf.handle();

    println!();
    for (n,mv) in moves.iter().enumerate() {
        eprintln!("{}\t{:?}", n, mv);
    }

    // return;

    // for mv in moves.iter() {
    //     eprintln!("mv = {:?}", mv);
    //     let zb = g.zobrist.update_move_unchecked(&ts, &g, *mv);
    //     if let Some(si) = tt_r.get_one(&zb) {
    //         let mv1 = si.best_move;
    //         let d   = si.depth_searched;
    //         let nt  = si.node_type;
    //         eprintln!("{:?} = {:?}, {:?}", mv1, nt, d);
    //     } else {
    //         println!("wat 1");
    //     }
    // }

    // let n = 35;
    // let t = 0.5;
    // let stop = Arc::new(AtomicBool::new(false));
    // let timesettings = TimeSettings::new_f64(0.0,t);
    // let mut ex = Explorer::new(g.state.side_to_move, g.clone(), n, timesettings);
    // ex.load_syzygy("/home/me/code/rust/rchess/tables/syzygy/").unwrap();
    // let mut t1;
    // let mut t2;
    // let ((best, scores),stats0,(tt_r,tt_w)) = loop {
    //     let t0 = std::time::Instant::now();
    //     // let ((best, scores),stats0,(tt_r,tt_w)) = go(&ts, n, g.clone(), t);
    //     let ((best, scores),stats0,(tt_r,tt_w)) = ex.lazy_smp_negamax(&ts, false, false);
    //     t1 = t0.elapsed();
    //     t2 = t1.as_secs_f64();
    //     eprintln!("done in {:?}", t2);
    //     eprintln!("tt_r.len() = {:?}", tt_r.len());
    //     // {
    //     //     let mut w = tt_w.lock();
    //     //     w.purge();
    //     //     w.refresh();
    //     // }
    //     if best.score > 50000 {
    //         break ((best, scores),stats0,(tt_r,tt_w));
    //     }
    // };

    // eprintln!("tt_r.len() = {:?}", tt_r.len());
    // let s0 = tt_total_size(&tt_r);
    // eprintln!("s0 = {:?}", s0);

    // let mut scores = scores;
    // scores.sort_by_key(|x| x.score);
    // scores.reverse();
    // for s in scores.iter() {
    //     let mv = s.moves[0];
    //     eprintln!("{:?} = {:?}", mv, s.score);
    // }

    // eprintln!("tt_r.len() = {:?}", tt_r.len());

    // return;

    // let k = best.score - CHECKMATE_VALUE;
    // eprintln!("k = {:?}", k);

    // return;

    // let mut g1 = Game::from_fen(&ts, fen1).unwrap();
    // let ((best, scores),stats0,(tt_r,tt_w)) = go(&ts, n, g1.clone(), 4.0);

    // for m in mvs0.iter() {
    //     eprintln!("\t{:?}", m);
    // }

    stats0.print(t1);

    eprintln!();

    stats0.print_ebf(false);

    // let zb = g.zobrist;
    // let si = tt_r.get_one(&zb).unwrap();
    // eprintln!("si = {:?}", si);

    // let mut k = 0;
    // for (zb,sis) in tt_r.read().unwrap().iter() {
    //     // let si = sis.iter().next().unwrap();
    //     k += 1;
    // }
    // eprintln!("k = {:?}", k);

}

#[allow(unreachable_code)]
fn main_sts(sts: Option<u64>) {
    let sts = sts.unwrap();
    let mut games = read_epd(&format!("testpositions/STS/STS{}.epd", sts)).unwrap();

    // let (fen,m) = &games[0];
    // eprintln!("fen = {:?}", fen);
    // eprintln!("m = {:?}", m);
    // return;

    let n = 35;

    // let ts = Tables::new();
    // let ts = Tables::read_from_file("tables.bin").unwrap();
    let ts = Tables::read_from_file_def().unwrap();

    let timesettings = TimeSettings::new_f64(
        0.0,
        1.0,
    );

    let mut total = (0,0);
    let t0 = std::time::Instant::now();

    println!("running STS {}", sts);

    for (i,(fen,m)) in games.into_iter().enumerate() {
        let g = Game::from_fen(&ts, &fen).unwrap();

        let mut ex = Explorer::new(g.state.side_to_move, g.clone(), n, timesettings);

        let (m0,stats) = ex.explore(&ts);

        let mv = m0.clone().unwrap().0.to_algebraic(&g);
        let mv0 = m[0].replace("+", "");
        if mv0 == mv {
            // println!("#{:>2}: Correct", i);
            total.0 += 1;
            total.1 += 1;
        } else {
            total.1 += 1;
            let t = t0.elapsed().as_secs_f64() / total.1 as f64;
            println!(
                "#{:>2}: Wrong, Correct: {:>5}, engine: {:>5} ({:?}), ({}/{}), avg: {:.2}",
                i, m[0], mv, m0.unwrap().0, total.0, total.1, total.0 as f64 / total.1 as f64);
        }
    }

    println!("Score = {} / {} ({:.2})", total.0, total.1, total.0 as f64 / total.1 as f64);
    println!("Finished in {:.3} seconds.", t0.elapsed().as_secs_f64());

}

fn main7() {
    let fen = STARTPOS;
    let n = 10;

    init_logger();

    // let fen = "rnbqkbnr/ppppp1pp/8/5P2/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2";
    // let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1";

    // let fen = "5r1k/4Qpq1/4p3/1p1p2P1/2p2P2/1p2P3/3P4/BK6 b - - 0 1";

    // // // minimax = 869088, 2.08 s
    // // // AB      = 92245,  0.24 s
    // let fen = "r4q1k/5P1b/2p2n1P/p2p1P2/3P4/8/2PKN1Q1/6R1 w - - 1 34";

    // // AB = 808182 leaves, 1.87 s
    // let fen = "7k/2pq2p1/6rp/1P2p3/2Qp1n2/P2P3P/R1P2PPK/3N2R1 b - - 0 28";

    // let fen = "2rr3k/pp3pp1/1nnqbN1p/3pN3/2pP4/2P3Q1/PPB4P/R4RK1 w - - 0 1"; // WAC.001 = Qg6 = g3g6
    // let fen = "2rr3k/pp3pp1/1nnqbNQ1/3pN2p/2pP4/2P5/PPB4P/R4RK1 w - - 0 2"; // WAC.001 = Qg6 = g3g6
    // let fen = "2rr3k/pp4p1/1nnqbNpp/3pN3/2pP4/2P5/PPB4P/R4RK1 w - - 0 2"; // WAC.001

    // let fen = "5rk1/1ppb3p/p1pb4/6q1/3P1p1r/2P1R2P/PP1BQ1P1/5RKN w - -"; // WAC.003, e3g3

    // let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - "; // Position 3

    // let fen = "Q3b3/4bkp1/1q2np1p/NPp1p3/2P1P3/4BP1P/4B1P1/7K b - - 1 1"; // Correct = e6c7
    // let fen = "rnbqkb1r/pppp1ppp/8/4P3/6n1/7P/PPPNPPP1/R1BQKBNR b KQkq - 0 1"; // WAC.007, Ne3 = g4e3
    // let fen = "3q1rk1/p4pp1/2pb3p/3p4/6Pr/1PNQ4/P1PB1PP1/4RRK1 b - - 0 1"; // WAC.009, Bh2+ = d6h2

    // let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"; // Perft Position 2

    let fen = "5k2/6pp/p1qN4/1p1p4/3P4/2PKP2Q/PP3r2/3R4 b - - 0 1"; // WAC.005, Qc4 = c6c4
    // let fen = "4r3/2R3pp/q2pkp2/4p3/4P1P1/4nQ1P/PP6/2K5 w - - 0 1"; // WAC.005, color reversed, f3f5

    // let fen = "rnbqkb1r/pppp1ppp/8/4P3/6n1/7P/PPPNPPP1/R1BQKBNR b KQkq - 0 1"; // WAC.007, Ne3 = g4e3

    // let fen = "rn2kbnr/pppppppp/8/8/6b1/1QP4P/PP1PqPPN/RNB1KB1R w KQkq - 0 2"; // 1 move, then lots

    // let fen = "k7/1p6/2p5/8/3N4/8/8/7K w - - 0 1"; // Quiescence test
    // let fen = "rnbqkbnr/pppppppp/8/8/8/8/1PP1PPP1/RNBQKBNR w KQkq - 0 1";

    // let fen = "k7/2n5/4p3/3p4/2P1P3/4N3/8/7K w - - 0 1"; // SEE test
    // let fen = "k7/2n5/4p3/3p3R/2P1P1P1/4N3/8/7K w - - 0 1"; // SEE test

    // let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - "; // Position 3

    // let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"; // Perft Position 2

    // let fen = "r2rb1k1/pp1q1p1p/2n1p1p1/2bp4/5P2/PP1BPR1Q/1BPN2PP/R5K1 w - - 0 1"; // WAC.014, h3h7

    // let fen = "3q1rk1/p4pp1/2pb3p/3p4/6Pr/1PNQ4/P1PB1PP1/4RRK1 b - - 0 1"; // WAC.009, Bh2+ = d6h2

    // let fen = "8/1p4pk/6rp/3Pp3/4Qn2/2P2qP1/1B3P1P/4R1K1 b - - 1 1"; // f4h3, #2
    let fen = "6k1/6pp/3q4/5p2/QP1pB3/4P1P1/4KPP1/2r5 w - - 0 2"; // a4e8, #3
    // let fen = "5rk1/ppR1Q1p1/1q6/8/8/1P6/P2r1PPP/5RK1 b - - 0 1"; // b6f2, #-4
    // let fen = "8/p6k/1p5p/4Bpp1/8/1P3q1P/P1Q2P1K/3r4 w - - 0 2"; // c2c7, #5;
    // let fen = "1rq2k1r/p1p2p2/2B2P2/3RP2p/1b3N1p/2N4P/PPP1QPP1/2K4R w - - 1 23"; // e5e6, #9

    // let fen = "2k5/8/KP6/8/8/8/8/8 w - - 1 10"; // #12
    // let fen = "8/8/1K4k1/8/7Q/8/8/8 w - - 7 16"; // #6

    // let fen = "1k3r1r/p1p3p1/1pn3q1/3R1n2/3P4/P1B1p2p/P1PN1PPP/4QK1R w - - 0 22"; // #-10 if d2b3
    // let fen = "r1bqk1nr/ppppbppp/2n5/8/4Q3/N7/PPP1PPPP/R1B1KBNR w KQkq - 3 5"; // ??

    let fen = "r4rk1/4npp1/1p1q2b1/1B2p3/1B1P2Q1/P3P3/5PP1/R3K2R b KQ - 1 1"; // Q cap d6b4

    // let fen = "7k/1n1n4/2P5/8/5b2/8/7P/7K b - - 0 1"; // Horizon
    // let fen = "7k/8/8/r7/r7/8/p1RR4/7K w - - 0 1"; // Horizon

    // let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - "; // Position 2
    // let fen = "r3k2r/p1Ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"; // Pos 2 + pawn prom
    // let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"; // Position 4

    // /// https://www.chessprogramming.org/Caesar#HorizonEffect
    // let fen = "2kr4/3nR3/p2B1p2/1p1p1Bp1/1P1P3p/2P4P/P5PK/8 b - - 1 32"; // Horizon

    // let fen = "8/8/p1p5/1p5p/1P5p/8/PPP2K1p/4R1rk w - - 0 1";    // Rf1; id ";zugzwang.001";;
    // let fen = "1q1k4/2Rr4/8/2Q3K1/8/8/8/8 w - - 0 1";            // Kh6;  id ";zugzwang.002";;
    // let fen = "7k/5K2/5P1p/3p4/6P1/3p4/8/8 w - - 0 1";           // g5; id ";zugzwang.003";;
    // let fen = "8/6B1/p5p1/Pp4kp/1P5r/5P1Q/4q1PK/8 w - - 0 32";   // Qxh4; id ";zugzwang.004";;
    // let fen = "8/8/1p1r1k2/p1pPN1p1/P3KnP1/1P6/8/3R4 b - - 0 1"; // Nxd5; id ";zugzwang.005";;

    // let fen = "2q2rk1/p4pp1/5n1p/8/8/Q4N1P/P4PP1/5RK1 b - - 0 1";; // Null move cutoff
    // let fen = "5rk1/p4pp1/5n1p/8/8/5N1P/P4PP1/2Q2RK1 b - - 0 2";; // Null move cutoff

    fn games(i: usize) -> String {
        let mut games = read_epd("testpositions/WAC.epd").unwrap();
        // let mut games = read_epd("testpositions/STS6.epd").unwrap();
        let mut games = games.into_iter();
        let games = games.map(|x| x.0).collect::<Vec<_>>();
        games[i - 1].clone()
    }

    // XXX: STS6
    // let fen = &games(3); // 

    // XXX: WAC
    // let fen = &games(2); // b3b2 (SF says b3b7)
    // let fen = &games(4); // h6h7, #2
    // let fen = &games(6); // b6b7, #11
    // let fen = &games(7); // N g4e3
    // let fen = &games(8); // R e7f7, #7
    // let fen = &games(9); // d6h2, #-5
    // let fen = &games(17); // c4e5
    // let fen = &games(18); // a8h8, #27, Tablebase
    // let fen = &games(21); // d2h6

    // let fen = "7k/1n1n4/2P5/8/5b2/8/7P/7K b - - 0 1"; // Horizon
    // let fen = "7k/8/8/r7/r7/8/p1RR4/7K w - - 0 1"; // Horizon
    // let fen = "r1bqkb1r/1pp2ppp/p1n1pn2/3p4/3P1B2/4PQ2/PPPN1PPP/R3KBNR w KQkq - 4 6"; // ??
    // let fen = "r3kbnr/pppn1ppp/4pq2/3p1b2/3P4/P1N1PN2/1PP2PPP/R1BQKB1R b KQkq - 0 1"; // ??

    // let fen = "1QqQqQq1/r6Q/Q6q/q6Q/B2q4/q6Q/k6K/1qQ1QqRb w - - 0 1"; // all the queens

    eprintln!("fen = {:?}", fen);

    // let ts = Tables::new();
    // ts.write_to_file("tables.bin").unwrap();
    let ts = Tables::read_from_file_def().unwrap();
    // let ts = &_TABLES;
    // let ts = Tables::_new(false);

    let mut g = Game::from_fen(&ts, fen).unwrap();

    let mut timesettings = TimeSettings::new_f64(0.0,2.0);
    let mut ex = Explorer::new(g.state.side_to_move, g.clone(), n, timesettings);

    // let mut ps = vec![
    //     ("pawn x queen", Move::Capture { from: "E4".into(), to: "F5".into(), pc: Pawn, victim: Queen }),
    //     ("queen x pawn", Move::Capture { from: "E4".into(), to: "F5".into(), pc: Queen, victim: Pawn }),
    //     ("pawn x pawn", Move::Capture { from: "E4".into(), to: "D5".into(), pc: Pawn, victim: Pawn }),
    //     ("pawn x rook",   Move::Capture { from: "E4".into(), to: "D5".into(), pc: Pawn, victim: Rook }),
    //     ("bishop x pawn", Move::Capture { from: "E4".into(), to: "D5".into(), pc: Bishop, victim: Pawn }),
    //     ("quiet", Move::Quiet { from: "E4".into(), to: "E5".into(), pc: Pawn }),
    //     ("promotion", Move::Promotion { from: "E7".into(), to: "E8".into(), new_piece: Queen }),
    //     // ("EP", Move::EnPassant { from: "E5".into(), to: "D6".into(), capture: "D5".into() }),
    // ];

    // order_mvv_lva(&mut ps[..]);
    // for (s,m) in ps.iter() {
    //     eprintln!("{:>15} = {:?}", s, m);
    // }

    // let ms = vec![
    //     Move::Capture { from: "C2".into(), to: "A2".into(), pc: Rook, victim: Pawn },
    //     Move::Quiet { from: "C2".into(), to: "C8".into(), pc: Rook },
    // ];
    // let g2 = g.make_move_unchecked(&ts, ms[0]).unwrap();
    // let g3 = g.make_move_unchecked(&ts, ms[1]).unwrap();
    // let e0 = g2.evaluate(&ts).sum();
    // let e1 = g3.evaluate(&ts).sum();
    // eprintln!("e0 = {:?}", e0);
    // eprintln!("e1 = {:?}", e1);

    // let (alpha,beta) = (i32::MIN,i32::MAX);
    // // let (alpha,beta) = (-1000, 1000);
    // let maximizing = false;
    // let mut ss = SearchStats::default();

    // let m0 = Move::Capture { from: "C2".into(), to: "A2".into(), pc: Rook, victim: Pawn };
    // let m1 = Move::Quiet { from: "C2".into(), to: "C8".into(), pc: Rook };

    // let m0 = Move::Quiet { from: "F3".into(), to: "G3".into(), pc: Queen };
    // let m1 = Move::Quiet { from: "F3".into(), to: "H3".into(), pc: Queen };
    // let mm = m0;

    // let g2 = g.make_move_unchecked(&ts, mm).unwrap();
    // let mut ex2 = Explorer::new(g2.state.side_to_move, g2.clone(), n, stop.clone(), timesettings);

    // let m00 = Move::Capture { from: "A4".into(), to: "A2".into(), pc: Rook, victim: Rook };
    // let g3 = g2.make_move_unchecked(&ts, m00).unwrap();
    // let m01 = Move::Capture { from: "D2".into(), to: "A2".into(), pc: Rook, victim: Rook };
    // let g4 = g3.make_move_unchecked(&ts, m01).unwrap();

    // eprintln!("g4.zobrist = {:?}", g4.zobrist);

    // let zb2 = Zobrist(0x586f4df179d639f6);
    // let zb3 = Zobrist(0x5c7013e02d0493c8);
    // let zb4 = Zobrist(0x12724159f0aaac53);

    // return;

    // let ms = vec![
    //     // Move::Capture { from: "A4".into(), to: "A2".into(), pc: Rook, victim: Pawn },
    //     // Move::Quiet { from: "A4".into(), to: "B4".into(), pc: Rook },
    //     // Move::NullMove,
    // ];

    // eprintln!("g2 = {:?}", g2);

    // let ms = g2.search_only_captures(&ts).get_moves_unsafe();
    // let score = ex2.quiescence(
    //     &ts, &g2, ms, 0, alpha, beta, maximizing, &mut ss);
    // eprintln!("score = {:?}", score);

    // eprintln!("g = {:?}", g);
    // let firstguess = 0;
    // let (mvs,score) = ex.mtd_f(&ts, firstguess);
    // let mv = mvs[mvs.len() - 1];
    // eprintln!("s, mv = {:?}: {:?}", score, mv);

    // let g = Game::from_fen(&ts, "r6k/8/8/8/r7/8/p1RR4/6K1 w - - 2 2").unwrap();
    // eprintln!("g = {:?}", g);
    // eprintln!("g.zobrist = {:?}", g.zobrist);

    // let ms = g.search_only_captures(&ts).get_moves_unsafe();
    // for m in ms.iter() {
    //     eprintln!("m = {:?}", m);
    // }

    // let g = Game::from_fen(&ts, "3R3k/2R5/8/r7/r7/8/8/b6K w - - 0 1").unwrap();
    // let e = g.evaluate(&ts).sum();
    // eprintln!("e = {:?}", e);

    // return;

    // let fenn = "rnb1kb1r/pppppNpp/8/8/8/3n4/P1PPPPPP/R1B1KB1R w KQkq - 0 1";
    // let g0 = Game::from_fen(&ts, fenn).unwrap();

    let fen = STARTPOS;
    let mut g2 = Game::from_fen(&ts, fen).unwrap();

    // let ms0 = vec![
    //     "e2e4",
    //     "e7e6",
    //     "d2d3",
    // ];

    let ms = "g1f3 g8f6 f3e5 f6e4 b1c3 e4c3 e5c6 c3d1 c6d8 d1b2 d8f7 b2d3";

    let ms0 = ms.split(" ");
    let mut ms0 = ms0.collect::<Vec<_>>();

    // ms0.truncate(ms0.len() - 20);
    let ms0 = &ms0[..42];

    eprintln!("last move = {:?}", ms0.last().unwrap());

    // let mut g2 = g2.clone();
    // for m in ms0.into_iter() {
    //     let from = &m[0..2];
    //     let to = &m[2..4];
    //     let other = &m[4..];
    //     let mm = g2.convert_move(from, to, other).unwrap();
    //     g2 = g2.make_move_unchecked(&ts, mm).unwrap();
    // }
    // // eprintln!("hash0 = {:?}", g2.zobrist);

    // eprintln!("g2 = {:?}", g2);
    // let g2fen = g2.to_fen();
    // eprintln!("g2fen = {:?}", g2fen);
    // // // eprintln!("g0 = {:?}", g0);

    // let m0 = g2.convert_move("g7", "f6", "").unwrap();
    // // let m0 = g2.convert_move("e5", "d6", "").unwrap();

    // eprintln!("m0 = {:?}", m0);

    // let m0 = Move::EnPassant { from: "E5".into(), to: "D6".into(), capture: "D5".into() };
    // let g3 = g2.make_move_unchecked(&ts, m0).unwrap();
    // eprintln!("g3 = {:?}", g3);

    // eprintln!("g0.zobrist == g2.zobrist = {:?}", g0.zobrist == g2.zobrist);
    // g0.state.debug_equal(g2.state);

    // let fen = "4r2k/3Q2pp/4p3/p3p3/P1P1N3/5P2/1P4P1/1KR5 b - -";
    // let g2 = Game::from_fen(&ts, &fen).unwrap();

    // eprintln!("g2 = {:?}", g2);

    // let n = 35;
    // let n = 10;
    let n = 6;

    let t = 2.0;

    timesettings.increment = [t, t];
    // let mut ex0 = Explorer::new(g0.state.side_to_move, g0.clone(), n, stop.clone(), timesettings);
    let mut ex2 = Explorer::new(g2.state.side_to_move, g2.clone(), n, timesettings);

    // let moves = vec![
    //     Move::Quiet { from: "E2".into(), to: "E4".into() },
    //     Move::Quiet { from: "D2".into(), to: "D4".into() },
    //     // Move::Quiet { from: "H2".into(), to: "H3".into() },
    // ];

    // let moves = g2.search_all(&ts).get_moves_unsafe();
    // for m in moves.iter() { println!("m = {:?}", m); }

    // let t = std::time::Instant::now();
    // let (m,stats) = ex2.explore(&ts, None);
    // eprintln!("m = {:?}", m.unwrap());
    // // ex.rank_moves(&ts, true);
    // println!("explore done in {:.3} seconds.", t.elapsed().as_secs_f64());



}

/// WAC
fn main_wac(num: Option<u64>, send_url: bool) {
    // let mut games = read_ccr_onehour("ccr_onehour.txt").unwrap();
    // let mut games = read_epd("Midgames250.epd").unwrap();

    // let mut games = read_epd("testpositions/WAC.epd").unwrap();
    // let mut games = read_epd("testpositions/STS15.epd").unwrap();
    let mut games = read_epd("testpositions/iq6.epd").unwrap();

    for (fen,ms) in games.iter() {
        // eprintln!("fen, ms = {:?}: {:?}", fen, ms);
        eprintln!("ms = {:?}", ms);
    }

    if let Some(num) = num {
        games.truncate(num as usize);
    }

    let n = 35;

    // let ts = Tables::new();
    // let ts = Tables::read_from_file("tables.bin").unwrap();
    let ts = Tables::read_from_file_def().unwrap();

    let timesettings = TimeSettings::new_f64(
        0.0,
        1.0,
    );

    let mut total = (0,0);
    let t0 = std::time::Instant::now();

    println!("running WAC");

    for (i,(fen,m)) in games.into_iter().enumerate() {
        let i = i + 1;
        let g = Game::from_fen(&ts, &fen).unwrap();
        // eprintln!("g = {:?}", g);

        let mut ex = Explorer::new(g.state.side_to_move, g.clone(), n, timesettings);

        // let e = g.evaluate(&ts);
        // let (_,e_sf) = stockfish_eval(&fen, false).unwrap();

        let (m0,stats) = ex.explore(&ts);

        // let g = g.flip_sides(&ts);
        // let mut ex = Explorer::new(g.state.side_to_move, g.clone(), n, stop, timesettings);
        // let (m1,_) = ex.explore(&ts, None);

        let mv = m0.clone().unwrap().0.to_algebraic(&g);
        let mv0 = m[0].replace("+", "");
        if mv0 == mv {
            // println!("#{:>2}: Correct", i);
            total.0 += 1;
            total.1 += 1;
        } else {
            total.1 += 1;
            let t = t0.elapsed().as_secs_f64() / total.1 as f64;
            println!(
                "#{:>2}: Wrong, Correct: {:>5}, engine: {:>5} ({:?}), ({}/{}), avg: {:.2}",
                i, m[0], mv, m0.unwrap().0, total.0, total.1, total.0 as f64 / total.1 as f64);

            if send_url {
                g.open_with_lichess().unwrap();
                break;
            }

            // println!("Correct        Engine");
            // println!("{:<8}       {}", m[0], mv);
        }

    }

    println!("Score = {} / {} ({:.2})", total.0, total.1, total.0 as f64 / total.1 as f64);
    println!("Finished in {:.3} seconds.", t0.elapsed().as_secs_f64());

    // let games = read_json_fens("perft_fens.txt").unwrap();
    // let mut games = games;
    // games.truncate(1);
    // for (depth,nodes,fen) in games.into_iter() {
    //     let depth = 4;
    //     println!("FEN: {}", &fen);
    //     let (done, ((ns0,nodes0),(ns1,nodes1))) = test_stockfish(&fen, depth, false).unwrap();
    //     println!("perft depth {} done in {}", depth, done);
    //     if ns0 == ns1 {
    //         eprintln!("rchess, stockfish = {:>2} / {:>2}", ns0, ns1);
    //     } else {
    //         eprintln!("rchess, stockfish = {:>2} / {:>2} / failed ({})",
    //                   ns0, ns1, ns0 as i64 - ns1 as i64);
    //     }
    // }

}

fn main6() {
    let fens = vec![
        ("start w", "k7/8/8/4p3/3P4/8/8/7K w - - 0 1"),
        ("start b", "k7/8/8/4p3/3P4/8/8/7K b - - 0 1"),

        ("w push", "k7/8/8/3Pp3/8/8/8/7K b - - 0 1"), // push
        ("w cap", "k7/8/8/4P3/8/8/8/7K b - - 0 1"), // capture

        ("b push", "k7/8/8/8/3Pp3/8/8/7K w - - 0 2"), // push
        ("b cap", "k7/8/8/8/3p4/8/8/7K w - - 0 2"), // capture
    ];

    let n = 1;

    let ts = Tables::new();

    // let fen = fens[1].1;
    let fen = STARTPOS;
    let mut g = Game::from_fen(&ts, fen).unwrap();

    // let mut g = Game::empty();
    // g.insert_pieces_mut_unchecked(&vec![
    //     ("H1", King, White),
    //     ("A8", King, Black),
    //     // ("D4", Pawn, White),
    //     // ("E4", Pawn, Black),
    //     ("D4", Pawn, Black),
    // ]);

    // let _ = g.recalc_gameinfo_mut(&ts);
    // eprintln!("g = {:?}", g);

    // let m = Move::Capture { from: "E5".into(), to: "D4".into() };
    // let m = Move::Quiet { from: "E5".into(), to: "E4".into() };
    // let g2 = g.make_move_unchecked(&ts, &m).unwrap();

    // let ew = g.evaluate(&ts).sum_color(White);
    // let eb = g.evaluate(&ts).sum_color(Black);
    // eprintln!("ew = {:?}", ew);
    // eprintln!("eb = {:?}", eb);

    // let e = g.evaluate(&ts).sum(Black);
    // eprintln!("sum = {:?}", e);


    // for (s,fen) in fens.iter() {
    //     let mut g = Game::from_fen(&ts, fen).unwrap();
    //     let _ = g.recalc_gameinfo_mut(&ts);
    //     let e = g.sum_evaluate(&EvalParams::default(), &ts);
    //     eprintln!("{} = {:?}", s, e);
    // }

}

fn main5() {

    let fen = STARTPOS;

    // let fen = "3b4/1pN5/1P1p4/3pN2R/3kP3/K2B1bP1/1P3P2/6B1 w - - 0 1";
    // let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1";

    // let fen = "rnbqk1nr/p3ppbp/2pp2p1/8/3PP3/4BN2/PPp2PPP/1R1NK2R b Kkq - 1 9";
    // let fen = "rnbqkbnr/pp3ppp/2pp4/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR b KQkq - 1 4";

    // let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - "; // Position 2
    // let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - "; // Position 3
    // let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"; // Position 4

    // let fen = "k7/8/8/4p3/3P4/8/8/7K b - - 0 1";
    let fen1 = "k7/8/8/4p3/3P4/8/8/7K w - - 0 1";
    let fen2 = "k7/8/8/4p3/3P4/8/8/7K b - - 0 1";

    // let games = read_epd("WAC.epd").unwrap();
    // let fen = &games[1].0;

    let fen = "Q3b3/4bkp1/1q2np1p/NPp1p3/2P1P3/4BP1P/4B1P1/7K b - - 1 1"; // Correct = e6c7

    let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - "; // Position 3

    let fen1 = "5k2/6pp/p1qN4/1p1p4/3P4/2PKP2Q/PP3r2/3R4 b - - 0 1"; // WAC.005, Qc4 = c6c4
    let fen2 = "4r3/2R3pp/q2pkp2/4p3/4P1P1/4nQ1P/PP6/2K5 w - - 0 1"; // WAC.005, color reversed

    let n = 3;

    let ts = Tables::new();

    // let mut g = Game::from_fen(&ts, fen).unwrap();
    // let _ = g.recalc_gameinfo_mut(&ts);
    // // eprintln!("g = {:?}", g);

    let mut g1 = Game::from_fen(&ts, fen1).unwrap();
    let _ = g1.recalc_gameinfo_mut(&ts);
    let mut g2 = Game::from_fen(&ts, fen2).unwrap();
    let _ = g2.recalc_gameinfo_mut(&ts);

    eprintln!("g1 = {:?}", g1);
    eprintln!("g2 = {:?}", g2);

    // let mut g1 = Game::from_fen(&ts, fen1).unwrap();
    // let _ = g1.recalc_gameinfo_mut(&ts);

    // let mut g2 = Game::from_fen(&ts, fen2).unwrap();
    // let _ = g2.recalc_gameinfo_mut(&ts);

    let timesettings = TimeSettings::new_f64(10., 0.1);
    // let ex = Explorer::new(g.state.side_to_move, g.clone(), n, stop.clone(), timesettings);
    // let ex = Explorer::new(White, g.clone(), n, stop.clone(), timesettings);

    let ex1 = Explorer::new(g1.state.side_to_move, g1.clone(), n, timesettings);
    let ex2 = Explorer::new(g2.state.side_to_move, g2.clone(), n, timesettings);

    let t1 = std::time::Instant::now();
    let (mv1,stats1) = ex1.explore(&ts);
    // let (mv1,stats1) = ex1.explore(&ts, n, false);
    eprintln!("mv1 = {:?}, (c6c4)", mv1.unwrap());
    stats1.print(t1.elapsed());

    print!("\n");

    let t2 = std::time::Instant::now();
    let (mv2,stats2) = ex2.explore(&ts);
    // let (mv2,stats2) = ex2.explore(&ts, n, false);
    eprintln!("mv2 = {:?}, (f3f5)", mv2.unwrap());
    stats2.print(t2.elapsed());

    // eprintln!("stats1 = {:?}", stats1);

    // let moves = vec![
    //     Move::Quiet { from: "C4".into(), to: "C3".into() },
    //     Move::Capture { from: "B3".into(), to: "B2".into() },
    // ];
    // ex.rank_moves_list(&ts, true, moves);

    // let mut ms0 = g.search_sliding(&ts, Rook, col);
    // let mut ms1 = g.search_sliding_iter(&ts, Rook, col).collect::<Vec<_>>();
    // ms0.sort_by(|a,b| a.partial_cmp(&b).unwrap());
    // ms1.sort_by(|a,b| a.partial_cmp(&b).unwrap());
    // eprintln!("ms0 = {:?}", ms0);
    // eprintln!("ms1 = {:?}", ms1);
    // assert_eq!(ms0, ms1);

    // let t = std::time::Instant::now();
    // let m = ex.explore(&ts, ex.depth);
    // eprintln!("m = {:?}", m);
    // // ex.rank_moves(&ts, true);
    // println!("explore done in {} seconds.", t.elapsed().as_secs_f64());

    // let mut ex1 = Explorer::new(g1.state.side_to_move, g1.clone(), n, stop.clone(), timesettings);
    // let mut ex2 = Explorer::new(g2.state.side_to_move, g2.clone(), n, stop, timesettings);

    // let m = ex.explore(&ts, ex.depth);
    // eprintln!("m w = {:?}", m);
    // ex.rank_moves(&ts, true, true);

    // println!("w:");
    // let moves0 = vec![
    //     Move::Capture { from: "D4".into(), to: "E5".into() },
    //     Move::Quiet { from: "D4".into(), to: "D5".into() },
    //     Move::Quiet { from: "H1".into(), to: "H2".into() },
    // ];
    // ex1.rank_moves_list(&ts, true, moves0);
    // // ex.rank_moves(&ts, true, true);

    // println!("\nb:");
    // let moves1 = vec![
    //     Move::Capture { from: "E5".into(), to: "D4".into() },
    //     Move::Quiet { from: "E5".into(), to: "E4".into() },
    //     Move::Quiet { from: "A8".into(), to: "A7".into() },
    // ];
    // ex2.rank_moves_list(&ts, true, moves1);
    // // ex2.rank_moves(&ts, true, true);

    // let m0 = ex1.explore(&ts, n);
    // eprintln!("m0 w = {:?}", m0);
    // let m1 = ex2.explore(&ts, n);
    // eprintln!("m1 b = {:?}", m1);

    // assert_eq!(m0, Some(Move::Capture { from: "D4".into(), to: "E5".into() }));
    // assert_eq!(m1, Some(Move::Capture { from: "E5".into(), to: "D4".into() }));


    // let m = ex2.explore(&ts, ex2.depth);
    // eprintln!("m b = {:?}", m);

    // let e = g2.evaluate(&ts);

    // let mw: Score = e.material_white.iter().sum();
    // let mb: Score = e.material_black.iter().sum();
    // let mw: Score = e.piece_positions_white.iter().sum();
    // let mb: Score = e.piece_positions_black.iter().sum();

    // eprintln!("mw = {:?}", e.piece_positions_white);
    // eprintln!("mb = {:?}", e.piece_positions_black);
    // // eprintln!("mw = {:?}", e.material_white);
    // // eprintln!("mb = {:?}", e.material_black);

    // let s = g.score_positions(&ts, Pawn, !White);
    // eprintln!("s = {:?}", s);

    // let ew = g.evaluate(&ts).sum_color(White);
    // let eb = g.evaluate(&ts).sum_color(Black);
    // eprintln!("sum w = {:?}", ew);
    // eprintln!("sum b = {:?}", eb);

    // let e = g.score_material(Pawn, White);
    // eprintln!("w = {:?}", e);
    // let e = g.score_material(Pawn, Black);
    // eprintln!("b = {:?}", e);

    // let moves = g.search_all(&ts, White);
    // for m in moves.clone() {
    //     eprintln!("m = {:?}", m);
    // }
    // eprintln!("moves.len() = {:?}", moves.get_moves_unsafe().len());

    // let maximizing = ex.side == g2.state.side_to_move;
    // eprintln!("maximizing = {:?}", maximizing);

    // let alpha = (None,i32::MIN);
    // let beta  = (None,i32::MAX);
    // let (_,score) = ex._ab_search(&ts, g2, n, 1, None, alpha, beta);
    // eprintln!("score = {:?}", score);

    // let from = "e5";
    // let to = "d6";
    // let other = "";
    // let m = g.convert_move(from, to, other);
    // eprintln!("m = {:?}", m);

}

/// Perft
#[allow(unreachable_code)]
fn main_perft(depth: Option<u64>) {

    // let ts = Tables::new();
    let ts = Tables::read_from_file_def().unwrap();

    // let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    // let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -";
    // let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ";
    // let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
    // let fen = STARTPOS;

    let fen2 = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - "; // Position 2
    let fen3 = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - "; // Position 3
    let fen4 = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"; // Position 4
    let fen5 = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8  "; // Position 5

    // let fen = "r3k2r/p1p1qpb1/bn1ppnp1/3PN3/1p2P3/2N4Q/PPPBBPPP/R3K2R w KQkq - 0 2";

    // let fen = "rnb1k1nr/pppp1ppp/5q2/2b1p3/4P1P1/7P/PPPP1P2/RNBQKBNR w KQkq - 1 4";


    use rchess_engine_lib::movegen::*;

    // let gen = MoveGen::new(&ts, &g, None, 0, 0);

    let d = depth.unwrap_or(4) as Depth;

    // let fen = "3n1n2/3pkp2/Pp1ppp2/8/8/4P3/3P1PN1/B2QKR2 w - - 0 1";
    let fen = fen2;
    let mut g = Game::from_fen(&ts, fen).unwrap();

    let mut gen = MoveGen::new(&ts, &g, None, 0, 0);
    // while let Some(mv) = gen.next() {
    //     eprintln!("mv = {:?}", mv);
    // }

    // // gen.gen_castles();
    // gen.gen_pawns(MoveGenType::CapturesPromotions);
    // for mv in gen.buf().iter() {
    //     eprintln!("mv = {:?}", mv);
    // }

    let ((t,t_sf),(_,_)) = test_stockfish(&ts, fen, d as u64, true, true).unwrap();

    // let (k0,mvs0) = g.perft(&ts, d as u64);
    // eprintln!("k0 = {:?}", k0);

    // let (k1,mvs1) = MoveGen::perft(&ts, &g, d);
    // eprintln!("k1 = {:?}", k1);

    return;

    let t0 = std::time::Instant::now();
    for depth in 0..d {
        let (k,_) = MoveGen::perft(&ts, &g, depth as Depth);
        let t1 = t0.elapsed().as_secs_f64();
        eprintln!("depth {:>2} = {:>10}, done in {:.3}", depth, k, t1);
    }

    return;

    timer_loop!(4,{
        let _ = MoveGen::perft(&ts, &g, d as Depth);
    });

    timer_loop!(4,{
        let (tot,_) = g.perft(&ts, d as u64);
    });

    return;

    let n = match depth {
        None    => 4,
        Some(d) => d,
    };

    let fens = vec![STARTPOS,fen2,fen3,fen4,fen5];

    // let k0 = 0u8;
    // eprintln!("k0 = {:?}", k0);
    // let k1 = Coord::new_int(k0);

    println!("starting");
    for (k,fen) in fens.iter().enumerate() {
        let mut g = Game::from_fen(&ts, fen).unwrap();

        // let (ns0, ms) = g.perft(&ts, n);

        println!("fen #{}:", k + 1);
        let ((t,t_sf),(_,_)) = test_stockfish(&ts, fen, n, true, false).unwrap();
        println!("perft done in {} seconds.", t);
        println!("stockfish took {} seconds.", t_sf);
        println!();

    }
    return;

    // let ts = Tables::new();
    // let ts = &_TABLES;
    let ts = Tables::read_from_file_def().unwrap();
    let mut g = Game::from_fen(&ts, fen).unwrap();
    // eprintln!("g = {:?}", g);

    let ((t,t_sf),(_,_)) = test_stockfish(&ts, fen, n, true, true).unwrap();
    // let (t,(_,_)) = test_stockfish(fen, n, false).unwrap();
    println!("perft done in {} seconds.", t);
    println!("stockfish took {} seconds.", t_sf);

    // let t0 = std::time::Instant::now();
    // let (tot,_) = g.perft(&ts, n);
    // // let (tot,vs) = g.perft2(&ts, n as Depth);
    // // eprintln!("n = {:?}", n);
    // let t1 = t0.elapsed().as_secs_f64();
    // println!("perft done in {} seconds.", t1);

    let ds = vec![
        20,
        400,
        8902,
        197281,
    ];

    // for d in 1..n+1 {
    //     let t0 = std::time::Instant::now();
    //     let (tot,_) = g.perft(&ts, d);
    //     let t1 = t0.elapsed().as_secs_f64();
    //     println!("depth {:>2}: {:>12} leaves, {} leaves/sec",
    //              d, tot, pretty_print_si((tot as f64 / t1) as i64));
    //     // eprintln!("correct == tot = {:?}", ds[d as usize - 1] == tot);
    //     assert!(ds[d as usize - 1] == tot);
    // }

    // for d in 1..n+1 {
    //     let t0 = std::time::Instant::now();
    //     let (tot,_) = g.perft(&ts, d);
    //     let t1 = t0.elapsed().as_secs_f64();
    //     println!("depth {:>2}: {:>12} leaves, {} leaves/sec", d, tot, _print((tot as f64 / t1) as i32));
    // }

    // for (d, n) in vs.iter().enumerate() {
    //     if *n > 0 {
    //         println!("depth {:>2}: {:>12} leaves", d, n);
    //     }
    // }

    // println!("total:    {:>12} leaves", pretty_print_si(tot as i64));

    // println!("speed: {} leaves / second", _print((tot as f64 / t1) as i32));

}

fn main2() {

    // println!("start");
    let now = std::time::Instant::now();

    // let game = Game::new();

    // let mut g = Game::new();
    // let mut g = Game::empty();

    let mut g = Game::default();

    // g.insert_pieces_mut_unchecked(&vec![
    //     ("E1", King, White),
    //     ("E8", King, Black),
    //     ("B2", Pawn, White),
    //     ("C4", Pawn, Black),
    // ]);
    // // g.state.side_to_move = Black;
    // g.state.castling = Castling::new_with(true, true);

    // let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ";
    // let fen = "4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1";
    // let fen = "4k3/8/8/8/1Pp5/8/8/4K3 b - b3 0 1";
    // let fen = "r1bqkbnr/p1pppppp/n7/1p6/8/N7/PPPPPPPP/R1BQKBNR b Kkq - 1 3";
    // let fen = "8/2p5/3p4/KP5r/1R3pPk/8/4P3/8 b - g3";
    let fen = "r3k3/p1ppqpb1/bn2pnp1/3PN3/1p2P2r/2N1Q2p/PPPBBPPP/R3K2R w KQq - 2 2";

    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    // let fen = "1rbqkbnr/pppppppp/n7/8/3P4/8/PPPKPPPP/RNBQ1BNR w k - 3 3";

    let fen = "8/2p5/3p4/KP5r/1R3pP1/4P2k/8/8 b - - 0 2";

    let fen = "rnQq1k1r/pp3ppp/2pQ4/8/2B5/8/PPP1NnPP/RNB1K2R b KQ - 0 9";

    let fen = "8/8/8/p1Nk4/8/8/8/7K b - - 0 1";
    let fen = "rnbqkbnr/pppp1pp1/7p/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq e6 0 3";
    let fen = "8/8/3p3k/8/3R4/7K/3P4/8 w - - 0 1";

    let fen = "1k1r4/pp1b1R2/3q2p1/4p2p/2B5/4Q3/PPP2B2/2K5 w - - 0 2";
    let fen = "1k1r4/pp1b1R2/6pp/4p3/2B5/4Q3/PPP2B2/2Kq4 w - - 1 2";
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -";

    let fen = "B5KR/1r5B/6R1/2b1p1p1/2P1k1P1/1p2P2p/1P2P2P/3N1N2 w - - 0 1";

    // let mut g = Game::from_fen(&ts, "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ").unwrap();
    // let mut g = Game::from_fen(&ts, "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ").unwrap();
    // let mut g = Game::from_fen(&ts, "8/2p5/3p4/KP5r/1R3p1k/6P1/4P3/8 b - - 0 1").unwrap();
    // let mut g = Game::from_fen(&ts, fen).unwrap();

    // let mut g = Game::from_fen(&ts, "8/2p5/3p4/KP5r/1R3p1k/6P1/4P3/8 b - - 0 1").unwrap();
    // g.state.side_to_move = Black;

    // let mut g = Game::from_fen(&ts, "8/2p5/3p4/KP5r/1R3p1k/6P1/4P3/8 w - - 0 1").unwrap();

    let ts = Tables::new();
    // let mut g = Game::new();

    let mut g = Game::from_fen(&ts, fen).unwrap();

    let _ = g.recalc_gameinfo_mut(&ts);

    // // let m = Move::Quiet { from: "G2".into(), to: "G4".into() };
    // let m = Move::Castle {
    //     from:        "E1".into(),
    //     to:          "G1".into(),
    //     rook_from:   "H1".into(),
    //     rook_to:     "F1".into(),
    // };
    // let mut g = g.make_move_unchecked(&ts, &m).unwrap();
    // g.recalc_gameinfo_mut(&ts);

    eprintln!("{:?}", g);

    // let b = g.find_attacks_by_side(&ts, "B1".into(), !g.state.side_to_move, false);
    // eprintln!("b = {:?}", b);

    // let b = g.all_occupied() & !g.get(King, Black);
    // let b = !g.get(King, Black);
    // eprintln!("b = {:?}", b);

    // let moves = g._search_castles(&ts);
    // let moves = g.search_all(&ts, g.state.side_to_move);
    // let moves = g.search_sliding(&ts, Rook, g.state.side_to_move);
    // let moves = g._search_sliding_single(&ts, Rook, "D4".into(), g.state.side_to_move);
    // eprintln!("moves = {:?}", moves);

    // // let m = Move::Quiet { from: "C1".into(), to: "D1".into() };
    // let m = Move::Quiet { from: "C1".into(), to: "B1".into() };
    // let b = g.move_is_legal(&ts, &m);
    // eprintln!("b = {:?}", b);


    // eprintln!("moves.len() = {:?}", moves.get_moves_unsafe().len());
    // // eprintln!("moves.len() = {:?}", moves.len());
    // for m in moves.into_iter() {
    //     eprintln!("m = {:?}", m);
    //     // let b = g.move_is_legal(&ts, &m);
    //     // eprintln!("b = {:?}", b);
    // }

    // // // let moves = g._search_castles(&ts);
    // // let moves = g._search_pawns(None, &ts, White);
    // let moves = g._search_pawns(None, &ts, g.state.side_to_move);
    // let m = moves[1];
    // eprintln!("m = {:?}", m);

    // let m = Move::EnPassant { from: "D5".into(), to: "E6".into() };

    // let mut g = g.make_move_unchecked(&ts, &m).unwrap();
    // g.recalc_gameinfo_mut(&ts);

    // eprintln!("{:?}", g);

    // let depth = 3;
    // let (ns,cs) = g._perft(&ts, depth, true);
    // eprintln!("\nperft total    = {:?}", ns);
    // eprintln!("perft captures = {:?}", cs);

    // let moves = g.search_all(&ts, g.state.side_to_move);
    // let moves = g.search_sliding(Bishop, &ts, White);
    // let moves = g._search_pawns(None, &ts, Black);
    // let moves = g._search_promotions(&ts, None, White);
    // let m = moves[10];
    // eprintln!("m = {:?}", m);

    // let x = g.state.checkers;
    // // let x = g.move_is_legal(&ts, &m3);
    // eprintln!("x = {:?}", x);

    // eprintln!("moves.len() = {:?}", moves.len());
    // for m in moves.iter() {
    //     eprintln!("m = {:?}", m);
    // }

    // let (blockers,pinners) = g.find_slider_blockers(&ts, "A3".into());

    // // let c0 = g.get(King, White);
    // // let c0 = c0.bitscan().into();
    // let c0: Coord = "E1".into();
    // let (bs_w, ps_b) = g.find_slider_blockers(&ts, c0, White);

    // eprintln!("bs_w = {:?}", bs_w);
    // eprintln!("ps_b = {:?}", ps_b);

    // let b = g.state.pinners;
    // eprintln!("b = {:?}", b);

    // let c0: Coord = "A1".into();
    // let c1: Coord = "B3".into();
    // let b = ts.between(c0, c1);
    // eprintln!("b = {:?}", b);

    // let b = g.find_pins_absolute(&ts, White);
    // eprintln!("b = {:?}", b);

    // let moves = g.find_attacks_to(&ts, "A2".into(), Black);
    // let moves: Vec<Move> = moves.collect();

    // let k = g.find_attacks_by_side(&ts, "B1".into(), Black);
    // eprintln!("k = {:?}", k);

    // let moves = g.search_king(&ts, White);

    // let moves = g.search_sliding(Rook, &ts, White);

    // let g2 = g.make_move_unchecked(&m).unwrap();
    // eprintln!("{:?}", g2);

    // let moves = g.search_king(&ts, White);
    // let m = Move::Quiet { from: "A2".into(), to: "A1".into() };
    // let x = g.move_is_legal(&ts, &m);
    // eprintln!("x = {:?}", x);

    // let xs = g.find_attacks_by_side(&ts, "A1".into(), Black);
    // eprintln!("xs = {:?}", xs);

    // // let ms = g.search_all(&ts, White);
    // // let ms = g.search_king(White);
    // // let ms = g.search_knights(&ts, White);
    // // let ms = g.search_sliding(Rook, &ts, White);
    // // let ms = g.search_sliding(Bishop, &ts, White);
    // let ms = g.search_sliding(Queen, &ts, White);
    // // let ms = g.search_pawns(White);
    // // ms.sort_by(|a,b| a.partial_cmp(b).unwrap());

    // let b = BitBoard::new(&vec![
    //     Coord(1,1),
    //     Coord(1,0),
    //     Coord(0,1),
    //     Coord(2,1),
    //     Coord(1,2),
    //     Coord(1,3),

    //     Coord(7,0),
    //     Coord(0,7),

    //     // Coord(0,0),
    //     // Coord(7,7),

    // ]);

    // // let v: Vec<Coord> = (0..8).map(|x| Coord(1,x)).collect();
    // // let v: Vec<Coord> = (0..8).map(|x| Coord(x,x)).collect();
    // let v: Vec<Coord> = (0..8).map(|x| Coord(x,7-x)).collect();
    // let b = BitBoard::new(&v);
    // eprintln!("{:?}\n", b);

    // // let b = b.mirror_vert();
    // // let b = b.mirror_horiz();
    // // let b = b.flip_diag();
    // // let b = b.flip_antidiag();
    // // let b = b.rotate_180();
    // // let b = b.rotate_90_ccw();
    // // let b = b.rotate_45_cw();
    // let b = b.rotate_45_ccw();

    // eprintln!("{:?}", b);
    // // let x: u64 = 2u64.pow(63);
    // // eprintln!("b = {:0>64b}", x);
    // // eprintln!("b = {:0>64b}", x as i64);

    // let ms2 = vec![
    //     Move::Capture { from: Coord(0, 0), to: Coord(0, 2) },
    //     Move::Quiet { from: Coord(0, 0), to: Coord(0, 1) },
    //     Move::Quiet { from: Coord(0, 0), to: Coord(1, 0) },
    // ];
    // assert_eq!(ms, ms2);

    // let mut ms2: Vec<Move> = (0..9).into_iter()
    //     .zip(0..9)
    //     .map(|(x,y)| (Move::Quiet { from: Coord(1,1), to: Coord(x,y) }))
    //     .collect();
    // ms2.sort_by(|a,b| a.partial_cmp(b).unwrap());

    // println!("====");
    // for m in ms2.iter() {
    //     eprintln!("m = {:?}", m);
    // }

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

fn init_logger() {
    let cfg = ConfigBuilder::new()
            .set_time_level(LevelFilter::Off)
            .set_target_level(LevelFilter::Off)
            .set_thread_level(LevelFilter::Info)
        // .set_thread_level(LevelFilter::Off)
            .set_location_level(LevelFilter::Off)
            .build();

    let logfile = std::fs::File::create("test.log").unwrap();
    let log0 = WriteLogger::new(LevelFilter::Trace, cfg.clone(), logfile);

    // let log1 = TermLogger::new(LevelFilter::Trace, cfg.clone(), TerminalMode::Stderr, ColorChoice::Auto);
    let log1 = TermLogger::new(LevelFilter::Debug, cfg.clone(), TerminalMode::Stderr, ColorChoice::Auto);

    CombinedLogger::init(vec![
        log0,
        log1,
    ]).unwrap();

}

