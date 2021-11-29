
use crate::evaluate::TaperedScore;
use crate::types::*;
// use crate::tables::*;
use crate::evaluate::*;

pub use self::piece_square_tables::*;

// use rchess_macros::EvalIndex;

use serde::{Serialize,Deserialize};
use derive_new::new;

pub static LMR_MIN_MOVES: Depth = 2;
pub static LMR_MIN_PLY: Depth = 3;
pub static LMR_MIN_DEPTH: Depth = 3;

pub static LMR_REDUCTION: Depth = 3;
pub static LMR_PLY_CONST: Depth = 6;

pub static QS_RECAPS_ONLY: Depth = 5;
// pub static QS_RECAPS_ONLY: Depth = 100;

pub static NULL_PRUNE_MIN_DEPTH: Depth = 2;

pub trait Tunable {
    const LEN: usize;
    fn to_arr(&self) -> Vec<Score>;
    fn from_arr(v: &[Score]) -> Self;
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
        pawn:       [Score; 64],
        #[serde(with = "BigArray")]
        knight:     [Score; 64],
        #[serde(with = "BigArray")]
        bishop:     [Score; 64],
        #[serde(with = "BigArray")]
        rook:       [Score; 64],
        #[serde(with = "BigArray")]
        queen:      [Score; 64],
        #[serde(with = "BigArray")]
        king:       [Score; 64],
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
        }
    }

    impl PcTables {

        pub fn print_table(ss: [Score; 64]) {
            for y in 0..8 {
                let y = 7 - y;
                for x in 0..8 {
                    // println!("(x,y) = ({},{}), coord = {:?}", x, y, Coord(x,y));
                    // print!("{:>3?},", ps.get(Pawn, Coord(x,y)));
                    let s = ss[Coord(x,y)];
                    print!("{:>3?},", s);
                }
                println!();
            }
        }

        pub fn get<T: Into<Coord>>(&self, pc: Piece, col: Color, c0: T) -> Score {
            let c1: Coord = c0.into();
            let c1 = if col == White { c1 } else { Coord(c1.0,7 - c1.1) };
            self[pc][c1]
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

        #[allow(clippy::vec_init_then_push)]
        fn old_gen_pawns() -> [Score; 64] {
            // let mut out = [0; 64];
            let mut scores: Vec<(&str,Score)> = vec![];

            // Castles
            scores.push(("A2",5));
            scores.push(("B2",10));
            scores.push(("C2",10));

            // Castle holes
            scores.push(("A3",5));
            scores.push(("B3",-5));
            scores.push(("C3",-10));

            // King/Queen Pawns
            scores.push(("D2",-20));

            // Center pawns
            scores.push(("D4",20));

            // Rank 5 pawns
            scores.push(("A5",5));
            scores.push(("B5",5));
            scores.push(("C5",10));
            scores.push(("D5",25));

            // Rank 6 pawns
            scores.push(("A6",10));
            scores.push(("B6",10));
            scores.push(("C6",20));
            scores.push(("D6",30));

            // Rank 7 pawns
            scores.push(("A7",50));
            scores.push(("B7",50));
            scores.push(("C7",50));
            scores.push(("D7",50));

            let mut out = [0; 64];

            for (c,s) in scores.into_iter() {
                let c0: Coord = c.into();
                let sq: usize = c0.into();
                out[sq] = s;
                let c1 = Coord(7-c0.0,c0.1);
                let sq: usize = c1.into();
                out[sq] = s;
            }
            // Self::transform_arr(out)
            out
        }

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
                    out[Coord(x,7 - y)] = xs[Coord(x,y)];
                }
            }
            out
        }

    }
}

#[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy,Serialize,Deserialize)]
pub struct EvalParams {
    pub mid:       bool,
    pub pawns:     EPPawns,
    pub pieces:    EPPieces,
    #[serde(skip)]
    pub psqt:      PcTables,
}

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
            psqt:    PcTables::from_arr(&v[n1..n2 + 1]),
        }
    }
}

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy,Serialize,Deserialize,new)]
pub struct EPPieces {
    pub rook_open_file:  [Score; 2],
    pub outpost:         EvOutpost,
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

impl Tunable for EPPawns {
    const LEN: usize = 13;
    fn to_arr(&self) -> Vec<Score> {
        let mut out = vec![self.supported];
        out.extend_from_slice(&self.connected_ranks);
        out.push(self.blocked_r5);
        out.push(self.blocked_r6);
        out.push(self.doubled);
        out.push(self.isolated);
        out.push(self.backward);
        out
    }

    fn from_arr(v: &[Score]) -> Self {
        assert!(v.len() >= Self::LEN);
        let mut out = Self::default();
        out.supported = v[0];
        out.connected_ranks.copy_from_slice(&v[1..8]);

        out.blocked_r5 = v[8];
        out.blocked_r6 = v[9];

        out.doubled  = v[10];
        out.isolated = v[11];
        out.backward = v[12];

        out
    }
}

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
            isolated:             5,
            backward:             10,
            doubled:              10,
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

    // impl std::ops::Index<usize> for EvalParams {
    //     type Output = Score;
    //     fn index(&self, idx: usize) -> &Self::Output {
    //         match idx {
    //             0 => &self.a,
    //             _ => unimplemented!(),
    //         }
    //     }
    // }
    // impl std::ops::IndexMut<usize> for EvalParams {
    //     fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
    //         match idx {
    //             0 => &mut self.a,
    //             _ => unimplemented!(),
    //         }
    //     }
    // }

    // pub trait EvalIndex {}

    // use rchess_macros::evalindex_derive;

    // #[derive(EvalIndex)]
    // #[derive(evalindex_derive)]
    pub struct Wat {
        pub a:    Score,
        pub b:    Score,
    }

    impl std::ops::Index<usize> for Wat {
        type Output = i32;
        fn index(&self, idx: usize) -> &Self::Output {
            match idx {
                0usize => &self.a,
                1usize => &self.b,
                _ => unimplemented!(),
            }
        }
    }
    impl std::ops::IndexMut<usize> for Wat {
        fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
            match idx {
                0usize => &mut self.a,
                1usize => &mut self.b,
                _ => unimplemented!(),
            }
        }
    }


    // impl std::ops::IndexMut<usize> for Wat {
    //     fn index_mut(&mut self, pc: Piece) -> &mut Self::Output {
    //         &mut self[pc.index()]
    //     }
    // }

    // impl std::ops::Index<usize> for EPPawns {
    //     type Output = Score;
    //     fn index(&self, idx: usize) -> &Self::Output {
    //         match idx {
    //             0 => &self.supported,
    //             1 => &self.connected_ranks,
    //             2 => &self.reachable_knight,
    //             // 3 => &self.reachable_bishop,
    //             _ => unimplemented!()
    //         }
    //     }
    // }

    // impl std::ops::Index<usize> for EvOutpost {
    //     type Output = Score;
    //     fn index(&self, idx: usize) -> &Self::Output {
    //         match idx {
    //             0 => &self.outpost_knight,
    //             1 => &self.outpost_bishop,
    //             2 => &self.reachable_knight,
    //             // 3 => &self.reachable_bishop,
    //             _ => unimplemented!()
    //         }
    //     }
    // }

}

