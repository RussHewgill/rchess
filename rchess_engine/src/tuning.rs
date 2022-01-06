
use crate::evaluate::TaperedScore;
use crate::explore::ExHelper;
use crate::types::*;
// use crate::tables::*;
use crate::evaluate::*;

pub use self::piece_square_tables::*;

// use rchess_macros::EPIndex;

use std::path::Path;
use std::cell::RefCell;

use serde::{Serialize,Deserialize};
use serde_big_array::BigArray;
use derive_new::new;

#[derive(Debug,Clone,Copy)]
pub struct SParams {
    max_ply:                  Depth,

    value_checkmate:          Score,
    value_stalemate:          Score,

    lmr_min_moves:            Depth,
    lmr_min_ply:              Depth,
    lmr_min_depth:            Depth,

    lmr_reduction:            Depth,
    lmr_ply_const:            Depth,

    qs_recaps_only:           Depth,

    null_prune_min_depth:     Depth,
    null_prune_min_phase:     Phase,
    null_prune_reduction:     Depth,

    rfp_min_depth:            Depth,
    rfp_margin:               Score,

    futility_min_alpha:       Score,
    futility_margin:          Score,

    history_max:              Score, // 20 * 20

}

impl Default for SParams {
    fn default() -> Self {
        Self {
            max_ply:                 220,

            value_checkmate:         100_000_000,
            value_stalemate:         0,

            lmr_min_moves:           2,
            lmr_min_ply:             3,
            lmr_min_depth:           3,

            lmr_reduction:           3,
            lmr_ply_const:           6,

            qs_recaps_only:          5,

            null_prune_min_depth:    3,
            null_prune_min_phase:    200,
            null_prune_reduction:    2,

            rfp_min_depth:           8,
            rfp_margin:              100,

            futility_min_alpha:      31000,
            futility_margin:         300,

            history_max:             400, // 20 * 20

        }
    }
}

pub use self::const_params::*;
mod const_params {
    use crate::types::*;

    pub const MAX_SEARCH_PLY: Depth = 220;

    /// -2147482414
    pub const VALUE_INVALID: Score   = Score::MIN + 1234;

    pub const CHECKMATE_VALUE: Score = 100_000_000;
    pub const TB_WIN_VALUE: Score    = 90_000_000;
    // pub const KNOWN_WIN_VALUE: Score = 80_000_000;

    // pub const TB_WIN_MAX: Score = TB_WIN_VALUE 


    // pub const STALEMATE_VALUE: Score = 20_000_000;
    // pub const DRAW_VALUE: Score = 20_000_000;
    pub const STALEMATE_VALUE: Score = 0;
    pub const DRAW_VALUE: Score = 0;

    // pub const CHECKMATE_VALUE: Score = 32000;
    // pub const STALEMATE_VALUE: Score = 31000;

    // pub const TB_WIN_VALUE: Score    = 90_000_000;
    // pub const KNOWN_WIN_VALUE: Score = 80_000_000;

    pub const LMR_MIN_MOVES: Depth = 2;
    pub const LMR_MIN_PLY: Depth = 3;
    pub const LMR_MIN_DEPTH: Depth = 3;

    pub const LMR_REDUCTION: Depth = 3;
    pub const LMR_PLY_CONST: Depth = 6;

    pub const QS_RECAPS_ONLY: Depth = 5;
    // pub static QS_RECAPS_ONLY: Depth = 100;

    pub const NULL_PRUNE_MIN_DEPTH: Depth = 3;
    pub const NULL_PRUNE_MIN_PHASE: Phase = 200;
    pub const NULL_PRUNE_REDUCTION: Depth = 2;

    pub const RFP_MIN_DEPTH: Depth = 8;
    pub const RFP_MARGIN: Score = 100;

    // pub const FUTILITY_MIN_ALPHA: Score = 95_000_000;
    pub const FUTILITY_MIN_ALPHA: Score = 31000;
    // pub const FUTILITY_MARGIN: Score = 200;
    pub const FUTILITY_MARGIN: Score = 300;

    pub const HISTORY_MAX: Score = 400; // 20 * 20

}

pub use self::misc_functions::*;
mod misc_functions {
    use crate::types::*;

    // // TODO: tune
    // pub fn depth_stat_bonus(ply: Depth) -> Score {
    //     let ply = ply as Score;
    //     (ply * ply).min(250)
    // }

