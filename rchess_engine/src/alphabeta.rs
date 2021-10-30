
use crate::searchstats;
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
use crate::pruning::*;
use crate::explore::*;

use std::collections::VecDeque;

use std::sync::atomic::Ordering::SeqCst;

/// AB search
impl Explorer {

    /// returns (can_use, SearchInfo)
    /// true: when score can be used instead of searching
    /// false: score can be used ONLY for move ordering
    pub fn check_tt(&self,
                ts:             &Tables,
                g:              &Game,
                depth:          Depth,
                maximizing:     bool,
                tt_r:           &TTRead,
                mut stats:      &mut SearchStats,
    ) -> Option<(SICanUse,SearchInfo)> {
        if let Some(si) = tt_r.get_one(&g.zobrist) {
            if si.depth_searched >= depth {
                stats.tt_hits += 1;
                Some((SICanUse::UseScore,si.clone()))
            } else {
                stats.tt_misses += 1;
                Some((SICanUse::UseOrdering,si.clone()))
            }

        } else {
            // if g.zobrist == Zobrist(0x1eebfbac03c62e9d) { println!("wat wat 3"); }
            stats.tt_misses += 1;
            // let score = self._ab_search(&ts, &g, depth - 1, k + 1, alpha, beta, !maximizing);
            // score
            None
        }
    }

