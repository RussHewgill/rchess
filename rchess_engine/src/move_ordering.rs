
use crate::types::*;
use crate::tables::*;
use crate::trans_table::*;
use crate::searchstats::*;

use rayon::prelude::*;



/// Sort Order:
///     TT Lookups sorted by score,
///     Captures sorted by MVV/LVA
///     Promotions
///     rest
pub fn order_searchinfo(
    maximizing: bool, mut xs: &mut [(Move,Game,Option<(SICanUse,SearchInfo)>)])
{

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
                    (Some((_,a)),Some((_,b))) => a.score.partial_cmp(&b.score).unwrap(),
                    (a,b)                     => a.partial_cmp(&b).unwrap(),
                }
            });
            xs.reverse();
        } else {
            xs.par_sort_unstable_by(|a,b| {
                match (a.2.as_ref(),b.2.as_ref()) {
                    (Some((_,a)),Some((_,b))) => a.score.partial_cmp(&b.score).unwrap().reverse(),
                    (a,b)                     => a.partial_cmp(&b).unwrap(),
                }
            });
            xs.reverse();
        }
    }

    #[cfg(not(feature = "par"))]
    {
        if maximizing {
            xs.sort_unstable_by(|a,b| {
                match (a.2.as_ref(),b.2.as_ref()) {
                    // (Some((_,a)),Some((_,b))) => a.partial_cmp(&b).unwrap(),
                    (Some((_,a)),Some((_,b))) => a.score.partial_cmp(&b.score).unwrap(),
                    _                         => a.partial_cmp(&b).unwrap(),
                }
            });
            xs.reverse();
        } else {
            xs.sort_unstable_by(|a,b| {
                match (a.2.as_ref(),b.2.as_ref()) {
                    (Some((_,a)),Some((_,b))) => a.score.partial_cmp(&b.score).unwrap().reverse(),
                    _                         => a.partial_cmp(&b).unwrap(),
                }
            });
            xs.reverse();
        }
    }

}




