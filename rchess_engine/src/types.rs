
pub use crate::bitboard::*;
pub use crate::coords::*;
pub use crate::game::*;
pub use crate::hashing::*;
// pub use self::packed_move::*;

// pub use crate::evaluate::Score;

// pub use log::{debug, error};
pub use log::{debug, error, warn, info, trace};
use evmap_derive::ShallowCopy;
use derive_new::new;

use serde::{Serialize,Deserialize};

pub use self::{Color::*,Piece::*};

// pub static PIECES: [Piece; 6] = [Pawn,Rook,Knight,Bishop,Queen,King];
pub static PIECES: [Piece; 6] = [Pawn,Knight,Bishop,Rook,Queen,King];

// pub use self::score::*;
// mod score {
//     use derive_more::*;
//     use evmap_derive::ShallowCopy;
//     use serde::{Serialize,Deserialize};
//     #[derive(Debug,Deref,Eq,Ord,PartialEq,PartialOrd,Hash,Clone,Copy,
//              Index,Add,Sub,Mul,Div,Sum,Neg,
//              AddAssign,MulAssign,
//              From,Into,AsRef,AsMut,
//              Serialize,Deserialize,ShallowCopy
//     )]
//     #[repr(transparent)]
//     pub struct Score(i32);
//     impl Score {
//         pub fn get(&self) -> i32 {
//             self.0
//         }
//     }
// }

pub type Score = i32;
// pub type Score = i16;

// pub fn scale_score_to_i8(s: i16) -> i8 {
//     const K: i16 = 258;
//     (s / K) as i8
// }

pub fn scale_score_to_i8(s: i32) -> i8 {
    const K: i32 = 16909320;
    (s / K) as i8
}

pub type Depth = i16;

#[derive(Serialize,Deserialize,Debug,Hash,Eq,PartialEq,PartialOrd,Ord,ShallowCopy,Clone,Copy)]
// #[derive(Debug,Hash,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub enum Color {
    White = 0,
    Black = 1,
}

#[derive(Debug,Default,Hash,Eq,PartialEq,PartialOrd,ShallowCopy,Clone,Copy,Serialize,Deserialize,new)]
pub struct ByColor<T> {
    pub white:  T,
    pub black:  T,
}

impl<T> ByColor<T> {
    pub fn get(&self, side: Color) -> &T {
        match side {
            White => &self.white,
            Black => &self.black,
        }
    }

    pub fn insert_mut(&mut self, side: Color, t: T) {
        match side {
            White => self.white = t,
            Black => self.black = t,
        }
    }

}

#[derive(Serialize,Deserialize,Debug,Hash,Eq,PartialEq,Ord,PartialOrd,ShallowCopy,Clone,Copy)]
// #[derive(Debug,Hash,Eq,PartialEq,Ord,PartialOrd,ShallowCopy,Clone,Copy)]
// #[derive(Debug,Hash,Eq,PartialEq,Ord,PartialOrd,Clone,Copy)]
pub enum Piece {
    Pawn   = 0,
    Knight = 1,
    Bishop = 2,
    Rook   = 3,
    Queen  = 4,
    King   = 5,
}

/// Quiet              { from: Coord, to: Coord, pc: Piece },
/// PawnDouble         { from: Coord, to: Coord },
/// Capture            { from: Coord, to: Coord, pc: Piece, victim: Piece },
/// EnPassant          { from: Coord, to: Coord, capture: Coord },
/// Castle             { from: Coord, to: Coord, rook_from: Coord, rook_to: Coord },
/// Promotion          { from: Coord, to: Coord, new_piece: Piece },
/// PromotionCapture   { from: Coord, to: Coord, new_piece: Piece, victim: Piece },
/// NullMove,
#[derive(Serialize,Deserialize,Eq,PartialEq,Hash,ShallowCopy,Clone,Copy)]
// #[derive(Serialize,Deserialize,Hash,ShallowCopy,Clone,Copy)]
pub enum Move {