    pub const fn futility_move_count(improving: bool, depth: Depth) -> Depth {
        let i = if improving { 1 } else { 0 };
        (3 + depth * depth) / (2 - i)
    }

    pub fn lmr_reduction(d: Depth, ms: u8) -> Depth {
        if ms < 4 {
            return 1;
        }

        (d / 3).min(1)

        // let mut r =
        // unimplemented!()
    }

}

pub trait Tunable {
    const LEN: usize;
    fn to_arr(&self) -> Vec<Score>;
    fn from_arr(v: &[Score]) -> Self;
    fn to_arr_mut(&mut self) -> Vec<&mut Score>;
    fn update_exhelper(&self, exhelper: &mut ExHelper, mid: bool);
}

mod piece_square_tables {
    use crate::types::*;
    use crate::tables::*;
    use crate::evaluate::*;
    use crate::tuning::*;

    use serde::{Serialize,Deserialize};
    use serde_big_array::BigArray;

    // #[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
    #[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy,Serialize,Deserialize)]
    pub struct PcTables {
        // tables:      [[Score; 64]; 6],
        #[serde(with = "BigArray")]
        pub pawn:       [Score; 64],
        #[serde(with = "BigArray")]
        pub knight:     [Score; 64],
        #[serde(with = "BigArray")]
        pub bishop:     [Score; 64],
        #[serde(with = "BigArray")]
        pub rook:       [Score; 64],
        #[serde(with = "BigArray")]
        pub queen:      [Score; 64],
        #[serde(with = "BigArray")]
        pub king:       [Score; 64],
    }

    impl std::ops::Index<Piece> for PcTables {
        type Output = [Score; 64];
        fn index(&self, pc: Piece) -> &Self::Output {
            match pc {
                Pawn   => &self.pawn,
                Knight => &self.knight,
                Bishop => &self.bishop,
                Rook   => &self.rook,
                Queen  => &self.queen,
                King   => &self.king,
            }
        }
    }

    impl std::ops::IndexMut<Piece> for PcTables {
        fn index_mut(&mut self, pc: Piece) -> &mut Self::Output {
            match pc {
                Pawn   => &mut self.pawn,
                Knight => &mut self.knight,
                Bishop => &mut self.bishop,
                Rook   => &mut self.rook,
                Queen  => &mut self.queen,
                King   => &mut self.king,
            }
        }
    }

    impl Default for PcTables {
        fn default() -> Self {
            Self::new_mid()
            // Self::empty()
        }
    }

    /// print
    impl PcTables {

        pub fn map_color(score: Score) -> termcolor::Color {
            let x = (score.clamp(-127, 127) - 127).abs() as u8;

            // let x = x / 4;
            let x = x / 6;

            let i = x as f64;
            let r = (f64::sin(0.024 * i + 0.0) * 127.0 + 128.0).round();
            let g = (f64::sin(0.024 * i + 2.0) * 127.0 + 128.0).round();
            let b = (f64::sin(0.024 * i + 4.0) * 127.0 + 128.0).round();
            let r = r as u8;
            let g = g as u8;
            let b = b as u8;
            termcolor::Color::Rgb(r,g,b)
        }

        fn _print_table(ss: [Score; 64]) -> std::io::Result<()> {
            use std::io::{self, Write};
            use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

            let mut stdout = StandardStream::stdout(ColorChoice::Always);

            for y in 0..8 {
                let y = 7 - y;
                for x in 0..8 {
                    // println!("(x,y) = ({},{}), coord = {:?}", x, y, Coord(x,y));
                    // print!("{:>3?},", ps.get(Pawn, Coord(x,y)));
                    let s = ss[Coord::new(x,y)];

                    stdout.set_color(ColorSpec::new().set_fg(Some(Self::map_color(s))))?;
                    stdout.flush()?;
                    write!(&mut stdout, "{:>3?}", s)?;

                    // print!("{:>3?},", s);
                }
                println!();
            }
            Ok(())
        }

        pub fn print_table(&self, pc: Piece) -> std::io::Result<()> {
            match pc {
                Pawn   => Self::_print_table(self.pawn)?,
                Knight => Self::_print_table(self.knight)?,
                Bishop => Self::_print_table(self.bishop)?,
                Rook   => Self::_print_table(self.rook)?,
                Queen  => Self::_print_table(self.queen)?,
                King   => Self::_print_table(self.king)?,
            }
            Ok(())
        }

