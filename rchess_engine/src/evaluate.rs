
use crate::types::*;
use crate::tables::*;

#[derive(Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct Eval {
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

impl Game {

    pub fn score_material(&self) -> Score {
        let (mut w,mut b) = (0,0);
        for pc in Piece::iter_pieces() {
            w += pc.score() * self.get(pc, White).popcount() as i32;
            b += pc.score() * self.get(pc, Black).popcount() as i32;
        }
        Score::new(w,b)
    }

    pub fn score_position(&self) -> Score {
        unimplemented!()
    }

    pub fn evaluate(&self, ts: &Tables) -> Eval {
        let mut out = Eval::default();
        out.score_material = self.score_material();
        // out.score_position = self.score_position();
        out
    }

}

// impl Ord for Eval {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         unimplemented!()
//     }
// }

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
        f.write_str(&format!("Score({}/{})", self.white, self.black))?;
        Ok(())
    }
}


