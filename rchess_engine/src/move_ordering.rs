
use crate::explore::*;
use crate::types::*;
use crate::tables::*;
use crate::trans_table::*;
use crate::searchstats::*;
use crate::evaluate::*;

use rayon::prelude::*;

use std::cmp::Ordering;

#[derive(Debug,Eq,PartialEq,Ord,PartialOrd,Clone,Copy)]
pub enum OrdMove {
    Hash,
    KillerMate,
    PromCapture,
    Prom,
    /// GoodCapture = QxQ (119), RxR (118), NxB (117), BxB (116), BxN (115) and pxp (114).
    GoodCapture,
    KillerMove1,
    KillerMove2,
    CaptureGoodSee(i8),
    // Capture(u8),
    CaptureEvenSee,
    Castle,
    PromMinor,
    CaptureBadSee(i8),
    OtherScore(i8),
    Other,
}

fn convert_score(s: Score) -> i8 {
    const K: Score = 16909320;
    (s / K) as i8
}

impl ExHelper {
    pub fn order_moves(
        &self,
        ts:           &Tables,
        g:            &Game,
        ply:          Depth,
        tracking:     &ExTracking,
        mut gs:       &mut [(Move,Zobrist,Option<(SICanUse,SearchInfo)>)],
    ) {
        gs.sort_by_cached_key(|x| {
            Self::order_score_move(ts, g, ply, tracking, x)
        });
    }

    pub fn order_score_move(
        ts:           &Tables,
        g:            &Game,
        ply:          Depth,
        tracking:     &ExTracking,
        (mv,zb,mtt):  &(Move,Zobrist,Option<(SICanUse,SearchInfo)>),
    ) -> OrdMove {
        use self::OrdMove::*;

        match mtt {
            Some((SICanUse::UseScore,_))     => return Hash,
            Some((SICanUse::UseOrdering,si)) => { // XXX: use lower-depth values for ordering?
                if si.node_type == Node::PV { return Hash; }
            },
            None                             => {},
        }
        // TODO: killer mate

        match mv {
            &Move::PromotionCapture { .. }            => return PromCapture,
            &Move::Promotion { new_piece: Queen, .. } => return Prom,
            &Move::Promotion { .. }                   => return PromMinor,

            &Move::EnPassant { .. }                   => return GoodCapture,
            &Move::Capture { pc, victim, .. } => match (pc,victim) {
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
                            return CaptureGoodSee((see / 1000).clamp(-127,127) as i8);
                        } else if see < 0 {
                            return CaptureBadSee((see / 1000).clamp(-127,127) as i8);
                        }
                    }
                },
            }
            &Move::Castle { .. } => return Castle,
            _                                 => {},
        }

        // TODO: killer move

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
        _                    => a.cmp(&b).reverse(),
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




