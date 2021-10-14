
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;

use rayon::prelude::*;

#[derive(Debug,PartialEq,PartialOrd,Clone)]
pub struct Explorer {
    pub side:       Color,
    pub game:       Game,
    // pub stats:      ABStats
    pub depth:      u32,
}

impl Explorer {
    pub fn new(side: Color, game: Game, depth: u32) -> Self {
        Self {
            side,
            game,
            // stats: ABStats::new(),
            depth,
        }
    }
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

    pub fn explore(&self, ts: &Tables, depth: u32) -> Option<Move> {

        // let e = self.game.evaluate(&ts, self.side);
        // eprintln!("e = {:?}", e.diff());

        // let mut ms = self.negamax(&ts, depth, self.side);

        // let ms = self.ab_search(&ts, depth, None);
        let mut moves = self.rank_moves(&ts, false);

        // eprintln!("ms = {:?}", ms);

        moves.get(0).map(|x| x.0)

        // ms.map(|x| x.0).flatten()
        // ms.0
        // unimplemented!()

        // let ms = Eval::sort_rev(self.side, self.game.state.side_to_move, ms, |x| x.1);
        // for (m,e) in ms.into_iter() {
        //     eprintln!("0: {} = {:?}", e.score_material.diff(), m);
        // }

        // unimplemented!()
        // None
    }

    pub fn rank_moves_list(&self, ts: &Tables, print: bool, moves: Vec<Move>) -> Vec<(Move,i32)> {
        let mut out: Vec<(Move,i32)> = moves.into_par_iter()
        // let mut out: Vec<(Move,i32)> = moves.into_iter()
            .map(|m| {
                let g2 = self.game.make_move_unchecked(&ts, &m).unwrap();
                let alpha = (None,i32::MIN);
                let beta  = (None,i32::MAX);
                let (m2,score) = self._ab_search(&ts, g2, self.depth, 0, Some(m), alpha, beta);
                (m,score)
            })
            .collect();
        out.sort_by(|a,b| a.1.cmp(&b.1));
        out.reverse();
        if print {
            for (m,s) in out.iter() {
                eprintln!("{:?}: {:?}", s, m);
            }
        }
        out
    }

    pub fn rank_moves(&self, ts: &Tables, print: bool) -> Vec<(Move,i32)> {
        let mut out = vec![];

        let moves = self.game.search_all(&ts, self.game.state.side_to_move);
        // let moves = &moves[0..moves.len()];

        if moves.is_end() {
            return out;
            // panic!("is_end?");
        }

        // let moves = vec![
        //     Move::Quiet { from: "A1".into(), to: "D4".into() }, // mate
        //     Move::Quiet { from: "A1".into(), to: "C3".into() }, // mate in 1
        //     Move::Quiet { from: "A1".into(), to: "A4".into() }, // mate in 2
        // ];

        let moves = moves.get_moves_unsafe();
        let mut out: Vec<(Move,i32)> = moves.into_par_iter()
        // let mut out: Vec<(Move,i32)> = moves.into_iter()
            .map(|m| {
                let g2 = self.game.make_move_unchecked(&ts, &m).unwrap();
                let alpha = (None,i32::MIN);
                let beta  = (None,i32::MAX);
                let (m2,score) = self._ab_search(&ts, g2, self.depth, 1, Some(m), alpha, beta);
                (m,score)
            })
            .collect();

        // for m in moves.into_iter() {
        //     // eprintln!("m = {:?}", m);
        //     let g2 = self.game.make_move_unchecked(&ts, &m).unwrap();
        //     // let alpha = (None,i32::MIN,None);
        //     // let beta  = (None,i32::MAX,None);
        //     let alpha = (None,i32::MIN);
        //     let beta  = (None,i32::MAX);
        //     // let (m2,score,mate) = self._ab_search(&ts, g2, self.depth, 1, Some(m), alpha, beta);
        //     let (m2,score) = self._ab_search(&ts, g2, self.depth, 1, Some(m), alpha, beta);
        //     out.push((m,score))
        // }

        out.sort_by(|a,b| a.1.cmp(&b.1));
        out.reverse();

        if print {
            // for (m,s) in out[0..8].iter() {
            for (m,s) in out.iter() {
                eprintln!("{:?}: {:?}", s, m);
            }
        }

        out
        // unimplemented!()
    }

    pub fn ab_search(&self, ts: &Tables, m0: Move) -> i32 {
        let g = self.game.clone();

        match g.make_move_unchecked(&ts, &m0) {
            Ok(g2) => {
                let alpha = (None,i32::MIN);
                let beta  = (None,i32::MAX);
                let (m2,score) = self._ab_search(&ts, g2, self.depth, 1, Some(m0), alpha, beta);
                score
            },
            Err(win) => {
                panic!("bad move? {:?}", &win);
            }
        }
    }

    fn quiescence(&self,
                  ts: &Tables,
                  g: Game,
                  moves: Option<Outcome>,
                  k: i32,
                  mut alpha: i32,
                  mut beta: i32
    ) -> i32 {
        // println!("quiescence 0");
        let stand_pat = g.evaluate(&ts).sum(self.side);

        if stand_pat >= beta {
            return beta;
        }
        if alpha < stand_pat {
            alpha = stand_pat;
        }

        let moves = match moves {
            Some(ms) => ms,
            None     => g.search_all(&ts, g.state.side_to_move),
        };
        match moves {
            Outcome::Checkmate(c) => return 100_000_000 - k,
            Outcome::Stalemate    => return -100_000_000 + k,
            Outcome::Moves(_)     => {},
        }

        let moves = moves.into_iter().filter(|m| m.filter_all_captures());

        for m in moves {
            if let Ok(g2) = g.make_move_unchecked(&ts, &m) {
                let score = -self.quiescence(&ts, g2, None, k+1, -alpha, -beta);

                if score >= beta {
                    return beta;
                }
                if score > alpha {
                    alpha = score;
                }
            }
        }

        alpha
        // unimplemented!()
    }

