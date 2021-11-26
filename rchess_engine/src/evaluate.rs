
use crate::types::*;
use crate::tables::*;

pub use self::tapered::TaperedScore;

use nalgebra::Quaternion;
use serde::{Serialize,Deserialize};

pub type Score = i32;

pub static CHECKMATE_VALUE: Score = 100_000_000;
pub static STALEMATE_VALUE: Score = 20_000_000;

pub fn convert_from_score(s: Score) -> i8 {
    const K: Score = 16909320;
    (s / K) as i8
}

mod tapered {
    use crate::types::*;
    use crate::tables::*;
    use crate::evaluate::*;

    #[derive(Serialize,Deserialize,Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
    pub struct TaperedScore {
        mid: Score,
        end: Score,
    }

    impl TaperedScore {
        pub const fn new(mid: Score, end: Score) -> Self {
            Self {
                mid,
                end,
            }
        }

        // pub fn taper(&self, phase: Phase) -> Score {
        //     ((self.mid * (256 - phase as Score)) + (self.end * phase as Score)) / 256
        // }

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

#[derive(Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct Eval {
    pub phase:            Phase,
    pub material:         [[Score; 6]; 2],
    pub piece_positions:  [[Score; 6]; 2],
}

impl Eval {

    fn sum(&self) -> Score {

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

/// Main Evaluation
impl Game {

    fn sum_evaluate2(&self, ts: &Tables) -> Score {
        self.evaluate(ts).sum()
    }

    fn evaluate(&self, ts: &Tables) -> Eval {
        let mut out = Eval::default();

        let phase = self.state.phase;

        for &col in [White,Black].iter() {
            for pc in Piece::iter_pieces() {

                let mat = self.score_material(pc, col);
                    // .taper(phase);
                out.set_piece_mat_mut(pc, col, mat);

                #[cfg(feature = "positional_scoring")]
                {
                    let pos = self.score_positions(&ts, pc, col);
                        // .taper(phase);
                    out.set_piece_pos_mut(pc, col, pos);
                }

                // eprintln!("scoring = {:?} {:?}: {:?}/{:?}", col, pc, m, pos);
            }
        }

        out.phase = phase;

        out
    }

    fn taper_score(&self, mid: Score, end: Score) -> Score {
        let phase = self.state.phase as Score;
        ((mid * (256 - phase)) + (end * phase)) / 256
    }

}

/// Main Eval 2
impl Game {

    pub fn sum_evaluate(&self, ts: &Tables) -> Score {
        // const SIDES: [Color; 2] = [White,Black];
        // let side = self.state.side_to_move;

        let mg = self.sum_evaluate_mg(ts);
        let eg = self.sum_evaluate_eg(ts);

        self.taper_score(mg, eg)
    }

    fn sum_evaluate_mg(&self, ts: &Tables) -> Score {
        let mut score = 0;

        score += self.score_material2(White) - self.score_material2(Black);
        score += self.score_psqt(ts, White) - self.score_psqt(ts, Black);
        score += self.score_mobility(ts, White) - self.score_mobility(ts, Black);
        score += self.score_pieces_mg(ts, White) - self.score_pieces_mg(ts, Black);

        score
    }

    fn sum_evaluate_eg(&self, ts: &Tables) -> Score {
        let mut score = 0;

        score += self.score_material2(White) - self.score_material2(Black);
        score += self.score_psqt(ts, White) - self.score_psqt(ts, Black);
        score += self.score_mobility(ts, White) - self.score_mobility(ts, Black);
        score += self.score_pieces_eg(ts, White) - self.score_pieces_eg(ts, Black);

        score
    }

    fn score_material2(&self, side: Color) -> Score {
        const PCS: [Piece; 6] = [Pawn,Knight,Bishop,Rook,Queen,King];
        PCS.iter().map(|&pc| self.state.material.get(pc, side) as Score * pc.score()).sum()
    }

    fn score_psqt(&self, ts: &Tables, side: Color) -> Score {
        const PCS: [Piece; 6] = [Pawn,Knight,Bishop,Rook,Queen,King];
        PCS.iter().map(|&pc| {
            self._score_psqt(ts, pc, side)
        }).sum()
    }

    fn _score_psqt(&self, ts: &Tables, pc: Piece, side: Color) -> Score {
        let pieces = self.get(pc, side);
        let score_mg = pieces.into_iter().map(|sq| {
            ts.piece_tables.get_mid(pc, side, sq)
        }).sum();
        let score_eg = pieces.into_iter().map(|sq| {
            ts.piece_tables.get_end(pc, side, sq)
        }).sum();
        self.taper_score(score_mg, score_eg)
    }

    fn score_pieces_mg(&self, ts: &Tables, side: Color) -> Score {

        let rook_files = self.rook_on_open_file(side);
        let outposts   = self.outpost_squares(side);

        // unimplemented!()
        0
    }

    fn score_pieces_eg(&self, ts: &Tables, side: Color) -> Score {
        // unimplemented!()
        0
    }

}

/// Phase
impl Game {

    // pub fn update_phase(&self, new_piece: Option<Piece>, delete_piece: Option<>)
    #[must_use]
    pub fn update_phase(&self, mv: Move) -> Phase {
        let mut ph = self.state.phase;
        if let Some(victim) = mv.victim() {
        }
        unimplemented!()
    }

    // pub fn increment_phase(&self, mv: Move) -> u8 {
    //     match mv {
    //         Move::Promotion { new_piece, .. }        => unimplemented!(),
    //         Move::PromotionCapture { new_piece, .. } => unimplemented!(),
    //         _                                        => {},
    //     }
    //     match mv.victim() {
    //         None         => self.state.phase,
    //         Some(victim) => {
    //             self.state.phase - victim.score()
    //             unimplemented!()
    //         },
    //     }
    // }

    pub fn game_phase(&self) -> u8 {
        const MIDGAME_LIMIT: i32 = 15258;
        const ENDGAME_LIMIT: i32 = 3915;

        const NON_PAWN: [Piece; 4] = [Knight,Bishop,Rook,Queen];

        let side = self.state.side_to_move;

        let npm: Score = NON_PAWN.iter().map(|&pc| {
            self.state.material.get(pc, White) as Score * pc.score()
                + self.state.material.get(pc, Black) as Score * pc.score()
        }).sum();

        let npm = Score::max(ENDGAME_LIMIT, Score::min(npm, MIDGAME_LIMIT));

        let out = (((npm - ENDGAME_LIMIT) * 128) / (MIDGAME_LIMIT - ENDGAME_LIMIT)) << 0;

        let out = convert_from_score(out) as i16 + 127;

        out as u8
    }

    pub fn game_phase2(&self) -> u8 {
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
                let x = self.state.material.buf[col][pc.index()] as u16 * PHASES[pc.index()];
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

    pub fn score_material(&self, pc: Piece, side: Color) -> Score {
        match pc {
            King   => {
                let s = King.score();
                s
            }
            Rook   => {
                let rs = self.get(Rook, side);

                // let n = rs.popcount() as i32;
                let n = self.state.material.buf[side][Rook.index()] as i32;
                let s = Rook.score() * n;
                s
            },
            // Knight => {},
            Bishop => {
                // let n = self.get(Bishop, side).popcount() as i32;
                let n = self.state.material.buf[side][Bishop.index()] as i32;
                let s = if n > 1 {
                    // 2 bishops = 0.5 pawn
                    Bishop.score() * n + 50
                } else {
                    Bishop.score() * n
                };
                s
            },
            _      => {
                // let s = pc.score() * self.get(pc, side).popcount() as i32;
                let n = self.state.material.buf[side][pc.index()] as i32;
                let s = pc.score() * n;
                s
            },
        }
    }

    // pub fn score_material(&self, pc: Piece, side: Color) -> TaperedScore {
    //     match pc {
    //         King   => {
    //             let s = King.score();
    //             TaperedScore::new(s, s)
    //         }
    //         Rook   => {
    //             let rs = self.get(Rook, side);
    //             // let n = rs.popcount() as i32;
    //             let n = self.state.material[side][Rook.index()] as i32;
    //             let s = Rook.score() * n;
    //             TaperedScore::new(s,s)
    //         },
    //         // Knight => {},
    //         Bishop => {
    //             // let n = self.get(Bishop, side).popcount() as i32;
    //             let n = self.state.material[side][Bishop.index()] as i32;
    //             let s = if n > 1 {
    //                 // 2 bishops = 0.5 pawn
    //                 Bishop.score() * n + 50
    //             } else {
    //                 Bishop.score() * n
    //             };
    //             TaperedScore::new(s,s)
    //         },
    //         _      => {
    //             // let s = pc.score() * self.get(pc, side).popcount() as i32;
    //             let n = self.state.material[side][pc.index()] as i32;
    //             let s = pc.score() * n;
    //             TaperedScore::new(s,s)
    //         },
    //     }
    // }

}

/// Positional Scoring
impl Game {

    // pub fn score_positions(&self, ts: &Tables, pc: Piece, col: Color) -> TaperedScore {
    pub fn score_positions(&self, ts: &Tables, pc: Piece, side: Color) -> Score {
        let mut pos = self._score_positions(&ts, pc, side);
        match pc {
            Pawn   => {
                // let ds = self.count_pawns_doubled(&ts, col);
                // let ds = ts.piece_tables.ev_pawn.doubled * ds as Score;
                // pos = pos + ds;
                pos
            },
            Rook   => {
                let rs = self.get(Rook, side);

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

    // fn _score_positions(&self, ts: &Tables, pc: Piece, col: Color) -> TaperedScore {
    fn _score_positions(&self, ts: &Tables, pc: Piece, side: Color) -> Score {
        let pieces = self.get(pc, side);
        let mut score_mg = 0;
        let mut score_eg = 0;
        pieces.into_iter().for_each(|sq| {
            score_mg += ts.piece_tables.get_mid(pc, side, sq);
            score_eg += ts.piece_tables.get_end(pc, side, sq);
        });
        self.taper_score(score_mg, score_eg)
        // TaperedScore::new(score_mg,score_eg)
    }

}

/// Outposts
impl Game {

    pub fn outpost_squares(&self, side: Color) -> Score {
        let n = self.get(Knight, side).into_iter()
            .filter(|&sq| self.outpost_square(sq.into(), side)).count() as Score;
        let b = self.get(Bishop, side).into_iter()
            .filter(|&sq| self.outpost_square(sq.into(), side)).count() as Score;
        n + b
    }

    pub fn outpost_square(&self, c0: Coord, side: Color) -> bool {
        if c0.rank() < 4 || c0.rank() > 6 { return false; }

        let (dw,de) = if side == White { (NW,NE) } else { (SW,SE) };

        let b0 = BitBoard::single(c0);
        if (self.get(Pawn, side) & (b0.shift_dir(!dw) | b0.shift_dir(!de))).is_empty() {
            false
        } else {
            let b1 = b0.shift_dir(dw) | b0.shift_dir(de);
            let b2 = if side == White { b1.fill_north() } else { b1.fill_south() };

            (b2 & self.get(Pawn, !side)).is_empty()
        }
    }

}

/// Misc Positional
impl Game {

    /// Open      = no pawns = 2
    /// Half open = only enemy pawns = 1
    pub fn rook_on_open_file(&self, side: Color) -> Score {
        let pawns_own   = self.get(Pawn, side);
        let pawns_other = self.get(Pawn, !side);

        self.get(Rook, side).into_iter().map(|sq| {
            let c0 = Coord::from(sq);
            let file = c0.file();
            if (BitBoard::mask_file(file) & pawns_own).is_not_empty() {
                0
            } else if (BitBoard::mask_file(file) & pawns_other).is_not_empty() {
                1
            } else {
                2
            }
        }).sum()
    }

}

/// Mobility
impl Game {

    pub fn score_mobility(&self, ts: &Tables, side: Color) -> Score {
        const PCS: [Piece; 4] = [Knight,Bishop,Rook,Queen];
        let mob = self.mobility_area(ts, side);

        PCS.iter().map(|&pc| {
            self.get(pc, side).into_iter().map(|sq| {
                let c0: Coord = sq.into();
                self._score_mobility(ts, mob, pc, c0, side)
            }).sum::<Score>()
        }).sum::<Score>()
    }

    pub fn _score_mobility(&self, ts: &Tables, mob: BitBoard, pc: Piece, c0: Coord, side: Color) -> Score {
        match pc {
            Knight => {
                let mvs = ts.get_knight(c0);
                (*mvs & mob).popcount() as Score
            },
            Bishop => {
                let occ = self.all_occupied() & !self.get(Queen, side);
                let mvs = ts.attacks_bishop(c0, occ);
                (mvs & mob).popcount() as Score
            },
            Rook   => {
                let occ = self.all_occupied() & !self.get(Queen, side);
                let mvs = ts.attacks_rook(c0, occ);
                (mvs & mob).popcount() as Score
            },
            Queen  => {
                let mvs0 = ts.attacks_bishop(c0, self.all_occupied());
                let mvs1 = ts.attacks_rook(c0, self.all_occupied());
                ((mvs0 | mvs1) & mob).popcount() as Score
            },

            Pawn   => unimplemented!(),
            King   => unimplemented!(),
        }

    }

    pub fn mobility_area(&self, ts: &Tables, side: Color) -> BitBoard {
        const WHITE_R23: BitBoard = BitBoard(0x0000000000ffff00);
        const BLACK_R67: BitBoard = BitBoard(0x00ffff0000000000);

        let mut mob = !BitBoard::empty();

        mob &= !self.get(King, side);
        mob &= !self.get(Queen, side);
        mob &= !(self.get(Pawn, side) & (if side == White { WHITE_R23 } else { BLACK_R67 }));
        mob &= !self.get_pins(side);

        let (d,dw,de) = if side == White { (S,SW,SE) } else { (N,NW,NE) };

        let enemy_pawns = self.get(Pawn, !side);
        // Enemy pawn attacks
        mob &= !(enemy_pawns.shift_dir(dw) | enemy_pawns.shift_dir(de));

        // Blocked pawns
        mob &= !(self.get(Pawn, side) & enemy_pawns.shift_dir(d));

        mob
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

/// Pawn Spans
impl Game {

    // pub fn pawn_attacks_span(&self, side: Color) -> BitBoard {
    //     let (d,dw,de) = if side == White { (S,SW,SE) } else { (N,NW,NE) };
    //     let pawns = self.get(Pawn, !side);
    //     pawns.shift_dir(dw) | pawns.shift_dir(de)
    // }

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

    pub fn count_pawns_backward(&self, ts: &Tables, side: Color) -> u8 {
        unimplemented!()
    }

    pub fn count_pawns_isolated(&self, ts: &Tables, side: Color) -> u8 {
        unimplemented!()
    }

    pub fn count_pawns_passed(&self, ts: &Tables, side: Color) -> u8 {
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


