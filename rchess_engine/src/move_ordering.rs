
use crate::types::*;
use crate::tables::*;
use crate::trans_table::*;
use crate::searchstats::*;

use rayon::prelude::*;



// /// Sort Order:
// ///     TT Lookups sorted by score,
// ///     Captures sorted by MVV/LVA
// ///     Promotions // TODO:
// ///     rest
// pub fn order_searchinfo(maximizing: bool, mut xs: &mut [(Move,Game,Option<(SICanUse,SearchInfo)>)]) {
//     use std::cmp::Ordering;
//     if maximizing {
//         xs.par_sort_unstable_by(|(mv0,g0,msi0),(mv1,g1,msi1)| {
//             match (msi0,msi1) {
//                 (Some((_,si0)),Some((_,si1))) => si0.score.cmp(&si1.score).reverse(),
//                 (Some((_,si0)),None)          => Ordering::Less,
//                 (None,Some((_,si1)))          => Ordering::Greater,
//                 _                             => {
//                     _order_mvv_lva(mv0, mv1)
//                     // mv0.cmp(mv1)
//                 },
//             }
//         });
//     } else {
//         xs.par_sort_unstable_by(|(mv0,g0,msi0),(mv1,g1,msi1)| {
//             match (msi0,msi1) {
//                 (Some((_,si0)),Some((_,si1))) => si0.score.cmp(&si1.score),
//                 (Some((_,si0)),None)          => Ordering::Less,
//                 (None,Some((_,si1)))          => Ordering::Greater,
//                 _                             => {
//                     // _order_mvv_lva(mv0, mv1).reverse()
//                     _order_mvv_lva(mv0, mv1)
//                     // mv0.cmp(mv1).reverse()
//                 },
//             }
//         });
//     }
//     // xs.reverse()
// }

pub fn order_mvv_lva(mut xs: &mut [Move]) {
    use Move::*;
    use std::cmp::Ordering;
    xs.par_sort_unstable_by(|a,b| {
        _order_mvv_lva(a, b)
    });
}

pub fn _order_mvv_lva(a: &Move, b: &Move) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match (a.victim(), b.victim()) {
        (Some(v0), Some(v1)) => match (a.piece(),b.piece()) {
            (Some(pc0),Some(pc1)) => {
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

pub fn order_searchinfo(maximizing: bool, mut xs: &mut [(Move,Game,Option<(SICanUse,SearchInfo)>)]) {

    // #[cfg(feature = "par")]
    // xs.par_sort_unstable_by(|a,b| a.2.partial_cmp(&b.2).unwrap());
    // #[cfg(not(feature = "par"))]
    // xs.sort_unstable_by(|a,b| a.2.partial_cmp(&b.2).unwrap());
    // if !maximizing {
    //     xs.reverse();
    // }

    #[cfg(feature = "par")]
    {
        if maximizing {
            xs.par_sort_unstable_by(|a,b| {
                match (a.2.as_ref(),b.2.as_ref()) {
                    (Some((_,a)),Some((_,b))) => a.score.cmp(&b.score),
                    // (None,None)               => _order_mvv_lva(&a.0, &b.0),
                    (a,b)                     => a.partial_cmp(&b).unwrap(),
                }
            });
            xs.reverse();
        } else {
            xs.par_sort_unstable_by(|a,b| {
                match (a.2.as_ref(),b.2.as_ref()) {
                    (Some((_,a)),Some((_,b))) => a.score.cmp(&b.score).reverse(),
                    // (None,None)               => _order_mvv_lva(&a.0, &b.0),
                    // (a,b)                     => a.partial_cmp(&b).unwrap().reverse(),
                    (a,b)                     => a.partial_cmp(&b).unwrap(),
                }
            });
            xs.reverse();
        }
    }

    #[cfg(not(feature = "par"))]
    panic!("not par order_searchinfo2");

    // #[cfg(not(feature = "par"))]
    // {
    //     if maximizing {
    //         xs.sort_unstable_by(|a,b| {
    //             match (a.2.as_ref(),b.2.as_ref()) {
    //                 // (Some((_,a)),Some((_,b))) => a.partial_cmp(&b).unwrap(),
    //                 (Some((_,a)),Some((_,b))) => a.score.partial_cmp(&b.score).unwrap(),
    //                 _                         => a.partial_cmp(&b).unwrap(),
    //             }
    //         });
    //         xs.reverse();
    //     } else {
    //         xs.sort_unstable_by(|a,b| {
    //             match (a.2.as_ref(),b.2.as_ref()) {
    //                 (Some((_,a)),Some((_,b))) => a.score.partial_cmp(&b.score).unwrap().reverse(),
    //                 _                         => a.partial_cmp(&b).unwrap(),
    //             }
    //         });
    //         xs.reverse();
    //     }
    // }

}