        pub fn print_tables(&self) -> std::io::Result<()> {

            println!("Pawn: ");
            Self::_print_table(self.pawn)?;
            println!("Knight: ");
            Self::_print_table(self.knight)?;
            println!("Bishop: ");
            Self::_print_table(self.bishop)?;
            println!("Rook: ");
            Self::_print_table(self.rook)?;
            println!("Queen: ");
            Self::_print_table(self.queen)?;
            println!("King: ");
            Self::_print_table(self.king)?;

            Ok(())
        }

    }
    /// get
    impl PcTables {

        pub fn get<T: Into<Coord>>(&self, pc: Piece, col: Color, c0: T) -> Score {
            let c1: Coord = c0.into();
            let c1 = if col == White { c1 } else { Coord::new(c1.file(),7 - c1.rank()) };
            self[pc][c1]
        }

    }

    /// Generate
    impl PcTables {

        fn empty() -> Self {
            Self {
                pawn:       [0; 64],
                knight:     [0; 64],
                bishop:     [0; 64],
                rook:       [0; 64],
                queen:      [0; 64],
                king:       [0; 64],
            }
        }

        pub fn new_mid() -> Self {
            let pawn   = Self::gen_pawns();
            let knight = Self::gen_knights();
            let bishop = Self::gen_bishops();
            let rook   = Self::gen_rooks();
            let queen  = Self::gen_queens();
            let king   = Self::gen_kings_opening();
            Self {
                pawn,
                knight,
                bishop,
                rook,
                queen,
                king,
            }
        }

        pub fn new_end() -> Self {
            let pawn   = Self::gen_pawns();
            let knight = Self::gen_knights();
            let bishop = Self::gen_bishops();
            let rook   = Self::gen_rooks();
            let queen  = Self::gen_queens();
            let king   = Self::gen_kings_endgame();
            Self {
                pawn,
                knight,
                bishop,
                rook,
                queen,
                king,
            }
        }

    }

    /// Initial values
    impl PcTables {

        fn gen_pawns() -> [Score; 64] {
            Self::transform_arr([
                0,   0,  0,  0,  0,  0,  0,  0,
                50, 50, 50, 50, 50, 50, 50, 50,
                10, 10, 20, 30, 30, 20, 10, 10,
                5,   5, 10, 25, 25, 10,  5,  5,
                0,   0,  0, 20, 20,  0,  0,  0,
                5,  -5,-10,  0,  0,-10, -5,  5,
                5,  10, 10,-20,-20, 10, 10,  5,
                0,   0,  0,  0,  0,  0,  0,  0,
            ])
        }

        fn gen_rooks() -> [Score; 64] {
            Self::transform_arr([
                 0,  0,  0,  0,  0,  0,  0,  0,
                 5, 10, 10, 10, 10, 10, 10,  5,
                -5,  0,  0,  0,  0,  0,  0, -5,
                -5,  0,  0,  0,  0,  0,  0, -5,
                -5,  0,  0,  0,  0,  0,  0, -5,
                -5,  0,  0,  0,  0,  0,  0, -5,
                -5,  0,  0,  0,  0,  0,  0, -5,
                 0,  0,  0,  5,  5,  0,  0,  0,
            ])
        }

        fn gen_knights() -> [Score; 64] {
            let mut scores: Vec<(&str,Score)> = vec![];

            let out = Self::transform_arr([
                -50,-40,-30,-30,-30,-30,-40,-50,
                -40,-20,  0,  0,  0,  0,-20,-40,
                -30,  0, 10, 15, 15, 10,  0,-30,
                -30,  5, 15, 20, 20, 15,  5,-30,
                -30,  0, 15, 20, 20, 15,  0,-30,
                -30,  5, 10, 15, 15, 10,  5,-30,
                -40,-20,  0,  5,  5,  0,-20,-40,
                -50,-40,-30,-30,-30,-30,-40,-50,
            ]);

            out
        }