    Quiet              { from: Coord, to: Coord, pc: Piece },
    PawnDouble         { from: Coord, to: Coord },
    // Capture            { from: Coord, to: Coord, pc: Piece, victim: Piece },
    Capture            { from: Coord, to: Coord, pcs: PackedPieces },
    EnPassant          { from: Coord, to: Coord, capture: Coord },
    // Castle             { from: Coord, to: Coord, rook_from: Coord, rook_to: Coord },
    Castle             { side: Color, kingside: bool },
    Promotion          { from: Coord, to: Coord, new_piece: Piece },
    // PromotionCapture   { from: Coord, to: Coord, new_piece: Piece, victim: Piece },
    PromotionCapture   { from: Coord, to: Coord, pcs: PackedPieces },
    NullMove,


    // Quiet              { side: Color, from: Coord, to: Coord, pc: Piece },
    // PawnDouble         { side: Color, from: Coord, to: Coord },
    // Capture            { side: Color, from: Coord, to: Coord, pc: Piece, victim: Piece },
    // EnPassant          { side: Color, from: Coord, to: Coord, capture: Coord },
    // Castle             { side: Color, from: Coord, to: Coord, rook_from: Coord, rook_to: Coord },
    // Promotion          { side: Color, from: Coord, to: Coord, new_piece: Piece },
    // PromotionCapture   { side: Color, from: Coord, to: Coord, new_piece: Piece, victim: Piece },
}

// impl Eq for Move {}
// impl PartialEq for Move {
//     fn eq(&self, other: &Self) -> bool {
//         panic!();
//     }
// }

#[derive(Serialize,Deserialize,Eq,PartialEq,Ord,PartialOrd,Hash,ShallowCopy,Clone,Copy)]
pub struct PackedPieces(u8);

/// New, get
impl PackedPieces {
    pub fn new(pc1: Piece, pc2: Piece) -> Self {
        let first = pc1.index() as u8;
        let second = (pc2.index() as u8) << 3;
        Self(first | second)
    }

    pub fn get(&self) -> (Piece,Piece) {
        (self.first(),self.second())
    }

    pub fn first(&self) -> Piece {
        Piece::from_index(self.0 & 0b111)
    }

    pub fn second(&self) -> Piece {
        Piece::from_index((self.0 & 0b111000) >> 3)
    }

}

/// Specific getter aliases
impl PackedPieces {

    pub fn new_piece(&self) -> Piece { self.first() }

    pub fn victim(&self) -> Piece { self.second() }

}

// #[derive(Serialize,Deserialize,Eq,PartialEq,Hash,ShallowCopy,Clone,Copy)]
// pub enum Move2 {
//     Quiet              { from: Coord, to: Coord, pc: Piece },
//     PawnDouble         { from: Coord, to: Coord },
//     // Capture            { from: Coord, to: Coord, pc: Piece, victim: Piece },
//     Capture            { from: Coord, to: Coord, pcs: PackedPieces },
//     EnPassant          { from: Coord, to: Coord, capture: Coord },
//     // Castle             { from: Coord, to: Coord, rook_from: Coord, rook_to: Coord },
//     Castle             { side: Color, kingside: bool },
//     Promotion          { from: Coord, to: Coord, new_piece: Piece },
//     // PromotionCapture   { from: Coord, to: Coord, new_piece: Piece, victim: Piece },
//     PromotionCapture   { from: Coord, to: Coord, pcs: PackedPieces },
//     NullMove,
// }

/// Const Castles
impl Move {

    // pub const fn new_castle_const(side: Color, kingside: bool) -> Move {
    //     if kingside {}
    // }

    // pub const CASTLE_KINGSIDE_BETWEEN: [[Coord; 2]; 2] = [
    //     [Sq::F1.to(), Sq::G1.to()],
    //     [Sq::F8.to(), Sq::G8.to()],
    // ];

    pub const CASTLE_QUEENSIDE_BETWEEN: [BitBoard; 2] = [
        // BitBoard::new(&[Sq::C1.to(), Sq::D1.to()]),
        // BitBoard::new(&[Sq::C8.to(), Sq::D8.to()]),
        BitBoard(0x000000000000000c),
        BitBoard(0x0c00000000000000),
    ];

