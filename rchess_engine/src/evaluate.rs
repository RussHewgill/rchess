
use crate::types::*;
use crate::tables::*;

use serde::{Serialize,Deserialize};

pub type Score = i32;

pub static CHECKMATE_VALUE: Score = 100_000_000;
pub static STALEMATE_VALUE: Score = 20_000_000;

#[derive(Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct Eval {
    pub phase:            Phase,
    pub material:         [[Score; 6]; 2],
    pub piece_positions:  [[Score; 6]; 2],
}

// #[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
#[derive(Serialize,Deserialize,Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct TaperedScore {
    mid: Score,
    end: Score,
}

impl Eval {

    pub fn sum(&self) -> Score {

        let white = self.sum_color(White);
        let black = self.sum_color(Black);

        // if col == White {
        //     white - black
        // } else {
        //     black - white
        // }

        // white - black
        white + black

        // unimplemented!("Eval::diff()")
    }

    fn sum_material(&self) -> [Score; 2] {
        let w = self.material[White].iter().sum();
        let b = self.material[Black].iter().sum();
        [w,b]
    }

    fn sum_positions(&self) -> [Score; 2] {
        let w = self.piece_positions[White].iter().sum();
        let b = self.piece_positions[Black].iter().sum();
        [w,b]
    }

    fn sum_color(&self, col: Color) -> Score {
        let mut score = 0;
        match col {
            White => {
                for m in self.material[White].iter() {
                    score += m;
                }
                for m in self.piece_positions[White].iter() {
                    score += m;
                }
            },
            Black => {
                for m in self.material[Black].iter() {
                    score -= m;
                }
                for m in self.piece_positions[Black].iter() {
                    score -= m;
                }
            },
        }
        score
    }

    pub fn get_piece_pos(&self, pc: Piece, col: Color) -> Score {
        self.piece_positions[col][pc.index()]
    }
    pub fn set_piece_pos_mut(&mut self, pc: Piece, col: Color, s: Score) {
        self.piece_positions[col][pc.index()] = s
    }
    pub fn get_piece_mat(&self, pc: Piece, col: Color) -> Score {
        self.material[col][pc.index()]
    }
    pub fn set_piece_mat_mut(&mut self, pc: Piece, col: Color, s: Score) {
        self.material[col][pc.index()] = s
    }

}

/// Attack and defend maps
impl Game {



}

/// Main Evaluation
impl Game {

    pub fn evaluate(&self, ts: &Tables) -> Eval {
        let mut out = Eval::default();

        let phase = self.state.phase;

        for &col in [White,Black].iter() {
            for pc in Piece::iter_pieces() {

                let mat = self.score_material(pc, col)
                    .taper(phase);
                out.set_piece_mat_mut(pc, col, mat);

                #[cfg(feature = "positional_scoring")]
                {
                    let pos = self.score_positions(&ts, pc, col)
                        .taper(phase);
                    out.set_piece_pos_mut(pc, col, pos);
                }

                // eprintln!("scoring = {:?} {:?}: {:?}/{:?}", col, pc, m, pos);
            }
        }

        out.phase = phase;

        out
    }

    pub fn non_pawn_material(&self) -> (Score,Score) {
        unimplemented!()
    }

    // pub fn update_phase(&self, new_piece: Option<Piece>, delete_piece: Option<>)
    #[must_use]
    pub fn update_phase(&self, mv: Move) -> Phase {
        let mut ph = self.state.phase;

        unimplemented!()
    }

    pub fn game_phase(&self) -> u8 {
        const PAWN_PH: u16   = 0;
        const KNIGHT_PH: u16 = 1;
        const BISHOP_PH: u16 = 1;
        const ROOK_PH: u16   = 2;
        const QUEEN_PH: u16  = 4;

        const PCS: [Piece; 5] = [Pawn,Knight,Bishop,Rook,Queen];
        const PHASES: [u16; 5] = [PAWN_PH,KNIGHT_PH,BISHOP_PH,ROOK_PH,QUEEN_PH];

        let ph_total = PAWN_PH * 16 + KNIGHT_PH * 4 + BISHOP_PH * 4 + ROOK_PH * 4 + QUEEN_PH * 2;
        let mut ph = ph_total;

        for &col in [White,Black].iter() {
            for pc in PCS {
                let x = self.state.material[col][pc.index()] as u16 * PHASES[pc.index()];
                if ph < x {
                    ph = 0;
                    break;
                }
                ph -= x;
            }
        }

        // for &col in [White,Black].iter() {
        //     for pc in PCS {
        //         let ps = self.get(pc, col);
        //         let pn = ps.popcount() as u16;
        //         let x = pn * PHASES[pc.index()];
        //         if ph < x {
        //             ph = 0;
        //             break;
        //         }
        //         ph -= x;
        //     }
        // }

        // eprintln!("ph_total = {:?}", ph_total);
        // eprintln!("ph = {:?}", ph);

        let phase = (ph * 256 + (ph_total / 2)) / ph_total;

        phase.min(255) as u8
    }

}

impl Piece {
    pub fn score_basic(&self) -> i32 {
        match self {
            Pawn   => 100,
            Knight => 300,
            Bishop => 300,
            Rook   => 500,
            Queen  => 900,
            King   => 1000000,
        }
    }
    pub fn score(&self) -> i32 {
        match self {
            Pawn   => 100,
            Knight => 320,
            Bishop => 330,
            Rook   => 500,
            Queen  => 900,
            King   => 1000000,
        }
    }
}

