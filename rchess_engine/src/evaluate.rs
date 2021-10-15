
use crate::types::*;
use crate::tables::*;

pub type Score = i32;

#[derive(Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct Eval {
    material_white:         [Score; 6],
    material_black:         [Score; 6],
    piece_positions_white:  [Score; 6],
    piece_positions_black:  [Score; 6],
}

impl Eval {

    pub fn sum(&self, col: Color) -> Score {

        let white = self.sum_color(White);
        let black = self.sum_color(Black);

        // if col == White {
        //     white - black
        // } else {
        //     black - white
        // }
        white - black
        // unimplemented!("Eval::diff()")
    }

    pub fn sum_color(&self, col: Color) -> Score {
        let mut score = 0;
        match col {
            White => {
                for m in self.material_white.iter() {
                    score += m;
                }
                for m in self.piece_positions_white.iter() {
                    score += m;
                }
            },
            Black => {
                for m in self.material_black.iter() {
                    score += m;
                }
                for m in self.piece_positions_black.iter() {
                    score += m;
                }
            },
        }
        score
    }

    pub fn get_piece_pos(&self, pc: Piece, col: Color) -> Score {
        match col {
            White => self.piece_positions_white[pc.index()],
            Black => self.piece_positions_black[pc.index()],
        }
    }
    pub fn set_piece_pos_mut(&mut self, pc: Piece, col: Color, s: Score) {
        match col {
            White => self.piece_positions_white[pc.index()] = s,
            Black => self.piece_positions_black[pc.index()] = s,
        }
    }
    pub fn get_piece_mat(&self, pc: Piece, col: Color) -> Score {
        match col {
            White => self.material_white[pc.index()],
            Black => self.material_black[pc.index()],
        }
    }
    pub fn set_piece_mat_mut(&mut self, pc: Piece, col: Color, s: Score) {
        match col {
            White => self.material_white[pc.index()] = s,
            Black => self.material_black[pc.index()] = s,
        }
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

    fn score_material(&self, pc: Piece, col: Color) -> i32 {
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

    fn score_positions(&self, ts: &Tables, pc: Piece, col: Color) -> Score {
        match pc {
            // Pawn   => self.score_positions_pawns(&ts, col),
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

    // fn score_positions_pawns(&self, ts: &Tables, col: Color) -> Score {
    //     let pawns = self.get(Pawn, col);
    //     let mut score = 0;
    //     pawns.iter_bitscan(|sq| {
    //         score += ts.piece_tables.get(Pawn, col, sq);
    //     });
    //     score
    // }

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