    pub const CASTLE_KINGSIDE: [Move; 2] = [
        Move::Castle { side: White, kingside: true },
        Move::Castle { side: Black, kingside: true },
    ];

    pub const CASTLE_QUEENSIDE: [Move; 2] = [
        Move::Castle { side: White, kingside: false },
        Move::Castle { side: Black, kingside: false },
    ];

    // pub const CASTLE_KINGSIDE: [Move; 2] = [
    //     Move::Castle {
    //         from:      Sq::E1.to(),
    //         to:        Sq::G1.to(),
    //         rook_from: Sq::H1.to(),
    //         rook_to:   Sq::F1.to(),
    //     },
    //     Move::Castle {
    //         from:      Sq::E8.to(),
    //         to:        Sq::G8.to(),
    //         rook_from: Sq::H8.to(),
    //         rook_to:   Sq::F8.to(),
    //     },
    // ];

    // pub const CASTLE_QUEENSIDE: [Move; 2] = [
    //     Move::Castle {
    //         from:      Sq::E1.to(),
    //         to:        Sq::C1.to(),
    //         rook_from: Sq::A1.to(),
    //         rook_to:   Sq::D1.to(),
    //     },
    //     Move::Castle {
    //         from:      Sq::E8.to(),
    //         to:        Sq::C8.to(),
    //         rook_from: Sq::A8.to(),
    //         rook_to:   Sq::D8.to(),
    //     },
    // ];

}

/// Castle getters
impl Move {

    const CASTLE_KINGSIDE_SQUARES: [((Coord,Coord),(Coord,Coord)); 2] = [
        ((Sq::E1.to(), Sq::G1.to()), (Sq::H1.to(), Sq::F1.to())),
        ((Sq::E8.to(), Sq::G8.to()), (Sq::H8.to(), Sq::F8.to())),
    ];

    const CASTLE_QUEENSIDE_SQUARES: [((Coord,Coord),(Coord,Coord)); 2] = [
        ((Sq::E1.to(), Sq::C1.to()), (Sq::A1.to(), Sq::D1.to())),
        ((Sq::E8.to(), Sq::C8.to()), (Sq::A8.to(), Sq::D8.to())),
    ];

    pub fn castle_moves(self) -> ((Coord,Coord),(Coord,Coord)) {
        match self {
            Move::Castle { side, kingside } => {
                if kingside {
                    Self::CASTLE_KINGSIDE_SQUARES[side]
                } else {
                    Self::CASTLE_QUEENSIDE_SQUARES[side]
                }
            },
            _ => unimplemented!(),
        }
    }

    pub fn castle_king_mv(self) -> (Coord,Coord) {
        match self {
            Move::Castle { side, kingside } => {
                if kingside {
                    Self::CASTLE_KINGSIDE_SQUARES[side].0
                } else {
                    Self::CASTLE_QUEENSIDE_SQUARES[side].0
                }
            },
            _ => unimplemented!(),
        }
    }

    pub fn castle_rook_mv(self) -> (Coord,Coord) {
        match self {
            Move::Castle { side, kingside } => {
                if kingside {
                    Self::CASTLE_KINGSIDE_SQUARES[side].1
                } else {
                    Self::CASTLE_QUEENSIDE_SQUARES[side].1
                }
            },
            _ => unimplemented!(),
        }
    }

}

/// Conveninience builders
impl Move {

    pub fn new_quiet<T: Into<Coord>>(from: T, to: T, pc: Piece) -> Move {
        Move::Quiet { from: from.into(), to: to.into(), pc }
    }
    pub fn new_capture<T: Into<Coord>>(from: T, to: T, pc: Piece, victim: Piece) -> Move {
        // Move::Capture { from: from.into(), to: to.into(), pc, victim }
        Move::Capture { from: from.into(), to: to.into(), pcs: PackedPieces::new(pc, victim) }
    }
    pub fn new_double<T: Into<Coord>>(from: T, to: T) -> Move {
        Move::PawnDouble { from: from.into(), to: to.into() }
    }

    pub fn new_castle(side: Color, kingside: bool) -> Self {
        Move::Castle { side, kingside }
    }