/// Material Scoring
impl Game {

    pub fn score_material(&self, pc: Piece, col: Color) -> TaperedScore {

        match pc {
            Rook   => {
                let rs = self.get(Rook, col);

                let n = rs.popcount() as i32;
                let s = Rook.score() * n;
                TaperedScore::new(s,s)
            },
            // Knight => {},
            Bishop => {
                let n = self.get(Bishop, col).popcount() as i32;
                let s = if n > 1 {
                    // 2 bishops = 0.5 pawn
                    Bishop.score() * n + 50
                } else {
                    Bishop.score() * n
                };
                TaperedScore::new(s,s)
            },
            _      => {
                let s = pc.score() * self.get(pc, col).popcount() as i32;
                TaperedScore::new(s,s)
            },
        }
    }

}

/// Positional Scoring
impl Game {

    pub fn score_positions(&self, ts: &Tables, pc: Piece, col: Color) -> TaperedScore {
        let mut pos = self._score_positions(&ts, pc, col);
        match pc {
            Pawn   => {
                // let ds = self.count_pawns_doubled(&ts, col);
                // let ds = ts.piece_tables.ev_pawn.doubled * ds as Score;
                // pos = pos + ds;
                pos
            },
            Rook   => {
                let rs = self.get(Rook, col);

                // let r_7r = if col == White {
                //     rs & BitBoard::mask_rank(6)
                // } else {
                //     rs & BitBoard::mask_rank(1)
                // };
                // let r_7r = r_7r.popcount();

                // let r_7r = ts.pie

                pos
            }
            // Knight => unimplemented!(),
            // Bishop => unimplemented!(),
            // Queen  => unimplemented!(),
            King   => {
                // let safety = self.king_safety(&ts, col);
                // pos + TaperedScore::new(safety,safety)
                pos
            },
            _      => pos,
        }
    }

    fn _score_positions(&self, ts: &Tables, pc: Piece, col: Color) -> TaperedScore {
        let pieces = self.get(pc, col);
        let mut score_mg = 0;
        let mut score_eg = 0;
        pieces.into_iter().for_each(|sq| {
            score_mg += ts.piece_tables.get_mid(pc, col, sq);
            score_eg += ts.piece_tables.get_end(pc, col, sq);
        });
        TaperedScore::new(score_mg,score_eg)
    }

}

/// King Safety
impl Game {
    fn king_safety(&self, ts: &Tables, side: Color) -> Score {

        let king = self.get(King, side).bitscan();
        let sqs = ts.get_king(king);
        let pawns = self.get(Pawn, side) & *sqs;
        let pawn_shield = pawns.popcount() as Score;

        unimplemented!()
    }
}

/// Pawn Structure
impl Game {

    pub fn count_pawns_doubled(&self, ts: &Tables, col: Color) -> u8 {
        let pawns = self.get(Pawn, col);

        let out = (0..8).map(|f| {
            let b = pawns & BitBoard::mask_file(f);

            // eprintln!("b = {:?}", b);

            match b.popcount() {
                0 | 1 => 0,
                2     => 1,
                3     => 2,
                4     => 3,
                5     => 4,
                6     => 5,
                _     => panic!("too many pawns in file")
            }
        // }).collect::<Vec<_>>();
        });

        // eprintln!("out = {:?}", out);

        // let coords = pawns.into_iter()
        //     .map(|sq| Coord::from(sq))
        //     .map(|c| c.0)
        //     .collect::<Vec<_>>();
        // let mut cs = coords.clone();
        // cs.sort();
        // cs.dedup();

        let out: u32 = out.sum();
        out as u8
        // out.len() as u8
        // unimplemented!()
    }

    pub fn count_pawns_backward(&self, ts: &Tables, col: Color) -> u8 {
        unimplemented!()
    }

    pub fn count_pawns_isolated(&self, ts: &Tables, col: Color) -> u8 {
        unimplemented!()
    }

    pub fn count_pawns_passed(&self, ts: &Tables, col: Color) -> u8 {
        unimplemented!()
    }

}

impl std::fmt::Debug for Eval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [m_w,m_b] = self.sum_material();
        let [p_w,p_b] = self.sum_positions();
        let m_w = m_w - King.score();
        let m_b = m_b - King.score();
        f.write_str(&format!("ph: {}, mat: ({},{}), pos: ({},{})",
                             self.phase, m_w, m_b, p_w, p_b,))?;
        Ok(())
    }
}

mod tapered {
    use crate::types::*;
    use crate::tables::*;
    use crate::evaluate::*;

    impl TaperedScore {
        pub const fn new(mid: Score, end: Score) -> Self {
            Self {
                mid,
                end,
            }
        }

        pub fn taper(&self, phase: Phase) -> Score {
            ((self.mid * (256 - phase as Score)) + (self.end * phase as Score)) / 256
        }

    }

    impl std::ops::Add for TaperedScore {
        type Output = Self;
        fn add(self, other: Self) -> Self {
            Self {
                mid: self.mid + other.mid,
                end: self.end + other.end,
            }
        }
    }

    impl std::ops::Mul<Score> for TaperedScore {
        type Output = Self;
        fn mul(self, x: Score) -> Self {
            Self {
                mid: self.mid * x,
                end: self.end * x,
            }
        }
    }

}