        fn gen_bishops() -> [Score; 64] {
            Self::transform_arr([
                -20,-10,-10,-10,-10,-10,-10,-20,
                -10,  0,  0,  0,  0,  0,  0,-10,
                -10,  0,  5, 10, 10,  5,  0,-10,
                -10,  5,  5, 10, 10,  5,  5,-10,
                -10,  0, 10, 10, 10, 10,  0,-10,
                -10, 10, 10, 10, 10, 10, 10,-10,
                -10,  5,  0,  0,  0,  0,  5,-10,
                -20,-10,-10,-10,-10,-10,-10,-20,
            ])
        }

        fn gen_queens() -> [Score; 64] {
            Self::transform_arr([
                -20,-10,-10, -5, -5,-10,-10,-20,
                -10,  0,  0,  0,  0,  0,  0,-10,
                -10,  0,  5,  5,  5,  5,  0,-10,
                 -5,  0,  5,  5,  5,  5,  0, -5,
                  0,  0,  5,  5,  5,  5,  0, -5,
                -10,  5,  5,  5,  5,  5,  0,-10,
                -10,  0,  5,  0,  0,  0,  0,-10,
                -20,-10,-10, -5, -5,-10,-10,-20,
            ])
        }

        pub fn gen_kings_opening() -> [Score; 64] {
            Self::transform_arr([
                -30,-40,-40,-50,-50,-40,-40,-30,
                -30,-40,-40,-50,-50,-40,-40,-30,
                -30,-40,-40,-50,-50,-40,-40,-30,
                -30,-40,-40,-50,-50,-40,-40,-30,
                -20,-30,-30,-40,-40,-30,-30,-20,
                -10,-20,-20,-20,-20,-20,-20,-10,
                 20, 20,  0,  0,  0,  0, 20, 20,
                 20, 30, 10, -5,  0, 10, 30, 20,
            ])
        }

        pub fn gen_kings_endgame() -> [Score; 64] {
            Self::transform_arr([
                -50,-40,-30,-20,-20,-30,-40,-50,
                -30,-20,-10,  0,  0,-10,-20,-30,
                -30,-10, 20, 30, 30, 20,-10,-30,
                -30,-10, 30, 40, 40, 30,-10,-30,
                -30,-10, 30, 40, 40, 30,-10,-30,
                -30,-10, 20, 30, 30, 20,-10,-30,
                -30,-30,  0,  0,  0,  0,-30,-30,
                -50,-30,-30,-30,-30,-30,-30,-50
            ])
        }

        fn transform_arr(xs: [Score; 64]) -> [Score; 64] {
            let mut out = [0; 64];
            for y in 0..8 {
                for x in 0..8 {
                    out[Coord::new(x,7 - y)] = xs[Coord::new(x,y)];
                }
            }
            out
        }

    }

    impl Tunable for PcTables {
        const LEN: usize = 64 * 6;
        fn from_arr(v: &[Score]) -> Self {
            const N: usize = 64;
            assert!(v.len() >= N * 6);
            let mut out = Self::empty();
            out.pawn.copy_from_slice(&v[..N]);
            out.knight.copy_from_slice(&v[N..N*2]);
            out.bishop.copy_from_slice(&v[N*2..N*3]);
            out.rook.copy_from_slice(&v[N*3..N*4]);
            out.queen.copy_from_slice(&v[N*4..N*5]);
            out.king.copy_from_slice(&v[N*5..N*6]);
            out
        }
        fn to_arr(&self) -> Vec<Score> {
            let mut out = vec![];
            out.extend_from_slice(&self.pawn);
            out.extend_from_slice(&self.knight);
            out.extend_from_slice(&self.bishop);
            out.extend_from_slice(&self.rook);
            out.extend_from_slice(&self.queen);
            out.extend_from_slice(&self.king);
            out
        }
        fn to_arr_mut(&mut self) -> Vec<&mut Score> {
            let mut xs = vec![];
            xs.extend(self.pawn.iter_mut());
            xs.extend(self.knight.iter_mut());
            xs.extend(self.bishop.iter_mut());
            xs.extend(self.rook.iter_mut());
            xs.extend(self.queen.iter_mut());
            xs.extend(self.king.iter_mut());
            xs
        }
        fn update_exhelper(&self, exhelper: &mut ExHelper, mid: bool) {
            if mid {
                exhelper.cfg.eval_params_mid.psqt = *self;
            } else {
                exhelper.cfg.eval_params_end.psqt = *self;
            }
        }
    }

}