    pub fn new_promotion<T: Into<Coord>>(from: T, to: T, new_piece: Piece) -> Move {
        Move::Promotion { from: from.into(), to: to.into(), new_piece }
    }
    pub fn new_promotion_cap<T: Into<Coord>>(from: T, to: T, new_piece: Piece, victim: Piece) -> Move {
        let pcs = PackedPieces::new(new_piece,victim);
        Move::PromotionCapture { from: from.into(), to: to.into(), pcs }
    }

}

#[cfg(feature = "nope")]
mod packed_move {
    use super::*;
    use crate::tables::Tables;

    use packed_struct::prelude::*;
    pub use packed_struct::PackedStruct;

    // #[derive(Debug,Eq,PartialEq,Clone,Copy,Serialize,Deserialize)]
    // pub struct PackedMove2 {
    //     #[serde(serialize_with = "PackedMove2::ser")]
    //     #[serde(deserialize_with = "PackedMove2::de")]
    //     mv:   PackedMove,
    // }

    // impl PackedMove2 {
    //     pub fn ser<S>(&self, s: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
    //         unimplemented!()
    //     }
    //     pub fn de<'de, D>(d: D) -> Result<PackedMove, D::Error>
    //     where D: serde::Deserializer<'de>
    //     {
    //         unimplemented!()
    //     }
    // }

    #[derive(Debug,Eq,PartialEq,Clone,Copy,PackedStruct,Serialize,Deserialize)]
    // #[derive(Debug,Eq,PartialEq,Clone,Copy,PackedStruct)]
    #[packed_struct(bit_numbering = "msb0")]
    pub struct PackedMove {
        #[packed_field(bits = "0..6")]
        _from:   Integer<u8, packed_bits::Bits::<6>>,
        #[packed_field(bits = "6..12")]
        _to:     Integer<u8, packed_bits::Bits::<6>>,
        #[packed_field(bits = "12..14")]
        _prom:   Integer<u8, packed_bits::Bits::<2>>,
        #[packed_field(bits = "14..16")]
        _flag:   Integer<u8, packed_bits::Bits::<2>>,
    }

    impl PackedMove {

        pub fn convert_to_move(&self, ts: &Tables, g: &Game) -> Move {
            let from = self.from().into();
            let to   = self.to().into();

            // TODO: other move, castle, ep etc
            let other = "";
            let mv = g._convert_move(from, to, other, false);
            mv.unwrap()
            // unimplemented!()
        }

        pub fn convert_from_move(mv: Move) -> Self {
            let flag = match mv {
                Move::Promotion { .. } | Move::PromotionCapture { .. } => 1,
                Move::EnPassant { .. }                                 => 2,
                Move::Castle { .. }                                    => 3,
                _                                                      => 0,
            };

            match mv {
                Move::Promotion { new_piece, .. } | Move::PromotionCapture { new_piece, .. } =>
                    Self::new(mv.sq_from().into(), mv.sq_to().into(), Some(new_piece), flag),
                _ => Self::new(mv.sq_from().into(), mv.sq_to().into(), None, flag),
            }
        }

        // pub fn from(&self) -> u8 {
        //     u8::from(self._from)
        // }
        // pub fn to(&self) -> u8 {
        //     u8::from(self._to)
        // }

        pub fn from(&self) -> Coord {
            Coord::new_int(u8::from(self._from))
        }
        pub fn to(&self) -> Coord {
            Coord::new_int(u8::from(self._to))
        }

        // pub fn prom(&self) -> Option<Piece> {
        //     Self::convert_to_piece(u8::from(self._prom))
        // }

        // pub fn new(from: u8, to: u8, prom: Option<Piece>, flag: ) -> Self {
        pub fn new(from: u8, to: u8, prom: Option<Piece>, flag: u8) -> Self {

            Self {
                _from:  from.into(),
                _to:    to.into(),
                _prom:  Self::convert_from_piece(prom).into(),
                _flag:  flag.into(),
            }
        }

