
use crate::types::*;
use crate::tables::*;

pub type Score = i32;

#[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct Eval {
    // material_white:         [Score; 6],
    // material_black:         [Score; 6],
    // piece_positions_white:  [Score; 6],
    // piece_positions_black:  [Score; 6],
    pub material:         [[Score; 6]; 2],
    pub piece_positions:  [[Score; 6]; 2],
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

        for &col in [White,Black].iter() {
            for pc in Piece::iter_pieces() {
                let m = self.score_material(pc, col);
                out.set_piece_mat_mut(pc, col, m);
                let pos = self.score_positions(&ts, pc, col);
                out.set_piece_pos_mut(pc, col, pos);

                // eprintln!("scoring = {:?} {:?}: {:?}/{:?}", col, pc, m, pos);
            }
        }
        // out.score_material = self.score_material();
        // out.score_position = self.score_position();
        out
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

    pub fn score_material(&self, pc: Piece, col: Color) -> i32 {
        match pc {
            // Rook   => {},
            // Knight => {},
            Bishop => {
                let n = self.get(pc, col).popcount() as i32;
                if n > 1 {
                    // 2 bishops = 0.5 pawn
                    pc.score() * n + 50
                } else {
                    pc.score() * n
                }
            },
            _      => {
                pc.score() * self.get(pc, col).popcount() as i32
            },
        }
    }

}

/// Positional Scoring
impl Game {

    pub fn game_phase(&self, ts: &Tables) -> i16 {
        let pawn_ph = 0;
        let knight_ph = 1;
        let bishop_ph = 1;
        let rook_ph = 2;
        let queen_ph = 4;
        let pcs = [Pawn,Rook,Knight,Bishop,Queen];
        let phases: [i16; 5] = [pawn_ph,rook_ph,knight_ph,bishop_ph,queen_ph];

        let ph_total = pawn_ph * 16 + knight_ph * 4 + bishop_ph * 4 + rook_ph * 4 + queen_ph * 2;
        let mut ph = ph_total;

        for &col in [White,Black].iter() {
            for pc in pcs {
                let ps = self.get(pc, col);
                let pn = ps.popcount() as i16;
                ph -= pn * phases[pc.index()];
            }
        }

        let phase = (ph * 256 + (ph_total / 2)) / ph_total;

        phase
    }

    pub fn score_positions(&self, ts: &Tables, pc: Piece, col: Color) -> Score {
        match pc {
            Pawn   => {
                let pos = self._score_positions(&ts, Pawn, col);
                let ds = self.count_pawns_doubled(&ts, col);
                pos + (ds as Score * ts.piece_tables_opening.ev_pawn.doubled)
                // pos
            },
            // Rook   => unimplemented!(),
            // Knight => unimplemented!(),
            // Bishop => unimplemented!(),
            // Queen  => unimplemented!(),
            // King   => unimplemented!(),
            _      => self._score_positions(&ts, pc, col),
        }
    }

    fn _score_positions(&self, ts: &Tables, pc: Piece, col: Color) -> Score {
        let pieces = self.get(pc, col);
        let mut score = 0;
        pieces.iter_bitscan(|sq| {
            // TODO: interpolate
            score += ts.piece_tables_opening.get(pc, col, sq);
        });
        score
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

// impl Ord for Eval {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         unimplemented!()
//     }
// }

mod old_eval {
    use crate::types::*;

    #[derive(Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
    pub struct Eval {
        // pub material: []
        pub score_material:    Score,
        pub score_position:    Score,
        // contempt: i32,
    }

    #[derive(Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
    pub struct Score {
        pub white:  i32,
        pub black:  i32,
    }

    impl Score {
        pub fn new(white: i32, black: i32) -> Self {
            Self { white, black }
        }
        /// Positive = white winning
        pub fn diff(&self) -> i32 {
            self.white - self.black
        }
        pub fn get(&self, col: Color) -> i32 {
            match col {
                White => self.white,
                Black => self.black,
            }
        }
    }

    impl Eval {
        /// Positive = white winning
        pub fn diff(&self) -> i32 {
            self.score_material.diff()
        }

        // pub fn sort(side: Color, evals: Vec<Eval>) -> Vec<Eval> {
        //     let mut out = evals.clone();
        //     out.sort_by(|a,b| Self::compare(side, *a, *b));
        //     out
        // }

        // pub fn sort_rev(side: Color, turn: Color, evals: Vec<Eval>) -> Vec<Eval> {
        //     let mut out = evals.clone();
        //     out.sort_by(|a,b| Self::compare(side, turn, *a, *b));
        //     out.reverse();
        //     out
        // }

        pub fn sort<F,T>(side: Color, turn: Color, xs: Vec<T>, mut f: F) -> Vec<T> where
            F: FnMut(&T) -> Eval,
            T: Clone,
        {
            let mut out = xs.clone();
            out.sort_by(|a,b| Self::compare(side, turn, f(a), f(b)));
            out
        }

        pub fn sort_rev<F,T>(side: Color, turn: Color, xs: Vec<T>, mut f: F) -> Vec<T> where
            F: FnMut(&T) -> Eval,
            T: Clone,
        {
            let mut out = xs.clone();
            out.sort_by(|a,b| Self::compare(side, turn, f(a), f(b)));
            out.reverse();
            out
        }

        pub fn compare(side: Color, turn: Color, a: Eval, b: Eval) -> std::cmp::Ordering {
            let s0 = a.score_material.get(side) - a.score_material.get(!side);
            if turn == side {
                s0.cmp(&0)
            } else {
                (-s0).cmp(&0)
            }
        }

        pub fn best<F,T>(side: Color, turn: Color, xs: impl Iterator<Item = T>, mut f: F) -> Option<T>
            where F: FnMut(&T) -> Eval
        {
            xs.min_by(|a,b| Eval::compare(side, turn, f(a), f(b)))
            // if side == turn {
            //     // Eval::max(g.state.side_to_move, out)
            //     xs.max_by(|a,b| Eval::compare(side, turn, f(a), f(b)))
            // } else {
            //     // Eval::min(g.state.side_to_move, out)
            //     xs.min_by(|a,b| Eval::compare(side, turn, f(a), f(b)))
            //     // unimplemented!()
            // }
        }

        pub fn max(side: Color, turn: Color, xs: impl Iterator<Item = Eval>) -> Option<Eval> {
            xs.max_by(|a,b| Eval::compare(side, turn, *a, *b))
        }

        pub fn min(side: Color, turn: Color, xs: impl Iterator<Item = Eval>) -> Option<Eval> {
            xs.min_by(|a,b| Eval::compare(side, turn, *a, *b))
        }

    }

    impl std::fmt::Debug for Eval {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            // f.write_str(&format!("Coord({}{})", r, self.1+1))?;
            f.write_str(&format!("Mat: {:?}", self.score_material))?;
            Ok(())
        }
    }

    impl std::fmt::Debug for Score {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            // f.write_str(&format!("Coord({}{})", r, self.1+1))?;
            f.write_str(&format!("Score({})({}/{})", self.white - self.black, self.white, self.black))?;
            Ok(())
        }
    }

}

