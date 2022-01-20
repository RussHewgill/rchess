
use crate::explore::*;
use crate::types::*;
use crate::tables::*;
use crate::trans_table::*;
use crate::searchstats::*;
use crate::evaluate::*;
use crate::movegen::*;

use rayon::prelude::*;

use std::cmp::Ordering;
use std::collections::HashMap;
use evmap_derive::ShallowCopy;

#[derive(Debug,Eq,PartialEq,Ord,PartialOrd,Hash,ShallowCopy,Clone,Copy)]
pub enum OrdMove {
    Hash,
    KillerMate,
    PromCapture,
    Prom,
    /// GoodCapture = QxQ (119), RxR (118), NxB (117), BxB (116), BxN (115) and pxp (114).
    GoodCapture(i8),
    // GoodCapture,
    KillerMove,
    CounterMove,
    // KillerMove(u8),
    CaptureGoodSee(i8),
    // CaptureGoodSee,
    // Capture(u8),
    CaptureEvenSee,
    Castle,
    PromMinor,
    CaptureBadSee(i8),
    // CaptureBadSee,
    OtherScore(i8),
    Other,
}

// impl OrdMove {
//     pub fn to_score(self) -> i8 {
//         use self::OrdMove::*;
//         match self {
//             Hash                => 0,
//             KillerMate          => 1,
//             PromCapture         => 4,
//             Prom                => 5,
//             /// TODO: Order?
//             // GoodCapture         => 10,
//             // KillerMove          => unimplemented!(),
//             KillerMove          => 8,
//             GoodCapture(v)      => 9 + v,
//             CaptureGoodSee(see) => unimplemented!(),
//             CaptureEvenSee      => unimplemented!(),
//             Castle              => unimplemented!(),
//             PromMinor           => unimplemented!(),
//             CaptureBadSee(see)  => unimplemented!(),
//             OtherScore(score)   => unimplemented!(),
//             Other               => unimplemented!(),
//         }
//     }
// }

#[derive(Debug,Eq,PartialEq,Ord,PartialOrd,Clone,Copy)]
pub enum OrdMove2 {

    QueenPromotion = 30_000,
    Capture        = 20_000,

    KillerMove     = 20_001,
    CounterMove    = 15_000,

    UnderPromotion = -1,

}

pub fn score_move_for_sort4(
    // ts:           &'static Tables,
    ts:           &Tables,
    g:            &Game,
    gentype:      MoveGenType,
    mut see_map:  &mut HashMap<Move,Score>,
    st:           &ABStack,
    ply:          Depth,
    mv:           Move,
    killers:      (Option<Move>,Option<Move>),
    countermove:  Option<Move>,
) -> Score {
    use self::OrdMove2::*;

    #[cfg(feature = "history_heuristic")]
    let history = st.get_move_history(mv, g.state.side_to_move, g.last_move);
    #[cfg(not(feature = "history_heuristic"))]
    let history = 0;

    match gentype {
        MoveGenType::Captures    => {
            if mv.filter_promotion() {
                if mv.filter_all_captures() {
                    let see = MoveGen::_static_exchange(ts, g, see_map, mv).unwrap_or(0);
                    QueenPromotion as Score + Capture as Score + history + see
                } else {
                    QueenPromotion as Score + history
                }
            } else if let Some(victim) = mv.victim() {
                let see = MoveGen::_static_exchange(ts, g, see_map, mv).unwrap_or(0);
                // Capture as Score + history + victim.score()
                Capture as Score + history + see
            } else {
                #[cfg(feature = "killer_moves")]
                if Some(mv) == killers.0 || Some(mv) == killers.1 {
                    return KillerMove as Score + history;
                }

                panic!("score_move_for_sort4: Captures, but quiet non-killer or counter move: {:?}", mv);
            }
        },
        MoveGenType::Quiets      => {
            if mv.filter_promotion() {
                UnderPromotion as Score + history
            } else {

                #[cfg(feature = "countermove_heuristic")]
                if Some(mv) == countermove {
                    return CounterMove as Score + history;
                }

                history
            }
        },
        MoveGenType::Evasions    => {
            if let Some(victim) = mv.victim() {
                let see = MoveGen::_static_exchange(ts, g, see_map, mv).unwrap_or(0);
                // Capture as Score + history + victim.score()
                Capture as Score + history + see
            } else {
                history
            }
        },
        MoveGenType::QuietChecks => {
            history
        },
    }
}

