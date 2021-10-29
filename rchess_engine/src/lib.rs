#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]

#![feature(destructuring_assignment)]
#![feature(core_intrinsics)]
#![feature(label_break_value)]

#![allow(
    // clippy::all,
    // clippy::restriction,
    // clippy::pedantic,
    // clippy::nursery,
    // clippy::cargo,
    // clippy::complexity,
    // clippy::correctness,
    // clippy::nursery,
    // clippy::restriction,
    clippy::style,
    // clippy::suspicious,
    // clippy::perf,
)]

// #![warn(
//     clippy::perf,
// )]

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

pub mod types;
pub mod bitboard;
pub mod coords;
pub mod tables;
pub mod magics;
pub mod game;
pub mod parsing;

// pub mod movegen;
pub mod search;
pub mod explore;
pub mod alphabeta;
pub mod evaluate;
pub mod timer;
pub mod tuning;
pub mod hashing;
pub mod trans_table;
pub mod searchstats;
pub mod pruning;
pub mod move_ordering;

// pub mod lockless_map;

#[allow(clippy::all)]
pub mod util;

// #[cfg(test)]
// pub mod tests;