    #[allow(unused_doc_comments)]
    /// alpha: the MIN score that the maximizing player is assured of
    /// beta:  the MAX score that the minimizing player is assured of
    pub fn _ab_search(
        &self,
        ts:                 &Tables,
        g:                  &Game,
        max_depth:          Depth,
        depth:              Depth,
        k:                  i16,
        mut alpha:          i32,
        mut beta:           i32,
        maximizing:         bool,
        mut stats:          &mut SearchStats,
        // mv0:                Move,
        prev_mvs:           VecDeque<(Zobrist,Move)>,
        mut history:        &mut [[[Score; 64]; 64]; 2],
        tt_r:               &TTRead,
        tt_w:               TTWrite,
    // ) -> Option<(Vec<Move>, Score)> {
    ) -> Option<((Vec<Move>, Score), (i32, i32))> {

        if self.stop.load(SeqCst) {
            return None;
        }

        {
            let r = self.best_mate.read();
            if let Some(best) = *r {
                drop(r);
                if best <= max_depth {
                    trace!("halting search of depth {}, faster mate found", max_depth);
                    return None;
                }
            }
        }

        let moves = g.search_all(&ts, None);

        let mut moves: Vec<Move> = match moves {
            Outcome::Checkmate(c) => {
                let score = 100_000_000 - k as Score;
                // if !self.tt_contains(&g.zobrist) {
                if !tt_r.contains_key(&g.zobrist) {
                    stats.leaves += 1;
                    stats.checkmates += 1;
                }
                if maximizing {
                    // return Some((vec![mv0],-score));
                    return Some(((vec![], -score),(alpha,beta)));
                } else {
                    // return Some((vec![mv0],score));
                    return Some(((vec![], score),(alpha,beta)));
                }

            },
            Outcome::Stalemate    => {
                let score = -100_000_000 + k as Score;
                // if !self.tt_contains(&g.zobrist) {
                if !tt_r.contains_key(&g.zobrist) {
                    stats.leaves += 1;
                    stats.stalemates += 1;
                }
                // return Some((vec![],score));
                return Some(((vec![], score),(alpha,beta)));
            },
            Outcome::Moves(ms)    => ms,
        };

        // if !tt_r.contains_key(&g.zobrist) {}
        /// XXX: stat padding by including nodes found in TT
        stats.inc_nodes_arr(depth);
        stats.nodes += 1;

        if depth == 0 {

            // let score = g.evaluate(&ts).sum();

            let mv0 = prev_mvs.back().unwrap().1;
            let score = self.quiescence(
                // ts, g, moves, k, alpha, beta, !maximizing, &mut stats, mv0,
                // ts, g, moves, k, alpha, beta, !maximizing, &mut stats,
                ts, g, moves, k, -alpha, -beta, !maximizing, &mut stats,
            );

            if !tt_r.contains_key(&g.zobrist) {
                stats.leaves += 1;
            }
            if self.side == Black {
                // return (vec![mv0], -score);
                // return Some((vec![], -score));
                return Some(((vec![], -score),(alpha,beta)));
            } else {
                // return (vec![mv0], score);
                // return Some((vec![], score));
                return Some(((vec![], score),(alpha,beta)));
            }
        }

        /// Null Move pruning
        #[cfg(feature = "null_pruning")]
        if g.state.checkers.is_empty()
            && g.game_phase() < 200
            && self.prune_null_move(
                // ts, g, max_depth, depth, k, alpha, beta, maximizing, &mut stats, tt_r, tt_w.clone()) {
                ts, g, max_depth, depth, k, alpha, beta, maximizing, &mut stats,
                prev_mvs.clone(), &mut history, tt_r, tt_w.clone()) {
                return None;
        }

        /// MVV LVA move ordering
        order_mvv_lva(&mut moves);

        /// History Heuristic ordering
        #[cfg(feature = "history_heuristic")]
        order_moves_history(&history[g.state.side_to_move], &mut moves);

        /// Make move, Lookup games in Trans Table
        // #[cfg(feature = "par")]
        let mut gs: Vec<(Move,Game,Option<(SICanUse,SearchInfo)>)> = {
            // let mut gs0 = moves.into_par_iter().flat_map(|m| if let Ok(g2) = g.make_move_unchecked(&ts, m) {
            let mut gs0 = moves.into_iter().flat_map(|m| if let Ok(g2) = g.make_move_unchecked(&ts, m) {
                let mut ss = SearchStats::default();
                let tt = self.check_tt(&ts, &g2, depth, maximizing, &tt_r, &mut ss);
                // Some(((m,g2,tt), ss))
                *stats = *stats + ss;
                Some((m,g2,tt))
            } else {
                None
            });
            gs0.collect()
        };

        /// Move Ordering
        order_searchinfo(maximizing, &mut gs[..]);

        // let mut node_type = Node::All;
        let mut node_type = Node::PV;
        // let mut parent_node_type = None;

        /// Get parent node type
        let moves = match tt_r.get_one(&g.zobrist) {
            None     => {
                // panic!("no parent node?");
            },
            Some(si) => {
                // parent_node_type = Some(si.node_type);
                match si.node_type {
                    Node::Cut => {
                        node_type = Node::All;
                        /// Cut nodes only need one child to be searched
                        gs.truncate(1);
                    },
                    /// Each child of an All node is a Cut nodes
                    Node::All => node_type = Node::Cut,
                    _         => {},
                }
            }
        };

        let mut val = if maximizing { i32::MIN } else { i32::MAX };
        let mut val: (Option<(Zobrist,Move,Vec<Move>)>,i32) = (None,val);

        // let (alpha0,beta0) = (alpha,beta);

        let mut moves_searched = 0;

        'outer: for (mv,g2,tt) in gs.iter() {

            let zb = g2.zobrist;

            /// Cycle prevention
            if self.cycle_prevention(&ts, (mv,g2), &prev_mvs) {
                panic!("wat: {:?}\n {:?}", mv, g2)
            }

            let (can_use,mut mv_seq,score) = match tt {
                Some((SICanUse::UseScore,si)) => {
                    // return (si.moves.clone(),si.score);
                    // (true,si.moves.clone(),si.score)
                    (true,si.moves.to_vec(),si.score)
                },
                _ => 'search: {
                    let mut pms = prev_mvs.clone();
                    pms.push_back((g.zobrist,*mv));

                    let mut lmr = true;

                    let mut depth2 = depth - 1;

                    /// not reducing when in check replaces check extension
                    #[cfg(feature = "late_move_reduction")]
                    if lmr
                        && moves_searched >= 4
                        && k >= 3
                        && depth > 2
                        // && depth > 3
                        && !mv.filter_all_captures()
                        && !mv.filter_promotion()
                        && g.state.checkers.is_empty()
                        && g2.state.checkers.is_empty()
                    {
                        let depth2 = depth - 2;
                        // trace!("Checking Late Move Reduction");
                        if let Some(((mv_seq,score),_)) = self._ab_search(
                            &ts, &g2, max_depth, depth2, k + 1,
                            alpha, beta, !maximizing, &mut stats, pms.clone(),
                            &mut history,
                            tt_r, tt_w.clone()) {
                            if maximizing {
                                if score <= alpha {
                                    // trace!("Late move reduction success 1");
                                    stats.lmrs.0 += 1;
                                    break 'search (false,mv_seq,score);
                                }
                            } else {
                                if score >= beta {
                                    // trace!("Late move reduction success -1");
                                    stats.lmrs.1 += 1;
                                    break 'search (false,mv_seq,score);
                                }
                            }
                        } else {
                            break 'outer;
                        }
                    }

                    if let Some(((mv_seq,score),_)) = self._ab_search(
                        &ts, &g2, max_depth, depth2, k + 1,
                        // alpha, beta, !maximizing, &mut stats, *mv,
                        alpha, beta, !maximizing, &mut stats, pms,
                        &mut history,
                        tt_r, tt_w.clone(),
                    ) {
                        (false,mv_seq,score)
                    } else {
                        break 'outer;
                    }

                },
            };

