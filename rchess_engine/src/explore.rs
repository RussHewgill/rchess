
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;

#[derive(Debug,PartialEq,PartialOrd,Clone)]
pub struct Explorer {
    pub side:       Color,
    pub game:       Game,
    // pub stats:      ABStats
}

#[derive(Debug,PartialEq,PartialOrd,Clone,Copy)]
pub struct ABStats {
    // /// Min score for Self
    // pub alpha:      Option<(Eval, i32)>,
    // /// Max score for Other
    // pub beta:       Option<(Eval, i32)>,
    /// Min score for Self
    pub alpha:      i32,
    /// Max score for Other
    pub beta:       i32,
    pub leaves:     u64,
}

impl ABStats {
    pub fn new() -> Self {
        Self {
            alpha:      i32::MIN,
            beta:       i32::MAX,
            // alpha:      None,
            // beta:       None,
            leaves:     0,
        }
    }
}

impl Explorer {
    pub fn new(side: Color, game: Game) -> Self {
        Self {
            side,
            game,
            // stats: ABStats::new(),
        }
    }
}

impl Explorer {

    pub fn explore(&self, ts: &Tables, depth: u32) -> Option<Move> {

        // let e = self.game.evaluate(&ts, self.side);
        // eprintln!("e = {:?}", e.diff());

        // let mut ms = self.negamax(&ts, depth, self.side);

        let ms = self.ab_search(&ts, depth);

        // eprintln!("ms = {:?}", ms);

        ms.map(|x| x.0)

        // let ms = Eval::sort_rev(self.side, self.game.state.side_to_move, ms, |x| x.1);
        // for (m,e) in ms.into_iter() {
        //     eprintln!("0: {} = {:?}", e.score_material.diff(), m);
        // }

        // unimplemented!()
        // None
    }

    // pub fn ab_search(&self, ts: &Tables, max_depth: u32) -> Vec<(Move,Eval)> {
    pub fn ab_search(&self, ts: &Tables, max_depth: u32) -> Option<(Move,i32)> {

        let g = self.game.clone();
        // let m = Move::Capture { from: "B7".into(), to: "C5".into() };
        // let g = g.make_move_unchecked(&ts, &m).unwrap();
        // eprintln!("g = {:?}", g);

        // let mut stats = ABStats::new();
        // let score = self.ab_max(&ts, &mut stats, self.game.clone(), max_depth, None);
        // let score = self.ab_max(&ts, i32::MIN, i32::MAX, self.game.clone(), max_depth, None);
        let score = self.ab_max(&ts, i32::MIN, i32::MAX, g, max_depth, None);

        // let score = if self.side == g.state.side_to_move {
        //     self.ab_max(&ts, i32::MIN, i32::MAX, g, max_depth, None)
        // } else {
        //     self.ab_min(&ts, i32::MIN, i32::MAX, g, max_depth, None)
        // };

        match score {
            Some((Some(m), s)) => Some((m,s)),
            _                  => None,
        }

    }

    // pub fn _ab_search(&self, ts: &Tables, mut stats: &mut ABStats, g: Game, depth: u32)
    //                   -> Option<Eval> {
    //     if depth == 0 { return Some(g.evaluate(&ts)); }
    //     let moves = g.search_all(&ts, g.state.side_to_move);

    //     if self.side == g.state.side_to_move {
    //         // Own

    //         let mut val = i32::MIN;
    //         for m in moves.into_iter() {
    //             // val = val.max();
    //             let g2 = g.make_move_unchecked(&ts, &m)?;
    //             let score = self._ab_search(&ts, &mut stats, g2, depth - 1)?;
    //         }


    //         unimplemented!()
    //     } else {
    //         // Other
    //         unimplemented!()
    //     }
    // }

    // pub fn ab_max(&self, ts: &Tables, mut stats: &mut ABStats, g: Game, depth: u32, m0: Option<Move>,
    pub fn ab_max(&self, ts: &Tables, alpha: i32, beta: i32, g: Game, depth: u32, m0: Option<Move>,
    ) -> Option<(Option<Move>,i32)> {
        if depth == 0 {
            // stats.leaves += 1;
            return Some((m0,-g.evaluate(&ts).diff()));
        }
        let moves = g.search_all(&ts, g.state.side_to_move);

        // let moves = {
        //     let m0 = Move::Capture { from: "B4".into(), to: "C5".into() };
        //     let m1 = Move::Capture { from: "B4".into(), to: "A5".into() };
        //     let m2 = Move::Quiet { from: "B4".into(), to: "B5".into() };
        //     vec![m0,m1,m2]
        // };

        let (mut alpha, mut beta) = (alpha,beta);
        let mut m1 = None;

        for m in moves.into_iter() {
            // let out: Option<(Option<Move>,i32)> = {
            let g2 = g.make_move_unchecked(&ts, &m).expect(&format!("ab_max g2: {:?}", m));
            // let (m,score) = self.ab_min(&ts, &mut stats, g2, depth - 1, Some(m)).expect("ab_max score");
            m1 = Some(m);

            let (m2,score) = self.ab_min(&ts, alpha, beta, g2, depth - 1, Some(m)).expect("ab_max score");

            // eprintln!("score, m = {:?}: {:?}", score, m);

            if score >= beta {
                // println!("wat 0");
                // XXX: 
                // return Some((None,beta));
                return Some((m2,beta));
            }

            if score > alpha {
                alpha = score;
            }

            // if score >= stats.beta {
            //     Some((m,stats.beta))
            // } else {
            //     if score > stats.alpha {
            //         stats.alpha = score;
            //     }
            //     Some((m,stats.alpha))
            // }

            // };
            // return out;
        }
        Some((m1, alpha))
        // Some()
        // None
        // panic!("ab_max wat");
    }

