
pub use crate::bitboard::*;
pub use crate::coords::*;
pub use crate::game::*;

pub use self::{Color::*,Piece::*};

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub enum Color {
    White,
    Black,
}

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub enum Piece {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub enum Move {
    Quiet              { from: Coord, to: Coord },
    PawnDouble         { from: Coord, to: Coord },
    // Capture    { from: Coord, to: Coord, victim: Piece },
    Capture            { from: Coord, to: Coord },
    EnPassant          { from: Coord, to: Coord },
    Promotion          { from: Coord, to: Coord, new_piece: Piece },
    PromotionCapture   { from: Coord, to: Coord, new_piece: Piece },
    Castle             { from: Coord, to: Coord, rook_from: Coord, rook_to: Coord },
}

impl Move {

    pub fn filter_quiet(&self) -> bool {
        match self {
            &Move::Quiet { .. }      => true,
            // TODO: ?
            // &Move::PawnDouble { .. } => true,
            _                        => false,
        }
    }

    pub fn filter_all_captures(&self) -> bool {
        match self {
            &Move::Capture { .. }          => true,
            &Move::EnPassant { .. }        => true,
            &Move::PromotionCapture { .. } => true,
            _                              => false,
        }
    }

    pub fn filter_en_passant(&self) -> bool {
        match self {
            &Move::EnPassant { .. }        => true,
            _                              => false,
        }
    }

    pub fn sq_from(&self) -> Coord {
        match self {
            &Move::Quiet { from, .. } => from,
            &Move::PawnDouble { from, .. } => from,
            &Move::Capture { from, .. } => from,
            &Move::EnPassant { from, .. } => from,
            &Move::Promotion { from, .. } => from,
            &Move::PromotionCapture { from, .. } => from,
            &Move::Castle { from, .. } => from,
            // _ => unimplemented!(),
        }
    }

    pub fn sq_to(&self) -> Coord {
        match self {
            &Move::Quiet { to, .. } => to,
            &Move::PawnDouble { to, .. } => to,
            &Move::Capture { to, .. } => to,
            &Move::EnPassant { to, .. } => to,
            &Move::Promotion { to, .. } => to,
            &Move::PromotionCapture { to, .. } => to,
            &Move::Castle { to, .. } => to,
            // _ => unimplemented!(),
        }
    }

}

impl Move {

    pub fn reverse(&self) -> Self {
        match *self {
            Move::Quiet      { from, to } => {
                Move::Quiet      { from: to, to: from }
            },
            Move::PawnDouble { from, to } => {
                Move::PawnDouble { from: to, to: from }
            },
            Move::Capture    { from, to } => {
                Move::Capture    { from: to, to: from }
            },
            Move::EnPassant  { from, to } => {
                Move::EnPassant  { from: to, to: from }
            },
            Move::Promotion  { from, to, new_piece } => {
                Move::Promotion  { from: to, to: from, new_piece }
            },
            Move::PromotionCapture  { from, to, new_piece } => {
                Move::PromotionCapture  { from: to, to: from, new_piece }
            },
            Move::Castle     { from, to, rook_from, rook_to } => {
                Move::Castle     { from: to, to: from, rook_from, rook_to }
            },
        }
    }

}

impl Color {
    // dark mode needs reversed
    pub fn print(&self) -> char {
        match self {
            // White => char::from_u32(0x25A1).unwrap(),
            // Black => char::from_u32(0x25A0).unwrap(),
            Black => char::from_u32(0x25A1).unwrap(),
            White => char::from_u32(0x25A0).unwrap(),
        }
    }
}

impl Piece {

    pub fn score(&self) -> i32 {
        match self {
            Pawn   => 1,
            Rook   => 5,
            Knight => 3,
            Bishop => 3,
            Queen  => 9,
            King   => 10000,
        }
    }

    pub fn print(&self, c: Color) -> char {
        // backward on dark terminal
        match c {
            Black => match self {
                Pawn   => char::from_u32(0x2659).unwrap(),
                Rook   => char::from_u32(0x2656).unwrap(),
                Knight => char::from_u32(0x2658).unwrap(),
                Bishop => char::from_u32(0x2657).unwrap(),
                Queen  => char::from_u32(0x2655).unwrap(),
                King   => char::from_u32(0x2654).unwrap(),
            },
            White => match self {
                Pawn   => char::from_u32(0x265F).unwrap(),
                Rook   => char::from_u32(0x265C).unwrap(),
                Knight => char::from_u32(0x265E).unwrap(),
                Bishop => char::from_u32(0x265D).unwrap(),
                Queen  => char::from_u32(0x265B).unwrap(),
                King   => char::from_u32(0x265A).unwrap(),
            },
        }
    }
}

impl Default for Color {
    fn default() -> Self { White }
}

impl std::ops::Not for Color {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            White => Black,
            Black => White,
        }
    }
}

