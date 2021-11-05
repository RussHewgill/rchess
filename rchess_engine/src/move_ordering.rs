
use crate::types::*;
use crate::tables::*;
use crate::trans_table::*;
use crate::searchstats::*;
use crate::evaluate::*;

use rayon::prelude::*;

use std::cmp::Ordering;

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

    // match (a.victim(), b.piece()) {
    //     (Some(victim), Some(attacker)) => {
    //         attacker.cmp(&victim)
    //     },
    //     (None,_) => a.cmp(&b).reverse(),
    //     (x,y) => panic!("wot mvv_lva: ({:?},{:?}) {:?}, {:?}", x, y, a, b)
    // }

    // a.cmp(&b).reverse()

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

pub fn order_searchinfo(mut xs: &mut [(Move,Game,Option<(SICanUse,SearchInfo)>)]) {

    // #[cfg(feature = "par")]
    // xs.par_sort_unstable_by(|a,b| a.2.partial_cmp(&b.2).unwrap());
    // #[cfg(not(feature = "par"))]
    // xs.sort_unstable_by(|a,b| a.2.partial_cmp(&b.2).unwrap());
    // if !maximizing {
    //     xs.reverse();
    // }

    // #[cfg(feature = "par")]
    // {
    //     if maximizing {
    //         // xs.par_sort_unstable_by(|a,b| {
    //         xs.par_sort_by(|a,b| {
    //             match (a.2.as_ref(),b.2.as_ref()) {
    //                 (Some((_,a)),Some((_,b))) => a.score.cmp(&b.score),
    //                 // (None,None)               => _order_mvv_lva(&a.0, &b.0),
    //                 (None,None)               => a.0.cmp(&b.0),
    //                 (a,b)                     => a.partial_cmp(&b).unwrap(),
    //                 // _                         => std::cmp::Ordering::Equal,
    //             }
    //         });
    //         xs.reverse();
    //     } else {
    //         // xs.par_sort_unstable_by(|a,b| {
    //         xs.par_sort_by(|a,b| {
    //             match (a.2.as_ref(),b.2.as_ref()) {
    //                 (Some((_,a)),Some((_,b))) => a.score.cmp(&b.score).reverse(),
    //                 // (None,None)               => _order_mvv_lva(&a.0, &b.0).reverse(),
    //                 (None,None)               => a.0.cmp(&b.0).reverse(),
    //                 (a,b)                     => a.partial_cmp(&b).unwrap(),
    //                 // _                         => std::cmp::Ordering::Equal,
    //             }
    //         });
    //         xs.reverse();
    //     }
    // }

    #[cfg(not(feature = "par"))]
    {
        xs.sort_by(|a,b| {
        // xs.sort_unstable_by(|a,b| {
            match (a.2.as_ref(),b.2.as_ref()) {

                (Some((_,a)),Some((_,b))) => a.score.cmp(&b.score).reverse(),

                // TODO: 
                // (Some((_,a)),Some((_,b))) => {
                //     match (a.node_type,b.node_type) {
                //         (Node::PV, Node::PV) => Ordering::Equal,
                //         (Node::PV, _)        => Ordering::Less,
                //         (_, Node::PV)        => Ordering::Greater,
                //         _                    => a.score.cmp(&b.score).reverse()
                //     }
                // },

                (None,None)               => a.0.cmp(&b.0).reverse(),
                (a,b)                     => a.partial_cmp(&b).unwrap(),
            }
        });
        xs.reverse();
    }

    // let maximizing = false;
    // #[cfg(not(feature = "par"))]
    // {
    //     if maximizing {
    //         // xs.par_sort_unstable_by(|a,b| {
    //         xs.sort_by(|a,b| {
    //             match (a.2.as_ref(),b.2.as_ref()) {
    //                 (Some((_,a)),Some((_,b))) => a.score.cmp(&b.score),
    //                 (None,None)               => a.0.cmp(&b.0),
    //                 (a,b)                     => a.partial_cmp(&b).unwrap(),
    //             }
    //         });
    //         xs.reverse();
    //     } else {
    //         // xs.par_sort_unstable_by(|a,b| {
    //         xs.sort_by(|a,b| {
    //             match (a.2.as_ref(),b.2.as_ref()) {
    //                 (Some((_,a)),Some((_,b))) => a.score.cmp(&b.score).reverse(),
    //                 (None,None)               => a.0.cmp(&b.0).reverse(),
    //                 (a,b)                     => a.partial_cmp(&b).unwrap(),
    //             }
    //         });
    //         xs.reverse();
    //     }
    // }

}