#[cfg(feature = "nope")]
pub mod ep_index {
    use super::*;


    #[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy,Serialize,Deserialize,EPIndex)]
    pub enum EPIndex {
        PawnSupported,
        #[epindex_max(7)]
        PawnConnectedRank(u8),
        PawnBlockedR5,
        PawnBlockedR6,
        PawnDoubled,
        PawnIsolated,
        PawnBackward,
        OutpostKnight,
        OutpostBishop,
        OutpostReachableKnight,
        PPRookOpenFile,
        PPRookHalfOpenFile,
        #[epindex_max(64)]
        PSQTPawn(u8),
        #[epindex_max(64)]
        PSQTKnight(u8),
        #[epindex_max(64)]
        PSQTBishop(u8),
        #[epindex_max(64)]
        PSQTRook(u8),
        #[epindex_max(64)]
        PSQTQueen(u8),
        #[epindex_max(64)]
        PSQTKing(u8),
    }

    impl EvalParams {

        pub fn new() -> Self {
            let mut out = Self::default();
            out
        }

        // pub fn idx_piece<T: Into<Coord>>(pc: Piece, side: Color, c0: T) -> EPIndex {
        pub fn idx_piece(pc: Piece, side: Color, c0: u8) -> EPIndex {
            // let c1: Coord = c0.into();
            // let c1 = if side == White { c1 } else { Coord(c1.0,7 - c1.1) };
            // let c2 = u8::from(c1);

            let c2 = if side == White { c0 } else { Coord::flip_vertical_int(c0) };
            match pc {
                Pawn   => EPIndex::PSQTPawn(c2),
                Knight => EPIndex::PSQTKnight(c2),
                Bishop => EPIndex::PSQTBishop(c2),
                Rook   => EPIndex::PSQTRook(c2),
                Queen  => EPIndex::PSQTQueen(c2),
                King   => EPIndex::PSQTKing(c2),
            }
        }
        // #[inline(never)]
        // pub fn get_psqt<T: Into<Coord>>(&self, pc: Piece, side: Color, c0: T) -> Score {
        pub fn get_psqt(&self, pc: Piece, side: Color, c0: u8) -> Score {
            let idx = Self::idx_piece(pc, side, c0);
            self[idx]
        }
    }

}

#[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy,Serialize,Deserialize)]
pub struct EvalParams {
    pub mid:       bool,
    pub pawns:     EPPawns,
    pub pieces:    EPPieces,
    pub psqt:      PcTables,
}

impl EvalParams {

    pub fn new_mid_end() -> (Self,Self) {
        let mut ev_mid = Self::default();
        let mut ev_end = Self::default();
        ev_mid.mid = true;
        ev_end.mid = false;
        (ev_mid,ev_end)
    }

    pub fn empty() -> Self {
        let mut out = Self::default();
        let mut arr_mut = out.to_arr_mut();
        for v in arr_mut {
            *v = 0;
        }
        out
    }

    // pub fn save_evparams_bak<P: AsRef<Path> + AsRef<std::ffi::OsStr>>(
    //     ev_mid: &Self,
    //     ev_end: &Self,
    //     path:   P,
    // ) -> std::io::Result<()> {
    //     use std::io::Write;
    //     if std::path::Path::new(&path).exists() {
    //         std::fs::rename(&path, &format!("{}", path))?;
    //     }
    //     let b: Vec<u8> = bincode::serialize(&(ev_mid,ev_end)).unwrap();
    //     let mut file = std::fs::File::create(path)?;
    //     file.write_all(&b)
    // }

    pub fn save_evparams<P: AsRef<Path>>(ev_mid: &Self, ev_end: &Self, path: P) -> std::io::Result<()> {
        use std::io::Write;
        let b: Vec<u8> = bincode::serialize(&(ev_mid,ev_end)).unwrap();
        let mut file = std::fs::File::create(path)?;
        file.write_all(&b)
    }

    pub fn read_evparams<P: AsRef<Path>>(path: P) -> std::io::Result<(Self,Self)> {
        use std::io::Write;
        let mut b = std::fs::read(path)?;
        let out = bincode::deserialize(&b).unwrap();
        Ok(out)
    }
}

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy,Serialize,Deserialize,new)]
pub struct EPPieces {
    pub rook_open_file:  [Score; 2],
    pub outpost:         EvOutpost,
}

