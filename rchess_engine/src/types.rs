
pub use crate::bitboard::*;
pub use crate::coords::*;
pub use crate::game::*;
pub use crate::hashing::*;

// pub use log::{debug, error};
pub use log::{debug, error, warn, info, trace};
use evmap_derive::ShallowCopy;

use serde::{Serialize,Deserialize};

pub use self::{Color::*,Piece::*};

// pub static PIECES: [Piece; 6] = [Pawn,Rook,Knight,Bishop,Queen,King];
pub static PIECES: [Piece; 6] = [Pawn,Knight,Bishop,Rook,Queen,King];

pub type Depth = u8;

#[derive(Serialize,Deserialize,Debug,Hash,Eq,PartialEq,PartialOrd,ShallowCopy,Clone,Copy)]
// #[derive(Debug,Hash,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub enum Color {
    White,
    Black,
}

#[derive(Serialize,Deserialize,Debug,Hash,Eq,PartialEq,Ord,PartialOrd,ShallowCopy,Clone,Copy)]
// #[derive(Debug,Hash,Eq,PartialEq,Ord,PartialOrd,ShallowCopy,Clone,Copy)]
// #[derive(Debug,Hash,Eq,PartialEq,Ord,PartialOrd,Clone,Copy)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

// #[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
// pub struct SimpleMove {
//     from: Coord,
//     to:   Coord,
// }

// #[derive(Serialize,Deserialize,Eq,PartialEq,Ord,PartialOrd,Hash,ShallowCopy,Clone,Copy)]
#[derive(Serialize,Deserialize,Eq,PartialEq,Hash,ShallowCopy,Clone,Copy)]
// #[derive(Serialize,Deserialize,Ord,Eq,PartialEq,Hash,ShallowCopy,Clone,Copy)]
pub enum Move {
    Quiet              { from: Coord, to: Coord, pc: Piece },
    PawnDouble         { from: Coord, to: Coord },
    // Capture            { from: Coord, to: Coord },
    Capture            { from: Coord, to: Coord, pc: Piece, victim: Piece },
    // EnPassant          { from: Coord, to: Coord, capture: Coord },
    EnPassant          { from: Coord, to: Coord, capture: Coord },
    Castle             { from: Coord, to: Coord, rook_from: Coord, rook_to: Coord },
    Promotion          { from: Coord, to: Coord, new_piece: Piece },
    PromotionCapture   { from: Coord, to: Coord, new_piece: Piece, victim: Piece },
    NullMove,
}

// #[derive(Serialize,Deserialize,Eq,PartialEq,Hash,ShallowCopy,Clone,Copy)]
// // #[derive(Serialize,Deserialize,Ord,Eq,PartialEq,Hash,ShallowCopy,Clone,Copy)]
// pub enum Move2 {
//     Quiet              { side: Color, from: Coord, to: Coord, pc: Piece },
//     PawnDouble         { side: Color, from: Coord, to: Coord },
//     // Capture            { side: Color, from: Coord, to: Coord },
//     Capture            { side: Color, from: Coord, to: Coord, pc: Piece, victim: Piece },
//     // EnPassant          { side: Color, from: Coord, to: Coord, capture: Coord },
//     EnPassant          { side: Color, from: Coord, to: Coord, capture: Coord },
//     Castle             { side: Color, from: Coord, to: Coord, rook_from: Coord, rook_to: Coord },
//     Promotion          { side: Color, from: Coord, to: Coord, new_piece: Piece },
//     PromotionCapture   { side: Color, from: Coord, to: Coord, new_piece: Piece, victim: Piece },
//     NullMove,
// }

impl Ord for Move {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use Move::*;
        use std::cmp::Ordering::*;
        match (self, other) {
            (PromotionCapture { .. }, PromotionCapture { .. }) => Equal,
            (PromotionCapture { .. }, _)                       => Greater,
            (_, PromotionCapture { .. })                       => Less,

            (Promotion { .. }, Promotion { .. })               => Equal,
            (Promotion { .. }, _)                              => Greater,
            (_, Promotion { .. })                              => Less,

            (EnPassant { .. }, EnPassant { .. })               => Equal,
            (EnPassant { .. }, _)                              => Greater,
            (_, EnPassant { .. })                              => Less,

            (Capture { .. }, Capture { .. })                   => Equal,
            (Capture { .. }, _)                                => Greater,
            (_, Capture { .. })                                => Less,

            (Castle { .. }, Castle { .. })                     => Equal,
            (Castle { .. }, _)                                 => Greater,
            (_, Castle { .. })                                 => Less,

            (Quiet { .. }, Quiet { .. })                       => Equal,
            (Quiet { .. }, _)                                  => Greater,
            (_, Quiet { .. })                                  => Less,

            (PawnDouble { .. }, PawnDouble { .. })             => Equal,
            (_, PawnDouble { .. })                             => Equal,
            (PawnDouble { .. }, _)                             => Equal,

            _                                                  => {
                debug!("cmp move: {:?}, {:?}", self, other);
                panic!("cmp move: {:?}, {:?}", self, other);
                // Equal
            },
        }
    }
}

