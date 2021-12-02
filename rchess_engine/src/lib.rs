#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]

#![feature(destructuring_assignment)]
#![feature(core_intrinsics)]
#![feature(label_break_value)]
// #![feature(backtrace,backtrace_frames)]

#![allow(
    // clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::complexity,
    // clippy::correctness,
    clippy::nursery,
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

extern crate blas_src;
// extern crate openblas_src;

// extern crate nalgebra as na;

pub mod types;
pub mod bitboard;
pub mod coords;
pub mod tables;
pub mod magics;
pub mod game;
pub mod parsing;
pub mod pgn;

pub mod see;
pub mod qsearch;
pub mod search;
pub mod explore;
pub mod alphabeta;
pub mod evaluate;
pub mod timer;
pub mod tuning;
pub mod texel;
pub mod hashing;
pub mod trans_table;

pub mod opening_book;
#[cfg(feature = "syzygy")]
pub mod syzygy;

pub mod attack_maps;
// pub mod stack_game;
// pub mod gen_moves;
// pub mod lockfree_hashmap;
// pub mod material_table;
pub mod pawn_hash_table;
pub mod evmap_tables;

pub mod brain;

// #[macro_use]
pub mod searchstats;

pub mod pruning;
pub mod move_ordering;

pub mod lockless_map;

pub mod simd_test;

#[allow(clippy::all)]
pub mod util;

// #[cfg(test)]
// pub mod tests;

// #[macro_export]
// macro_rules! with_game_move {
//     ($g:expr, $fn:expr) => {
//     }
// }

// #[macro_export]
// macro_rules! named_array {
//     ($name:ident, $( $fields:ident ),* ) => {
//     };
// }

#[macro_export]
macro_rules! builder_field {
    ($field:ident, $field_type:ty) => {
        pub fn $field(mut self, $field: $field_type) -> Self {
            self.$field = $field;
            self
        }
    };
}

#[macro_export]
macro_rules! eprint_self {
    ($e:expr) => {
        eprintln!("{} = {:?}", stringify!($e), $e);
    }
}

#[macro_export]
macro_rules! stats {
    ($e:expr) => {
        #[cfg(feature = "keep_stats")]
        $e
    }
}

#[macro_export]
macro_rules! not_stats {
    ($e:expr) => {
        #[cfg(not(feature = "keep_stats"))]
        $e
    }
}

#[macro_export]
macro_rules! stats_or {
    ($e:expr,$or:expr) => {
        #[cfg(feature = "keep_stats")]
        if true {
            $e
        }
        #[cfg(not(feature = "keep_stats"))]
        if true {
            $or
        }
    }
}
