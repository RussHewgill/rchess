
use crate::types::*;
use crate::tables::*;

pub type Score = i32;

#[derive(Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct Eval {
    pub phase:            u16,
    pub material:         [[Score; 6]; 2],
    pub piece_positions:  [[Score; 6]; 2],
}

#[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
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

/// Main Evaluation
impl Game {

    pub fn evaluate(&self, ts: &Tables) -> Eval {
        let mut out = Eval::default();

        let phase = self.game_phase();

        for &col in [White,Black].iter() {
            for pc in Piece::iter_pieces() {

                let mat = self.score_material(pc, col)
                    .taper(phase);
                out.set_piece_mat_mut(pc, col, mat);

                let pos = self.score_positions(&ts, pc, col)
                    .taper(phase);
                out.set_piece_pos_mut(pc, col, pos);

                // eprintln!("scoring = {:?} {:?}: {:?}/{:?}", col, pc, m, pos);
            }
        }

        out.phase = phase;

        out
    }

    pub fn game_phase(&self) -> u16 {
        let pawn_ph   = 0;
        let knight_ph = 1;
        let bishop_ph = 1;
        let rook_ph   = 2;
        let queen_ph  = 4;

        let pcs = [Pawn,Rook,Knight,Bishop,Queen];
        let phases: [u16; 5] = [pawn_ph,rook_ph,knight_ph,bishop_ph,queen_ph];

        let ph_total = pawn_ph * 16 + knight_ph * 4 + bishop_ph * 4 + rook_ph * 4 + queen_ph * 2;
        let mut ph = ph_total;

        for &col in [White,Black].iter() {
            for pc in pcs {
                let ps = self.get(pc, col);
                let pn = ps.popcount() as u16;
                ph -= pn * phases[pc.index()];
            }
        }

        // eprintln!("ph_total = {:?}", ph_total);
        // eprintln!("ph = {:?}", ph);

        let phase = (ph * 256 + (ph_total / 2)) / ph_total;

        phase
    }

}

impl Piece {
    pub fn score_basic(&self) -> i32 {
        match self {
            Pawn   => 100,
            Rook   => 500,
            Knight => 300,
            Bishop => 300,
            Queen  => 900,
            King   => 1000000,
        }
    }
    pub fn score(&self) -> i32 {
        match self {
            Pawn   => 100,
            Rook   => 500,
            Knight => 320,
            Bishop => 330,
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

                let r_7r = if col == White {
                    let b = rs & BitBoard::mask_rank(6);
                    // b.popcount() * 
                } else {
                    let b = rs & BitBoard::mask_rank(1);
                };

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
        match pc {
            Pawn   => {
                let mut pos = self._score_positions(&ts, Pawn, col);
                let ds = self.count_pawns_doubled(&ts, col);
                pos.mid += ds as Score * ts.piece_tables_midgame.ev_pawn.doubled;
                pos.end += ds as Score * ts.piece_tables_endgame.ev_pawn.doubled;
                pos
            },
            // Rook   => unimplemented!(),
            // Knight => unimplemented!(),
            // Bishop => unimplemented!(),
            // Queen  => unimplemented!(),
            // King   => unimplemented!(),
            _      => self._score_positions(&ts, pc, col),
        }
    }

    fn _score_positions(&self, ts: &Tables, pc: Piece, col: Color) -> TaperedScore {
        let pieces = self.get(pc, col);
        let mut score_mg = 0;
        let mut score_eg = 0;
        pieces.iter_bitscan(|sq| {
            score_mg += ts.piece_tables_midgame.get(pc, col, sq);
            score_eg += ts.piece_tables_endgame.get(pc, col, sq);
        });
        TaperedScore::new(score_mg,score_eg)
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

        pub fn taper(&self, phase: u16) -> Score {
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

    // impl std::ops::Mul<Score> for TaperedScore {
    //     type Output = Self;
    //     fn mul(self, x: Score) -> Self {
    //         Self {
    //             mid: self.mid * x,
    //             end: self.end * x,
    //         }
    //     }
    // }

}