        fn convert_from_piece(pc: Option<Piece>) -> u8 {
            match pc {
                None         => 0,
                Some(Knight) => 0,
                Some(Bishop) => 1,
                Some(Rook)   => 2,
                Some(Queen)  => 3,
                _            => panic!("PackedMove: bad promotion: {:?}", pc),
            }
        }

        // pub fn convert_to_piece(pc: u8) -> Option<Piece> {
        pub fn convert_to_piece(pc: u8) -> Piece {
            match pc {
                0 => Knight,
                1 => Bishop,
                2 => Rook,
                3 => Queen,
                _ => unimplemented!(),
            }
        }

    }

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

// impl Ord for Move {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         use Move::*;
//         use std::cmp::Ordering::*;
//         match (self, other) {
//             (PromotionCapture { .. }, PromotionCapture { .. }) => Equal,
//             (PromotionCapture { .. }, _)                       => Greater,
//             (_, PromotionCapture { .. })                       => Less,

//             (Promotion { .. }, Promotion { .. })               => Equal,
//             (Promotion { .. }, _)                              => Greater,
//             (_, Promotion { .. })                              => Less,

//             (EnPassant { .. }, EnPassant { .. })               => Equal,
//             (EnPassant { .. }, _)                              => Greater,
//             (_, EnPassant { .. })                              => Less,

//             (Capture { .. }, Capture { .. })                   => Equal,
//             (Capture { .. }, _)                                => Greater,
//             (_, Capture { .. })                                => Less,

//             (Castle { .. }, Castle { .. })                     => Equal,
//             (Castle { .. }, _)                                 => Greater,
//             (_, Castle { .. })                                 => Less,

//             (Quiet { .. }, Quiet { .. })                       => Equal,
//             (Quiet { .. }, _)                                  => Greater,
//             (_, Quiet { .. })                                  => Less,

//             (PawnDouble { .. }, PawnDouble { .. })             => Equal,
//             (_, PawnDouble { .. })                             => Equal,
//             (PawnDouble { .. }, _)                             => Equal,

//             _                                                  => {
//                 debug!("cmp move: {:?}, {:?}", self, other);
//                 panic!("cmp move: {:?}, {:?}", self, other);
//                 // Equal
//             },
//         }
//     }
// }

// impl PartialOrd for Move {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         Some(self.cmp(other))
//     }
// }

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Serialize,Deserialize)]
pub enum GameEnd {
    Checkmate { win: Color },
    Stalemate,
    Draw,
    DrawRepetition,
    Error,
}

pub type GameResult<T> = std::result::Result<T, GameEnd>;

// #[derive(Debug,Eq,PartialEq,PartialOrd,Clone)]
#[derive(Debug,PartialEq,Clone)]
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

/// Filters, zeroing
impl Move {

    pub fn is_zeroing(&self) -> bool {
        match self {
            Move::Quiet { pc, .. }        => *pc == Pawn,
            Move::PawnDouble { .. }       => true,
            Move::Capture { .. }          => true,
            Move::EnPassant { .. }        => true,
            Move::Castle { .. }           => true,
            Move::Promotion { .. }        => true,
            Move::PromotionCapture { .. } => true,
            _                             => false,
        }
    }

    pub fn filter_pawndouble(&self) -> bool {
        match self {
            &Move::PawnDouble { .. } => true,
            _                        => false,
        }
    }

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

    pub fn filter_capture_or_promotion(&self) -> bool {
        self.filter_all_captures() | self.filter_promotion()
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

}

/// Getters
impl Move {

    pub fn sq_from(&self) -> Coord {
        match self {
            &Move::Quiet { from, .. }            => from,
            &Move::PawnDouble { from, .. }       => from,
            &Move::Capture { from, .. }          => from,
            &Move::EnPassant { from, .. }        => from,
            &Move::Promotion { from, .. }        => from,
            &Move::PromotionCapture { from, .. } => from,
            // &Move::Castle { .. }                 => from,
            &Move::Castle { .. }                 => self.castle_king_mv().0,
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
            // &Move::Castle { to, .. }           => to,
            &Move::Castle { .. }               => self.castle_king_mv().1,
            &Move::NullMove                    => unimplemented!(),
        }
    }