impl PartialOrd for Move {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone)]
pub enum GameEnd {
    Checkmate { win: Color },
    Stalemate,
    Draw,
    DrawRepetition,
    Error,
}

pub type GameResult<T> = std::result::Result<T, GameEnd>;

// impl<T> MoveResult<T> {
//     pub fn unwrap(self) -> T {
//         match self {
//             Self::Legal(t) => t,
//             _              => panic!("MoveResult unwrap panic"),
//         }
//     }
// }

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone)]
pub enum Outcome {
    Checkmate(Color),
    Stalemate,
    Moves(Vec<Move>),
}

impl Outcome {

    // pub fn get_moves_unsafe(&self) -> &[Move] {
    pub fn get_moves_unsafe(&self) -> Vec<Move> {
        match self {
            // Self::Moves(v) => &v,
            Self::Moves(v) => v.clone(),
            _              => panic!("get_moves_unsafe"),
        }
    }

    pub fn is_end(&self) -> bool {
        match self {
            Self::Moves(_) => false,
            _              => true,
        }
    }
}

impl IntoIterator for Outcome {
    type Item = Move;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Moves(ms) => ms.into_iter(),
            // _               => vec![].into_iter(),
            _ => panic!("iterating over checkmate"),
        }
    }
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

    pub fn filter_promotion(&self) -> bool {
        match self {
            &Move::Promotion { .. }        => true,
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

    pub fn filter_castle(&self) -> bool {
        match self {
            &Move::Castle { .. } => true,
            _                    => false,
        }
    }

    pub fn sq_from(&self) -> Coord {
        match self {
            &Move::Quiet { from, .. }            => from,
            &Move::PawnDouble { from, .. }       => from,
            &Move::Capture { from, .. }          => from,
            &Move::EnPassant { from, .. }        => from,
            &Move::Promotion { from, .. }        => from,
            &Move::PromotionCapture { from, .. } => from,
            &Move::Castle { from, .. }           => from,
            &Move::NullMove                      => unimplemented!(),
        }
    }

    pub fn sq_to(&self) -> Coord {
        match self {
            &Move::Quiet { to, .. }            => to,
            &Move::PawnDouble { to, .. }       => to,
            &Move::Capture { to, .. }          => to,
            &Move::EnPassant { to, .. }        => to,
            &Move::Promotion { to, .. }        => to,
            &Move::PromotionCapture { to, .. } => to,
            &Move::Castle { to, .. }           => to,
            &Move::NullMove                    => unimplemented!(),
        }
    }

    // pub fn capture(&self) -> Option<(Piece,Piece)>

    pub fn piece(&self) -> Option<Piece> {
        match self {
            &Move::Capture { pc, .. }              => Some(pc),
            &Move::EnPassant { .. }                => Some(Pawn),

            // XXX: pawn or new_piece ???
            &Move::PromotionCapture { victim, .. } => Some(Pawn),
            &Move::Promotion { new_piece, .. }     => Some(Pawn),

            &Move::PawnDouble { .. }               => Some(Pawn),
            &Move::Quiet { pc, .. }                => Some(pc),
            &Move::Castle { to, .. }               => Some(King),
            // _                                   => None,
            &Move::NullMove                        => None,
        }
    }

    pub fn victim(&self) -> Option<Piece> {
        match self {
            &Move::Capture { victim, .. }          => Some(victim),
            &Move::EnPassant { .. }                => Some(Pawn),
            &Move::PromotionCapture { victim, .. } => Some(victim),
            _                                      => None,
        }
    }

    pub fn reverse(&self, g: &Game) -> Option<Self> {
        match *self {
            Move::Quiet      { from, to, pc } => {
                if pc == Pawn {
                    None
                } else {
                    Some(Move::Quiet { from: to, to: from, pc })
                }
            },
            Move::PawnDouble { from, to } => {
                // Move::PawnDouble { from: to, to: from }
                None
            },
            Move::Capture    { from, to, pc, victim } => {
                // Move::Capture    { from: to, to: from, pc: victim, victim: pc }
                None
            },
            Move::EnPassant  { from, to, capture } => {
                // Move::EnPassant  { from: to, to: from, capture }
                // panic!("reverse en passant?")
                None
            },
            Move::Promotion  { from, to, new_piece } => {
                // Move::Promotion  { from: to, to: from, new_piece }
                None
            },
            Move::PromotionCapture  { from, to, new_piece, victim } => {
                // Move::PromotionCapture  { from: to, to: from, new_piece }
                // panic!("reverse promotion capture?")
                None
            },
            Move::Castle     { from, to, rook_from, rook_to } => {
                // Move::Castle     { from: to, to: from, rook_from, rook_to }
                None
            },
            Move::NullMove                    => unimplemented!(),
        }
    }

    pub fn to_long_algebraic(&self) -> String {
        match self {
            Move::Promotion { new_piece, .. } | Move::PromotionCapture { new_piece, .. } => {
                let c = match new_piece {
                    Queen  => 'q',
                    Knight => 'n',
                    Rook   => 'r',
                    Bishop => 'b',
                    _      => panic!("Bad promotion"),
                };
                format!("{:?}{:?}{}", self.sq_from(), self.sq_to(), c).to_ascii_lowercase()
            },
            _ => {
                format!("{:?}{:?}", self.sq_from(), self.sq_to()).to_ascii_lowercase()
            },
        }
    }

    pub fn to_algebraic(&self, g: &Game) -> String {

        if let Some((_,pc)) = g.get_at(self.sq_from()) {
            let from = format!("{:?}", self.sq_from()).to_ascii_lowercase();
            let to = format!("{:?}", self.sq_to()).to_ascii_lowercase();

            let cs = vec!['a','b','c','d','e','f','g','h'];

            let cap = if self.filter_all_captures() { "x" } else { "" };

            // let check = if (g.state.checkers.unwrap() & BitBoard::single(self.sq_from()))

            match pc {
                Pawn   => {
                    if self.filter_all_captures() {
                        let cc = cs[self.sq_from().0 as usize];
                        format!("{}x{}", cc, to)
                    } else {
                        format!("{}", to)
                    }
                },
                Rook   => {
                    format!("R{}{:?}", cap, self.sq_to())
                },
                Knight => {
                    format!("N{}{:?}", cap, self.sq_to())
                },
                Bishop => {
                    format!("B{}{:?}", cap, self.sq_to())
                },
                Queen  => {
                    format!("Q{}{:?}", cap, self.sq_to())
                },
                King   => {
                    format!("K{}{:?}", cap, self.sq_to())
                },
            }
        } else {
            unimplemented!()
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

impl<T> std::ops::Index<Color> for [T; 2] {
    type Output = T;
    fn index(&self, col: Color) -> &Self::Output {
        let sq = if col == White { 0 } else { 1 };
        &self[sq]
    }
}

impl<T> std::ops::IndexMut<Color> for [T; 2] {
    fn index_mut(&mut self, col: Color) -> &mut Self::Output {
        let sq = if col == White { 0 } else { 1 };
        &mut self[sq]
    }
}

impl<T> std::ops::Index<Piece> for [T; 6] {
    type Output = T;
    fn index(&self, pc: Piece) -> &Self::Output {
        &self[pc.index()]
    }
}

impl<T> std::ops::IndexMut<Piece> for [T; 6] {
    fn index_mut(&mut self, pc: Piece) -> &mut Self::Output {
        &mut self[pc.index()]
    }
}

// impl Iterator for Piece {
//     type Item = Piece;
//     fn next(&mut self) -> Option<Self::Item> {
//         match self {
//         }
//     }
// }

struct PcIter(Option<Piece>, bool);

impl Iterator for PcIter {
    type Item = Piece;
    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            Some(King) => {
                self.0 = None;
                Some(King)
            },
            Some(Queen) if self.1 => {
                self.0 = None;
                Some(Queen)
            },
            Some(pc) => {
                self.0 = Some(PIECES[pc.index() + 1]);
                Some(pc)
            },
            None => None,
        }
    }
}

impl Piece {

    pub fn index(self) -> usize {
        match self {
            Pawn   => 0,
            Knight => 1,
            Bishop => 2,
            Rook   => 3,
            Queen  => 4,
            King   => 5,
        }
    }

    pub fn iter_pieces() -> impl Iterator<Item = Piece> {
        PcIter(Some(Pawn), false)
    }

    pub fn iter_nonking_pieces() -> impl Iterator<Item = Piece> {
        PcIter(Some(Pawn), true)
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

    fn print_char(&self) -> char {
        match self {
            Pawn   => 'p',
            Rook   => 'R',
            Knight => 'N',
            Bishop => 'B',
            Queen  => 'Q',
            King   => 'K',
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

impl std::fmt::Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Move::*;
        match self {
            Quiet              { from, to, pc } => {
                f.write_str(&format!("Qt {} {:?}{:?}", pc.print_char(), from, to))?;
            },
            PawnDouble         { from, to } => {
                f.write_str(&format!("Db   {:?}{:?}", from, to))?;
            },
            Capture            { from, to, pc, victim } => {
                f.write_str(&format!("Cp {} {:?}{:?}", pc.print_char(), from, to))?;
            },
            EnPassant          { from, to, capture } => {
                f.write_str(&format!("EP   {:?}{:?}", from, to))?;
            },
            Promotion          { from, to, new_piece } => {
                f.write_str(&format!("Prom {:?}{:?}={}", from, to, new_piece.print_char()))?;
            },
            PromotionCapture   { from, to, new_piece, victim } => {
                f.write_str(&format!("PCap {:?}{:?}={}", from, to, new_piece.print_char()))?;
            },
            Castle             { from, to, rook_from, rook_to } => {
                f.write_str(&format!("Cast {:?}{:?}", from, to))?;
            },
            NullMove => {
                f.write_str(&format!("NullMove"))?;
            },
        }
        Ok(())
    }
}