    // pub fn ab_min(&self, ts: &Tables, mut stats: &mut ABStats, g: Game, depth: u32, m0: Option<Move>,
    pub fn ab_min(&self, ts: &Tables, alpha: i32, beta: i32, g: Game, depth: u32, m0: Option<Move>,
    ) -> Option<(Option<Move>,i32)> {
        if depth == 0 {
            // stats.leaves += 1;
            return Some((m0,g.evaluate(&ts).diff()));
        }
        let moves = g.search_all(&ts, g.state.side_to_move);

        let (mut alpha, mut beta) = (alpha,beta);
        let mut m1 = None;

        for m in moves.into_iter() {
            // let out: Option<(Option<Move>,i32)> = {
            let g2 = g.make_move_unchecked(&ts, &m).expect("ab_max g2");
            // let (m,score) = self.ab_min(&ts, &mut stats, g2, depth - 1, Some(m)).expect("ab_max score");
            m1 = Some(m);
            let (m2,score) = self.ab_min(&ts, alpha, beta, g2, depth - 1, Some(m)).expect("ab_max score");

            // eprintln!("score, m = {:?}: {:?}", score, m);

            if score <= alpha {
                // XXX: 
                // return Some((None,alpha));
                return Some((m2,alpha));
            }

            if score < beta {
                beta = score;
            }
        }
        Some((m1,beta))

        // for m in moves.into_iter() {
        //     let out: Option<(Option<Move>,i32)> = {
        //         let g2 = g.make_move_unchecked(&ts, &m)?;
        //         let (m,score) = self.ab_max(&ts, &mut stats, g2, depth - 1, Some(m))?;
        //         if score <= stats.alpha {
        //             return Some((m,stats.alpha));
        //         }
        //         if score < stats.beta {
        //             stats.beta = score;
        //         }
        //         Some((m,stats.beta))
        //     };
        //     return out;
        // }

        // None

        // panic!("ab_min wat");
    }

    pub fn negamax(&self, ts: &Tables, max_depth: u32, col: Color) -> Vec<(Move,Eval)> {

        let moves = self.game.search_all(&ts, self.game.state.side_to_move);
        // let moves = vec![Move::Capture { from: "B7".into(), to: "D8".into()}];

        let mut out = vec![];
        for m in moves.into_iter() {
            if let Some(g) = self.game.make_move_unchecked(&ts, &m) {
                if let Some(s) = self._negamax(&ts, g, max_depth) {
                    out.push((m,s));
                } else {
                    panic!("negamax panic 1: {:?}", m);
                }
            } else {
                panic!("negamax panic 0: {:?}", m);
            }
        }

        out
    }

    fn _negamax(&self, ts: &Tables, g: Game, depth: u32) -> Option<Eval> {

        if depth == 0 { return Some(g.evaluate(&ts)); }

        let moves = g.search_all(&ts, g.state.side_to_move);

        let out = moves.into_iter()
            .map(|m| {
                if let Some(g) = g.make_move_unchecked(&ts, &m) {
                    // let s = self._negamax(&ts, g, depth - 1);
                    // (m,s)
                    if let Some(s) = self._negamax(&ts, g, depth - 1) {
                        Some((m,s))
                    } else { None }
                } else {
                    panic!("negamax panic: {:?}", m)
                }
            })
            .flatten()
            ;

        let ms = out.clone();

        // let out = out.map(|x| x.1);

        // for (m,e) in ms.into_iter() {
        //     eprintln!("{} = {:?}", e.score_material.diff(), m);
        // }

        // XXX: ! ?

        // let out = if g.state.side_to_move == White {
        //     Eval::max(g.state.side_to_move, out)
        // } else {
        //     Eval::min(g.state.side_to_move, out)
        // };

        // for (m,e) in ms.into_iter() {
        //     eprintln!("1: {} = {:?}", e.score_material.diff(), m);
        // }

        // if g.state.side_to_move == White {
        //     println!("wat 0");
        // } else {
        //     println!("wat 1");
        // }

        let out = Eval::best(self.side, !g.state.side_to_move, out, |x| x.1);
        // eprintln!("out = {:?}", out);

        // let out = Eval::max(!g.state.side_to_move, out);
        // eprintln!("out = {:?}", out.unwrap().score_material.diff());

        out.map(|x| x.1)
    }

}