    // pub fn capture(&self) -> Option<(Piece,Piece)>

    pub fn piece(&self) -> Option<Piece> {
        match self {
            // &Move::Capture { pc, .. }              => Some(pc),
            &Move::Capture { pcs, .. }             => Some(pcs.first()),
            &Move::EnPassant { .. }                => Some(Pawn),

            // XXX: pawn or new_piece ???
            &Move::PromotionCapture { .. }         => Some(Pawn),
            &Move::Promotion { .. }                => Some(Pawn),

            &Move::PawnDouble { .. }               => Some(Pawn),
            &Move::Quiet { pc, .. }                => Some(pc),
            &Move::Castle { .. }                   => Some(King),
            // _                                   => None,
            &Move::NullMove                        => None,
        }
    }

    pub fn victim(&self) -> Option<Piece> {
        match self {
            // &Move::Capture { victim, .. }          => Some(victim),
            &Move::Capture { pcs, .. }             => Some(pcs.second()),
            &Move::EnPassant { .. }                => Some(Pawn),
            // &Move::PromotionCapture { victim, .. } => Some(victim),
            &Move::PromotionCapture { pcs, .. }    => Some(pcs.second()),
            _                                      => None,
        }
    }

    pub fn new_piece(&self) -> Option<Piece> {
        match self {
            Move::Promotion { new_piece, .. }        => Some(*new_piece),
            // Move::PromotionCapture { new_piece, .. } => Some(*new_piece),
            Move::PromotionCapture { pcs, .. }       => Some(pcs.first()),
            _                                        => None,
        }
    }

}

impl Move {

    // pub fn reverse(&self, g: &Game) -> Option<Self> {
    //     match *self {
    //         Move::Quiet      { from, to, pc } => {
    //             if pc == Pawn {
    //                 None
    //             } else {
    //                 Some(Move::Quiet { from: to, to: from, pc })
    //             }
    //         },
    //         Move::PawnDouble { from, to } => {
    //             // Move::PawnDouble { from: to, to: from }
    //             None
    //         },
    //         Move::Capture    { from, to, pc, victim } => {
    //             // Move::Capture    { from: to, to: from, pc: victim, victim: pc }
    //             None
    //         },
    //         Move::EnPassant  { from, to, capture } => {
    //             // Move::EnPassant  { from: to, to: from, capture }
    //             // panic!("reverse en passant?")
    //             None
    //         },
    //         Move::Promotion  { from, to, new_piece } => {
    //             // Move::Promotion  { from: to, to: from, new_piece }
    //             None
    //         },
    //         Move::PromotionCapture  { from, to, new_piece, victim } => {
    //             // Move::PromotionCapture  { from: to, to: from, new_piece }
    //             // panic!("reverse promotion capture?")
    //             None
    //         },
    //         Move::Castle     { .. } => {
    //             // Move::Castle     { from: to, to: from, rook_from, rook_to }
    //             None
    //         },
    //         Move::NullMove                    => unimplemented!(),
    //     }
    // }