pub fn score_move_for_sort3(
    ts:           &'static Tables,
    g:            &Game,
    mut see_map:  &mut HashMap<Move,Score>,
    st:           &ABStack,
    ply:          Depth,
    mv:           Move,
    killers:      (Option<Move>,Option<Move>),
    countermove:  Option<Move>,
) -> Score {
    use self::OrdMove2::*;

    if mv.filter_promotion() {
        if mv.piece() == Some(Queen) {
            if mv.filter_all_captures() {
                let history = st.capture_history.get(mv);
                // let history = 0;
                QueenPromotion as Score + Capture as Score + history
            } else {
                QueenPromotion as Score
            }
        } else {
            UnderPromotion as Score
        }
    } else if mv.filter_all_captures() {
        let history = st.capture_history.get(mv);
        // let history = 0;

        if let Some(see) = MoveGen::_static_exchange(ts, g, see_map, mv) {
            Capture as Score + see + history
        } else {
            Capture as Score + history
        }
    } else {

        let history = st.get_move_history(mv, g.state.side_to_move, g.last_move);

        history
    }
}

// pub fn selection_sort<T: PartialOrd>(xs: &mut [T]) {
pub fn selection_sort<T: PartialOrd>(xs: &mut [(Move,T)]) {

    for ii in 0..xs.len() {

        let mut kmin = ii;

        for kk in ii+1 .. xs.len() {
            if xs[kk].1 < xs[kmin].1 {
                kmin = kk;
            }
        }

        if kmin != ii {
            xs.swap(ii, kmin);
        }

    }

}

#[cfg(feature = "nope")]
pub fn score_move_for_sort(
    ts:           &'static Tables,
    g:            &Game,
    mut see_map:  &mut HashMap<Move,Score>,
    stage:        MoveGenStage,
    gen_type:     MoveGenType,
    st:           &ABStack,
    ply:          Depth,
    mv:           Move,
    killers:      (Option<Move>,Option<Move>),
    countermove:  Option<Move>,
) -> OrdMove {
    use self::OrdMove::*;

    // let mut bonus = 0;

    match mv {
        Move::PromotionCapture { .. }            => return PromCapture,
        Move::Promotion { new_piece: Queen, .. } => return Prom,
        Move::Promotion { .. }                   => return PromMinor,
        Move::EnPassant { .. }                   => return GoodCapture(0),
        // Move::Capture { pc, victim, .. } => match (pc,victim) {
        Move::Capture { pcs, .. } => match (pcs.first(), pcs.second()) {

            (Queen,Queen)   => return GoodCapture(1),
            (Rook,Rook)     => return GoodCapture(2),
            (Knight,Bishop) => return GoodCapture(3),
            (Bishop,Bishop) => return GoodCapture(4),
            (Bishop,Knight) => return GoodCapture(5),
            (Pawn,Pawn)     => return GoodCapture(6),

            _               => {
                if let Some(see) = MoveGen::_static_exchange(ts, g, see_map, mv) {
                    if see == 0 {
                        return CaptureEvenSee;
                    } else if see > 0 {
                        return CaptureGoodSee(scale_score_to_i8(see));
                    } else if see < 0 {
                        return CaptureBadSee(scale_score_to_i8(see));
                    }
                }
            },

        }
        Move::Castle { .. } => return Castle,
        _                                 => {},
    }

    #[cfg(feature = "killer_moves")]
    if Some(mv) == killers.0 || Some(mv) == killers.1 {
        return KillerMove;
    }

    #[cfg(feature = "countermove_heuristic")]
    if Some(mv) == countermove {
        return CounterMove;
    }

    #[cfg(feature = "history_heuristic")]
    if let Some(hist_score) = st.get_score_for_ordering(mv, g.state.side_to_move) {
        return OtherScore(scale_score_to_i8(hist_score));
    }

    // #[cfg(feature = "history_heuristic")]
    // if mv.filter_quiet() || mv.filter_pawndouble() {
    //     if let Some(hist) = st.history.get_move(mv, g.state.side_to_move) {
    //         return OtherScore(scale_score_to_i8(hist));
    //     }
    // }

    Other
}

#[cfg(feature = "nope")]
impl ExHelper {

    pub fn order_moves(
        &self,
        ts:           &Tables,
        g:            &Game,
        ply:          Depth,
        tracking:     &ABStack,
        mut gs:       &mut [(Move,Zobrist,Option<(SICanUse,SearchInfo)>)],
    ) {

        // #[cfg(feature = "killer_moves")]
        // {
        //     let killers = tracking.killers.get(g.state.side_to_move,ply);
        //     gs.sort_by_cached_key(|x| {
        //         Self::order_score_move(ts, g, ply, tracking, x, &killers)
        //     });
        // }

        #[cfg(not(feature = "killer_moves"))]
        gs.sort_by_cached_key(|x| {
            Self::order_score_move(ts, g, ply, tracking, x)
        });

        // gs.reverse();
    }