impl Default for EPPieces {
    fn default() -> Self {
        Self {
            rook_open_file:   [10,20],
            outpost:          EvOutpost::default(),
        }
    }
}

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy,Serialize,Deserialize,new)]
pub struct EvOutpost {
    pub outpost_knight:     Score,
    pub outpost_bishop:     Score,
    pub reachable_knight:   Score,
    // pub reachable_bishop:   Score,
}

impl Default for EvOutpost {
    fn default() -> Self {
        Self {
            // Self::new(50,30,30,0)
            outpost_knight:     50,
            outpost_bishop:     30,
            reachable_knight:   30,
            // reachable_bishop:   0,
        }
    }
}

// #[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy,Serialize,Deserialize,new,EvalIndex)]
#[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy,Serialize,Deserialize,new)]
pub struct EPPawns {
    pub supported:            Score,
    pub connected_ranks:      [Score; 7],

    pub blocked_r5:           Score,
    pub blocked_r6:           Score,

    // pub candidate:            Score,
    // pub passed:               Score,

    // pub doubled_isolated:     Score,
    pub doubled:              Score,
    pub isolated:             Score,
    pub backward:             Score,

}

// unsafe impl std::slice::SliceIndex for EPPawns {
// }

impl Default for EPPawns {
    fn default() -> Self {
        Self {
            supported:            20,
            connected_ranks:      [0, 5, 10, 15, 30, 50, 80],

            blocked_r5:           -10,
            blocked_r6:           -5,

            // candidate:            Score,
            // passed:               Score,

            // doubled_isolated:     10,
            isolated:             -5,
            backward:             -10,
            doubled:              -10,
        }
    }
}

// /// Passed bonus = passed * ranks past 2nd
// // #[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
// #[derive(Serialize,Deserialize,Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
// pub struct EvPawn {
//     pub backward: TaperedScore,
//     pub doubled:  TaperedScore,
//     pub isolated: TaperedScore,
//     pub passed:   TaperedScore,
// }

// // #[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
// #[derive(Serialize,Deserialize,Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
// pub struct EvRook {
//     pub rank7: TaperedScore,
// }

// // #[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
// #[derive(Serialize,Deserialize,Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
// pub struct EvKnight {
// }

// // #[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
// #[derive(Serialize,Deserialize,Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
// pub struct EvBishop {
// }

// // #[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
// #[derive(Serialize,Deserialize,Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
// pub struct EvQueen {
// }

// // #[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
// #[derive(Serialize,Deserialize,Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
// pub struct EvKing {
// }

// impl EvPawn {
//     pub fn new() -> Self {
//         Self {
//             backward: TaperedScore::new(-10, -10),
//             doubled:  TaperedScore::new(-15, -15),
//             isolated: TaperedScore::new(-20, -20),
//             passed:   TaperedScore::new(5,   10),
//         }
//     }
// }

// impl EvRook {
//     pub fn new() -> Self {
//         Self {
//             rank7: TaperedScore::new(20,40),
//         }
//     }
// }

// impl EvKnight {
//     pub fn new() -> Self {
//         Self {
//         }
//     }
// }

// impl EvBishop {
//     pub fn new() -> Self {
//         Self {
//         }
//     }
// }

// impl EvQueen {
//     pub fn new() -> Self {
//         Self {
//         }
//     }
// }

// impl EvKing {
//     pub fn new() -> Self {
//         Self {
//         }
//     }
// }

pub mod indexing {
    use super::*;

    impl Tunable for EvalParams {

        const LEN: usize = 1
            + EPPawns::LEN
            + EPPieces::LEN
            + PcTables::LEN;