            // /// basic minimax
            // if maximizing {
            //     val.1 = i32::max(val.1, score);
            // } else {
            //     val.1 = i32::min(val.1, score);
            // }

            let b = self._ab_score(
                (*mv,&g2),
                (can_use,mv_seq,score),
                &mut val,
                depth,
                &mut alpha,
                &mut beta,
                maximizing,
                // mv0
            );
            if b {
                node_type = Node::Cut;

                if !mv.filter_all_captures() {
                    history[g.state.side_to_move][mv.sq_from()][mv.sq_to()] += k as Score * k as Score;
                }

                if moves_searched == 0 {
                    stats.beta_cut_first.0 += 1;
                } else {
                    stats.beta_cut_first.1 += 1;
                }

                break;
            }

            moves_searched += 1;
        }

        // XXX: depth or depth - 1? Update: Pretty sure depth - 1 is correct
        if let Some((zb,mv,mv_seq)) = &val.0 {
            let mut mv_seq = mv_seq.clone();
            Self::tt_insert_deepest(
                &tt_r, tt_w,
                *zb, SearchInfo::new(*mv, mv_seq, depth - 1, node_type, val.1));
        }

        stats.alpha = stats.alpha.max(alpha);
        stats.beta  = stats.beta.max(beta);

        match &val.0 {
            Some((zb,mv,mv_seq)) => Some(((mv_seq.clone(),val.1),(alpha,beta))),
            _                    => None,
        }

    }


    pub fn _ab_score(
        &self,
        (mv,g2):                       (Move,&Game),
        (can_use,mut mv_seq,score):    (bool,Vec<Move>,Score),
        mut val:                       &mut (Option<(Zobrist,Move,Vec<Move>)>,i32),
        depth:                         Depth,
        mut alpha:                     &mut i32,
        mut beta:                      &mut i32,
        maximizing:                    bool,
        // mv0:                           Move,
    ) -> bool {
        let zb = g2.zobrist;
        if maximizing {
            // if zb == Zobrist(0x1eebfbac03c62e9d) { println!("wat 0"); }
            if score > val.1 {
                // if zb == Zobrist(0x1eebfbac03c62e9d) { println!("wat 1"); }
                // mv_seq.push(mv);
                if !can_use { mv_seq.push(mv) };
                *val = (Some((zb,mv,mv_seq.clone())),score);
            }

            if val.1 > *alpha {
                *alpha = val.1;
            }

            // *alpha = i32::max(*alpha, val.1);
            if val.1 >= *beta { // Beta cutoff
                // if zb == Zobrist(0x1eebfbac03c62e9d) { println!("wat 2"); }
                // self.trans_table.insert(
                //     zb, SearchInfo::new(mv, mv_seq.clone(), depth, Node::Cut, val.1));
                return true;
                // return Some(Node::Cut);
            }

            // self.trans_table.insert_replace(
            //     zb, SearchInfo::new(mv, depth, Node::All, val.1));
        } else {
            // if zb == Zobrist(0x1eebfbac03c62e9d) { println!("wat 3"); }
            if score < val.1 {
                // if zb == Zobrist(0x1eebfbac03c62e9d) { println!("wat 4"); }
                // mv_seq.push(mv);
                if !can_use { mv_seq.push(mv) };
                *val = (Some((zb,mv,mv_seq.clone())),score);
            }

            *beta = i32::min(*beta, val.1);
            if val.1 <= *alpha {
                // if zb == Zobrist(0x1eebfbac03c62e9d) { println!("wat 5"); }
                // self.trans_table.insert(
                //     zb, SearchInfo::new(mv, mv_seq.clone(), depth, Node::Cut, val.1));
                return true;
                // return Some(Node::Cut);
            }

            // node_type = Node::All;
            // self.trans_table.insert_replace(
            //     zb, SearchInfo::new(mv, depth, Node::All, val.1));
        }
        false
        // None
    }

}