    pub fn order_score_move(
        ts:           &Tables,
        g:            &Game,
        ply:          Depth,
        tracking:     &ABStack,
        (mv,zb,mtt):  &(Move,Zobrist,Option<(SICanUse,SearchInfo)>),
        #[cfg(feature = "killer_moves")]
        killers:      &(Option<Move>,Option<Move>),
    ) -> OrdMove {
        use self::OrdMove::*;

        match mtt {
            Some((SICanUse::UseScore,_))     => return Hash,
            // Some((SICanUse::UseOrdering,si)) => { // XXX: use lower-depth values for ordering?
            //     if si.node_type == Node::PV { return Hash; }
            // },
            // None                             => {},
            _ => {},
        }
        // TODO: killer mate

        match *mv {
            Move::PromotionCapture { .. }            => return PromCapture,
            Move::Promotion { new_piece: Queen, .. } => return Prom,
            Move::Promotion { .. }                   => return PromMinor,
            Move::EnPassant { .. }                   => return GoodCapture,
            Move::Capture { pc, victim, .. } => match (pc,victim) {
                (Queen,Queen)   => return GoodCapture,
                (Rook,Rook)     => return GoodCapture,
                (Knight,Bishop) => return GoodCapture,
                (Bishop,Bishop) => return GoodCapture,
                (Bishop,Knight) => return GoodCapture,
                (Pawn,Pawn)     => return GoodCapture,
                _               => {
                    if let Some(see) = g.static_exchange(ts, *mv) {
                        if see == 0 {
                            return CaptureEvenSee;
                        } else if see > 0 {
                            // return CaptureGoodSee((see / 1000).clamp(-127,127) as i8);
                            return CaptureGoodSee;
                        } else if see < 0 {
                            // return CaptureBadSee((see / 1000).clamp(-127,127) as i8);
                            return CaptureBadSee;
                        }
                    }
                },
            }
            Move::Castle { .. } => return Castle,
            _                                 => {},
        }

        #[cfg(feature = "killer_moves")]
        if Some(*mv) == killers.0 || Some(*mv) == killers.1 {
            return KillerMove;
        }


        // #[cfg(feature = "killer_moves")]
        // {
        //     let x = tracking.killers.get(g.state.side_to_move, ply, mv);
        //     if x > 0 { return KillerMove; }
        // }

        #[cfg(feature = "history_heuristic")]
        {
        }

        Other
    }

}

pub fn order_moves_piece_tables(ts: &Tables, mut xs: &mut [Move]) {
    // xs.par_sort_unstable_by(|a,b| {
    //     let s0 = ts.piece_tables.get_mid(a.piece(), col, a.sq_from())
    // });
    unimplemented!()
}

pub fn order_moves_history(history: &[[Score; 64]; 64], mut mvs: &mut [Move]) {

    mvs.par_sort_by(|a,b| {
        if !a.filter_all_captures() && !b.filter_all_captures() {
            let a0 = history[a.sq_from()][a.sq_to()];
            let b0 = history[b.sq_from()][b.sq_to()];

            a0.cmp(&b0)
            // unimplemented!()
        } else {
            Ordering::Equal
        }
    });
}

pub fn order_mvv_lva(mut xs: &mut [Move]) {
// pub fn order_mvv_lva(mut xs: &mut [(&str,Move)]) {
    use Move::*;
    // xs.par_sort_unstable_by(|a,b| {
    #[cfg(feature = "par")]
    xs.par_sort_by(|a,b| {
        _order_mvv_lva(a, b)
        // _order_mvv_lva(&a.1, &b.1)
    });
    #[cfg(not(feature = "par"))]
    xs.sort_by(|a,b| {
        _order_mvv_lva(a, b)
        // _order_mvv_lva(&a.1, &b.1)
    });
}

pub fn _order_mvv_lva(a: &Move, b: &Move) -> std::cmp::Ordering {

    match (a.victim(), b.victim()) {
        (Some(v0), Some(v1)) => match (a.piece(),b.piece()) {
            (Some(pc0),Some(pc1)) => {
                // eprintln!("pc0.score = {:?}", pc0.score());
                // eprintln!("pc1.score = {:?}", pc1.score());
                // eprintln!("v0.score = {:?}", v0.score());
                // eprintln!("v1.score = {:?}", v1.score());
                let s0 = v0.score() - pc0.score();
                let s1 = v1.score() - pc1.score();
                s0.cmp(&s1).reverse()
            },
            _                     => panic!("captures with no pieces?"),
        },
        (Some(_), None)      => Ordering::Less,
        (None, Some(_))      => Ordering::Greater,
        _                    => a.cmp(b).reverse(),
    }

}

// pub fn order_searchinfo(mut xs: &mut [(Move,Game,Option<(SICanUse,SearchInfo)>)]) {
pub fn order_searchinfo(mut xs: &mut [(Move,Zobrist,Option<(SICanUse,SearchInfo)>)]) {

    #[cfg(not(feature = "par"))]
    {
        xs.sort_by(|a,b| {
        // xs.sort_unstable_by(|a,b| {
            match (a.2.as_ref(),b.2.as_ref()) {

                (Some((_,a)),Some((_,b))) => a.score.cmp(&b.score).reverse(),

                (None,None)               => a.0.cmp(&b.0).reverse(),
                (a,b)                     => a.partial_cmp(&b).unwrap(),
            }
        });
        xs.reverse();
    }

}