        fn to_arr(&self) -> Vec<Score> {
            let mut out = vec![if self.mid { 1 } else { 0 }];
            out.extend_from_slice(&self.pawns.to_arr());
            out.extend_from_slice(&self.pieces.to_arr());
            out.extend_from_slice(&self.psqt.to_arr());
            out
        }
        fn from_arr(v: &[Score]) -> Self {
            let n0 = 1 + EPPawns::LEN;
            let n1 = n0 + EPPieces::LEN;
            let n2 = n1 + PcTables::LEN;
            Self {
                mid:     v[0] == 1,
                pawns:   EPPawns::from_arr(&v[1..n0]),
                pieces:  EPPieces::from_arr(&v[n0..n1]),
                // psqt:    PcTables::from_arr(&v[n1..n2 + 1]),
                psqt:    PcTables::from_arr(&v[n1..]),
            }
        }
        fn to_arr_mut(&mut self) -> Vec<&mut Score> {
            let mut xs = vec![];
            xs.extend(self.pawns.to_arr_mut());
            xs.extend(self.pieces.to_arr_mut());
            xs.extend(self.psqt.to_arr_mut());
            xs
        }
        fn update_exhelper(&self, exhelper: &mut ExHelper, mid: bool) {
            if mid {
                exhelper.cfg.eval_params_mid = *self;
            } else {
                exhelper.cfg.eval_params_end = *self;
            }
        }
    }

    impl Tunable for EPPawns {
        const LEN: usize = 13;

        fn to_arr(&self) -> Vec<Score> {
            let mut out = vec![self.supported];
            out.extend_from_slice(&self.connected_ranks);
            out.push(self.blocked_r5);
            out.push(self.blocked_r6);
            out.push(self.isolated);
            out.push(self.backward);
            out.push(self.doubled);
            out
        }

        fn from_arr(v: &[Score]) -> Self {
            assert!(v.len() >= Self::LEN);
            let mut out = Self::default();
            out.supported = v[0];
            out.connected_ranks.copy_from_slice(&v[1..8]);

            out.blocked_r5 = v[8];
            out.blocked_r6 = v[9];

            out.isolated = v[10];
            out.backward = v[11];
            out.doubled  = v[12];

            out
        }

        fn to_arr_mut(&mut self) -> Vec<&mut Score> {
            let mut xs = vec![&mut self.supported];
            xs.extend(self.connected_ranks.iter_mut());
            xs.push(&mut self.blocked_r5);
            xs.push(&mut self.blocked_r6);
            xs.push(&mut self.doubled);
            xs.push(&mut self.isolated);
            xs.push(&mut self.backward);
            xs
        }

        fn update_exhelper(&self, exhelper: &mut ExHelper, mid: bool) {
            if mid {
                exhelper.cfg.eval_params_mid.pawns = *self;
            } else {
                exhelper.cfg.eval_params_end.pawns = *self;
            }
        }
    }

    impl Tunable for EPPieces {
        const LEN: usize = 2 + EvOutpost::LEN;

        fn to_arr(&self) -> Vec<Score> {
            let mut out = vec![
                self.rook_open_file[0],
                self.rook_open_file[1],
            ];
            out.extend_from_slice(&self.outpost.to_arr());
            out
        }

        fn from_arr(v: &[Score]) -> Self {
            assert!(v.len() >= Self::LEN);
            Self {
                rook_open_file: [v[0],v[1]],
                outpost:        EvOutpost::from_arr(&v[2..]),
            }
        }

        fn to_arr_mut(&mut self) -> Vec<&mut Score> {
            let mut xs = vec![];
            xs.extend(self.rook_open_file.iter_mut());
            xs.extend(self.outpost.to_arr_mut());
            xs
        }

        fn update_exhelper(&self, exhelper: &mut ExHelper, mid: bool) {
            if mid {
                exhelper.cfg.eval_params_mid.pieces = *self;
            } else {
                exhelper.cfg.eval_params_end.pieces = *self;
            }
        }
    }

    impl Tunable for EvOutpost {
        const LEN: usize = 3;

        fn to_arr(&self) -> Vec<Score> {
            vec![self.outpost_knight,self.outpost_bishop,self.reachable_knight]
        }

        fn from_arr(v: &[Score]) -> Self {
            assert!(v.len() >= 3);
            Self {
                outpost_knight: v[0],
                outpost_bishop: v[1],
                reachable_knight: v[2],
            }
        }

        fn to_arr_mut(&mut self) -> Vec<&mut Score> {
            let mut xs = vec![
                &mut self.outpost_knight,
                &mut self.outpost_bishop,
                &mut self.reachable_knight,
            ];
            xs
        }

        fn update_exhelper(&self, exhelper: &mut ExHelper, mid: bool) {
            if mid {
                exhelper.cfg.eval_params_mid.pieces.outpost = *self;
            } else {
                exhelper.cfg.eval_params_end.pieces.outpost = *self;
            }
        }
    }

}

