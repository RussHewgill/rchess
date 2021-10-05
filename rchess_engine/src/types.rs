
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
    Quiet      { from: Coord, to: Coord },
    PawnPush   { from: Coord, to: Coord },
    Capture    { from: Coord, to: Coord },
    EnPassant  { from: Coord, to: Coord },
    Promotion  { from: Coord, to: Coord, new_piece: Piece },
    Castle     { from: Coord, to: Coord, rook: Coord },
}