    pub fn to_long_algebraic(&self) -> String {
        match self {
            Move::Promotion { new_piece, .. } => {
                let c = match new_piece {
                    Queen  => 'q',
                    Knight => 'n',
                    Rook   => 'r',
                    Bishop => 'b',
                    _      => panic!("Bad promotion"),
                };
                format!("{:?}{:?}{}", self.sq_from(), self.sq_to(), c).to_ascii_lowercase()
            },
            Move::PromotionCapture { pcs, .. } => {
                let new_piece = pcs.first();
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
                        let cc = cs[self.sq_from().file() as usize];
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

    pub fn fold<T>(self, white: T, black: T) -> T {
        match self {
            White => white,
            Black => black,
        }
    }

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

impl std::ops::BitXor<bool> for Color {
    type Output = Color;

    #[inline]
    fn bitxor(self, flip: bool) -> Color {
        // Color::from_white(self.is_white() ^ flip)
        let b = (self == White) ^ flip;
        if b { White } else { Black }
    }
}

impl<T> std::ops::Index<Color> for [T; 2] {
    type Output = T;
    fn index(&self, side: Color) -> &Self::Output {
        &self[side as usize]
        // unsafe { self.get_unchecked(side as usize) }
    }
}

impl<T> std::ops::IndexMut<Color> for [T; 2] {
    fn index_mut(&mut self, side: Color) -> &mut Self::Output {
        &mut self[side as usize]
        // unsafe { self.get_unchecked_mut(side as usize) }
    }
}

impl<T> std::ops::Index<Piece> for [T; 5] {
    type Output = T;
    fn index(&self, pc: Piece) -> &Self::Output {
        &self[pc as usize]
        // assert!(pc != King);
        // unsafe { self.get_unchecked(pc as usize) }
    }
}

impl<T> std::ops::IndexMut<Piece> for [T; 5] {
    fn index_mut(&mut self, pc: Piece) -> &mut Self::Output {
        &mut self[pc as usize]
        // assert!(pc != King);
        // unsafe { self.get_unchecked_mut(pc as usize) }
    }
}

impl<T> std::ops::Index<Piece> for [T; 6] {
    type Output = T;
    fn index(&self, pc: Piece) -> &Self::Output {
        &self[pc as usize]
        // unsafe { self.get_unchecked(pc as usize) }
    }
}

impl<T> std::ops::IndexMut<Piece> for [T; 6] {
    fn index_mut(&mut self, pc: Piece) -> &mut Self::Output {
        &mut self[pc as usize]
        // unsafe { self.get_unchecked_mut(pc as usize) }
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

    pub fn from_char(c: char) -> Self {
        match c {
            'P' => Pawn,
            'N' => Knight,
            'B' => Bishop,
            'R' => Rook,
            'Q' => Queen,
            'K' => King,
            _ => panic!("Piece::from_index bad {:?}", c),
        }
    }

    pub fn from_index(x: u8) -> Self {
        match x {
            0 => Pawn,
            1 => Knight,
            2 => Bishop,
            3 => Rook,
            4 => Queen,
            5 => King,
            _ => panic!("Piece::from_index bad {:?}", x),
        }
    }

    #[inline(always)]
    pub fn index(self) -> usize {
        self as usize
    }

    // pub fn index(self) -> usize {
    //     match self {
    //         Pawn   => 0,
    //         Knight => 1,
    //         Bishop => 2,
    //         Rook   => 3,
    //         Queen  => 4,
    //         King   => 5,
    //     }
    // }

    pub fn iter_pieces() -> impl Iterator<Item = Piece> {
        PcIter(Some(Pawn), false)
    }

    pub fn iter_nonking_pieces() -> impl Iterator<Item = Piece> {
        PcIter(Some(Pawn), true)
    }

    pub fn iter_nonking_nonpawn_pieces() -> impl Iterator<Item = Piece> {
        PcIter(Some(Knight), true)
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

    pub fn print_char(&self) -> char {
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
            // Capture            { from, to, pc, victim } => {
            Capture { from, to, pcs } => {
                // f.write_str(&format!("Cp {} {:?}{:?}", pc.print_char(), from, to))?;
                f.write_str(&format!("Cp {} {:?}{:?}", pcs.first().print_char(), from, to))?;
            },
            EnPassant          { from, to, capture } => {
                f.write_str(&format!("EP   {:?}{:?}", from, to))?;
            },
            Promotion          { from, to, new_piece } => {
                f.write_str(&format!("Prom {:?}{:?}={}", from, to, new_piece.print_char()))?;
            },
            // PromotionCapture   { from, to, new_piece, victim } => {
            PromotionCapture   { from, to, pcs } => {
                // f.write_str(&format!("PCap {:?}{:?}={}", from, to, new_piece.print_char()))?;
                f.write_str(&format!("PCap {:?}{:?}={}", from, to, pcs.first().print_char()))?;
            },
            Castle             { .. } => {
                let (from, to) = self.castle_king_mv();
                let (rook_from, rook_to) = self.castle_rook_mv();
                f.write_str(&format!("Cast {:?}{:?}", from, to))?;
            },
            NullMove => {
                f.write_str(&format!("NullMove"))?;
            },
        }
        Ok(())
    }
}
