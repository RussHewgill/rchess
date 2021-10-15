#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]

pub mod types;
pub mod bitboard;
pub mod coords;
pub mod tables;
pub mod game;
pub mod parsing;

pub mod search;
pub mod explore;
pub mod evaluate;
pub mod timer;

pub mod util;

#[cfg(test)]
pub mod tests;