    fn _ab_search(
        &self,
        ts:         &Tables,
        g:          Game,
        depth:      u32,
        k:          i32,
        m0:         Option<Move>,
        mut alpha:  (Option<Move>,i32),
        mut beta:   (Option<Move>,i32),
    ) -> (Option<Move>,i32) {
        // eprintln!("m0 = {:?}", m0);

        let maximizing = self.side == g.state.side_to_move;

        let mut val = if maximizing {
            i32::MIN
        } else {
            i32::MAX
        };

        let moves = g.search_all(&ts, g.state.side_to_move);

        match moves {
            Outcome::Checkmate(c) => return (m0, 100_000_000 - k),
            Outcome::Stalemate    => return (m0, -100_000_000 + k),
            Outcome::Moves(_)     => {},
        }


        // if moves.is_end() {
        //     // panic!("is_end?");
        //     // println!("is end? {:?}", k);
        //     // return (m0,val,Some(k));
        //     return (m0,1000 - k);
        // }

        if depth == 0 {
            return (m0,g.evaluate(&ts).sum(self.side));
            // return (m0,self.quiescence(&ts, g, Some(moves), k, alpha.1, beta.1));
        }

        for m in moves.into_iter() {
            match g.make_move_unchecked(&ts, &m) {
                Ok(g2)   => {
                    let score = g2.evaluate(&ts);
                    let score = score.sum(self.side);

                    if maximizing {
                        // maximize self
                        let val2 = self._ab_search(&ts, g2, depth - 1, k + 1, Some(m), alpha, beta);
                        val = val.max(val2.1);

                        if val >= beta.1 {
                            return beta;
                        }

                        if alpha.0.is_none() {
                            alpha = (Some(m), val);
                        } else if val > alpha.1 {
                            alpha = (Some(m), val);
                        }

                    } else {
                        // minimize other
                        let val2 = self._ab_search(&ts, g2, depth - 1, k + 1, Some(m), alpha, beta);
                        val = val.min(val2.1);

                        if val <= alpha.1 {
                            return alpha;
                        }

                        if beta.0.is_none() {
                            beta = (Some(m), val);
                        } else if val < beta.1 {
                            beta = (Some(m), val);
                        }
                    }
                },
                Err(end) => {
                    panic!()
                },
            }
        }

        if maximizing {
            alpha
        } else {
            beta
        }
    }

    // pub fn negamax(&self, ts: &Tables, max_depth: u32, col: Color) -> Vec<(Move,Eval)> {

    //     let moves = self.game.search_all(&ts, self.game.state.side_to_move);
    //     // let moves = vec![Move::Capture { from: "B7".into(), to: "D8".into()}];

    //     // let mut out = vec![];
    //     // for m in moves.into_iter() {
    //     //     if let MoveResult::Legal(g) = self.game.make_move_unchecked(&ts, &m) {
    //     //         if let Some(s) = self._negamax(&ts, g, max_depth) {
    //     //             out.push((m,s));
    //     //         } else {
    //     //             panic!("negamax panic 1: {:?}", m);
    //     //         }
    //     //     } else {
    //     //         panic!("negamax panic 0: {:?}", m);
    //     //     }
    //     // }

    //     // out
    //     unimplemented!()
    // }

    // fn _negamax(&self, ts: &Tables, g: Game, depth: u32) -> Option<Eval> {

    //     if depth == 0 { return Some(g.evaluate(&ts)); }

    //     let moves = g.search_all(&ts, g.state.side_to_move);

    //     // let out = moves.into_iter()
    //     //     .map(|m| {
    //     //         if let MoveResult::Legal(g) = g.make_move_unchecked(&ts, &m) {
    //     //             // let s = self._negamax(&ts, g, depth - 1);
    //     //             // (m,s)
    //     //             if let Some(s) = self._negamax(&ts, g, depth - 1) {
    //     //                 Some((m,s))
    //     //             } else { None }
    //     //         } else {
    //     //             panic!("negamax panic: {:?}", m)
    //     //         }
    //     //     })
    //     //     .flatten()
    //     //     ;

    //     // let ms = out.clone();

    //     // let out = out.map(|x| x.1);

    //     // for (m,e) in ms.into_iter() {
    //     //     eprintln!("{} = {:?}", e.score_material.diff(), m);
    //     // }

    //     // XXX: ! ?

    //     // let out = if g.state.side_to_move == White {
    //     //     Eval::max(g.state.side_to_move, out)
    //     // } else {
    //     //     Eval::min(g.state.side_to_move, out)
    //     // };

    //     // for (m,e) in ms.into_iter() {
    //     //     eprintln!("1: {} = {:?}", e.score_material.diff(), m);
    //     // }

    //     // if g.state.side_to_move == White {
    //     //     println!("wat 0");
    //     // } else {
    //     //     println!("wat 1");
    //     // }

    //     // let out = Eval::best(self.side, !g.state.side_to_move, out, |x| x.1);
    //     // eprintln!("out = {:?}", out);

    //     // let out = Eval::max(!g.state.side_to_move, out);
    //     // eprintln!("out = {:?}", out.unwrap().score_material.diff());

    //     // out.map(|x| x.1)
    //     unimplemented!()
    // }

}


