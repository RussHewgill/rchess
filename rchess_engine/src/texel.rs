
use std::path::Path;

use crate::brain::trainer::TDOutcome;
use crate::explore::ExHelper;
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
use crate::qsearch::*;
use crate::searchstats::*;
use crate::pawn_hash_table::*;

use derive_new::new;

#[derive(Debug,PartialEq,Clone,new)]
pub struct TxPosition {
    pub game:     Game,
    pub result:   TDOutcome,
    // pub q_score:  Score,
}

pub fn load_txdata<P: AsRef<Path>>(
    ts:             &Tables,
    mut exhelper:   &mut ExHelper,
    count:          Option<usize>,
    path:           P,
) -> std::io::Result<Vec<TxPosition>> {
    use crate::brain::gensfen::TrainingData;
    let tds: Vec<TrainingData> = TrainingData::load_all(path)?;

    let mut stats = SearchStats::default();

    let mut ps: Vec<TxPosition> = vec![];
    let mut n = 0;
    // let mut non_q = 0;

    for td in tds.into_iter() {
        let mut g = Game::from_fen(&ts, STARTPOS).unwrap();
        for mv in td.opening.into_iter() {
            g = g.make_move_unchecked(&ts, mv).unwrap();
        }

        for te in td.moves.into_iter() {
            if let Ok(g2) = g.make_move_unchecked(&ts, te.mv) {
                g = g2;
                if !te.skip
                    && !te.mv.filter_all_captures()
                    && g.state.checkers.is_empty()
                    && te.eval.abs() < STALEMATE_VALUE - 100
                {
                    let (ev_mid,ev_end) = (&exhelper.cfg.eval_params_mid,&exhelper.cfg.eval_params_end);
                    let score   = g.sum_evaluate(&ts, &ev_mid, &ev_end, None);
                    let q_score = exhelper.qsearch_once(&ts, &g, &mut stats);
                    let q_score = g.state.side_to_move.fold(q_score, -q_score);

                    if score == q_score {
                        ps.push(TxPosition::new(g.clone(), td.result));
                    } else {
                        // non_q += 1;
                    }
                }
            }

        }

        n += 1;
        if count.map_or(false, |c| n >= c) { break; }
    }

    Ok(ps)
}

pub fn texel_optimize_once(
    ts:                          &Tables,
    inputs:                      &[TxPosition],
    mut exhelper:                &mut ExHelper,
    ignore_weights:              &[bool],
    count:                       Option<usize>,
    mid:                         bool,
    mut best_error:              &mut f64,
    k:                           Option<f64>,
    delta:                       Score,
) {
    let mut arr = if mid {
        exhelper.cfg.eval_params_mid.to_arr()
    } else {
        exhelper.cfg.eval_params_end.to_arr()
    };

    for n in 0..arr.len() {
        if let Some(true) = ignore_weights.get(n) {
            continue;
        }

        arr[n] = arr[n].checked_add(1).unwrap();
        if mid {
            exhelper.cfg.eval_params_mid = EvalParams::from_arr(&arr);
        } else {
            exhelper.cfg.eval_params_end = EvalParams::from_arr(&arr);
        }
        exhelper.ph_rw.purge_scores();

        let new_error = average_eval_error(ts, &inputs, &exhelper, None);

        if new_error < *best_error {
            *best_error = new_error;
        } else {
            arr[n] = arr[n].checked_sub(2).unwrap();
            if mid {
                exhelper.cfg.eval_params_mid = EvalParams::from_arr(&arr);
            } else {
                exhelper.cfg.eval_params_end = EvalParams::from_arr(&arr);
            }
            exhelper.ph_rw.purge_scores();

            let new_error = average_eval_error(ts, &inputs, &exhelper, None);

            if new_error < *best_error {
                *best_error = new_error;
            } else {
                arr[n] = arr[n].checked_add(1).unwrap();
                if mid {
                    exhelper.cfg.eval_params_mid = EvalParams::from_arr(&arr);
                } else {
                    exhelper.cfg.eval_params_end = EvalParams::from_arr(&arr);
                }
                exhelper.ph_rw.purge_scores();
            }
        }

        // let t1 = t0.elapsed().as_secs_f64();
        // println!("n {} in {:.3} seconds, {:.1}", n, t1, n as f64 / t1);
    }

}

pub fn texel_optimize(
    ts:                          &Tables,
    inputs:                      &[TxPosition],
    mut exhelper:                &mut ExHelper,
    ignore_weights:              &[bool],
    count:                       Option<usize>,
    k:                           Option<f64>,
    path:                        &str,
) -> (EvalParams,EvalParams) {

    let mut best_error = average_eval_error(ts, &inputs, &exhelper, k);

    let arr_len = exhelper.cfg.eval_params_mid.to_arr().len();

    EvalParams::save_evparams(&exhelper.cfg.eval_params_mid, &exhelper.cfg.eval_params_end, path)
        .unwrap();

    let mut delta = 1;

    println!("starting texel_optimize...");
    // eprintln!("arr_mid.len() = {:?}", arr_mid.len());
    let t0 = std::time::Instant::now();
    let mut loops = 0;
    loop {
        let t1 = std::time::Instant::now();

        texel_optimize_once(
            ts, inputs, &mut exhelper, ignore_weights, count, true, &mut best_error, k, delta);

        texel_optimize_once(
            ts, inputs, &mut exhelper, ignore_weights, count, false, &mut best_error, k, delta);

        EvalParams::save_evparams(&exhelper.cfg.eval_params_mid, &exhelper.cfg.eval_params_end, path)
            .unwrap();

        loops += 1;
        let t2 = t1.elapsed().as_secs_f64();
        eprintln!("loops = {:>3}, best_error = {:.3}, time: {:.1}s / {:.1}s, {:.2} inputs/weights/s",
                  loops, best_error,
                  t2, t0.elapsed().as_secs_f64(),
                  inputs.len() as f64 / (arr_len * 2) as f64 / t2,
        );
        if let Some(c) = count { if loops >= c { break; } }
    }

    // (ev_mid,ev_end)
    (exhelper.cfg.eval_params_mid,exhelper.cfg.eval_params_end)
}

pub fn find_k(
    ts:         &Tables,
    inputs:     &[TxPosition],
    exhelper:   &ExHelper,
    print:      bool,
) -> f64 {
    let mut start = 1.0;
    let mut end   = 10.0;
    let mut step  = 1.0;

    let mut curr = start;
    let mut best = average_eval_error(&ts, inputs, exhelper, Some(curr));

    const PREC: usize = 10;
    for i in 0..PREC {

        curr = start - step;

        let mut err = 100000.0;

        while curr < end {
            curr = curr + step;

            err = average_eval_error(&ts, inputs, exhelper, Some(curr));

            if err < best {
                best  = err;
                start = curr;
            }

        }

        if print {
            eprintln!("best k {:.3} on iter {}, err = {:.3}", start, i, err);
        }

        end = start + step;
        start = start - step;
        step = step / 10.0;
    }

    start
}

pub fn average_eval_error(
    ts:         &Tables,
    inputs:     &[TxPosition],
    exhelper:   &ExHelper,
    k:          Option<f64>,
) -> f64 {
    const K: f64 = 1.0;
    let k = k.unwrap_or(K);

    // const K: f64 = 1.0;

    fn sigmoid(s: f64, k: f64) -> f64 {
        1.0 / (1.0 + 10.0f64.powf(-k * s / 400.0))
    }

    let (alpha,beta) = (i32::MIN,i32::MAX);
    let (alpha,beta) = (alpha + 200,beta - 200);
    let mut stats = SearchStats::default();

    // let ev = if mid {
    //     &exhelper.cfg.eval_params_mid
    // } else {
    //     &exhelper.cfg.eval_params_end
    // };

    use rayon::prelude::*;

    let ev_mid = &exhelper.cfg.eval_params_mid;
    let ev_end = &exhelper.cfg.eval_params_end;

    let ph_rw  = &exhelper.ph_rw;

    // let n_cpus = num_cpus::get();
    let n_cpus = num_cpus::get_physical();

    // for (n,xs) in cs.clone().enumerate() {
    //     eprintln!("n = {:?}, xs.len() = {:?}", n, xs.len());
    // }

    let sum: f64 = inputs.par_iter().map(move |pos| {
        let r = match pos.result {
            TDOutcome::Win(White) => 1.0,
            TDOutcome::Win(Black) => 0.0,
            TDOutcome::Draw       => 0.5,
            TDOutcome::Stalemate  => 0.5,
        };
        // let ph_rw = PHTableFactory::new();
        // let ph2 = ph_rw.handle();
        // let ph2 = exhelper.ph_rw.clone();
        let score = pos.game.sum_evaluate(
            ts, &ev_mid, &ev_end,
            // Some(&ph2),
            // Some(&exhelper.ph_rw)
            None,
        );
        (r - sigmoid(score as f64, k)).powi(2)
    }).sum();

    // let cs = inputs.chunks(inputs.len() / n_cpus);
    // let sum: f64 = crossbeam::scope(|s| {
    //     let ph_rw = exhelper.ph_rw.clone();
    //     let mut hs = vec![];
    //     for xs in cs {
    //         let ph2 = ph_rw.clone();
    //         let h = s.spawn(move |_| {
    //             xs.iter().map(|pos| {
    //                 let r = match pos.result {
    //                     TDOutcome::Win(White) => 1.0,
    //                     TDOutcome::Win(Black) => 0.0,
    //                     TDOutcome::Draw       => 0.5,
    //                     TDOutcome::Stalemate  => 0.5,
    //                 };
    //                 let score = pos.game.sum_evaluate(
    //                     ts, &ev_mid, &ev_end,
    //                     Some(&ph2),
    //                     // None,
    //                 );
    //                 (r - sigmoid(score as f64, k)).powi(2)
    //             }).sum::<f64>()
    //         });
    //         hs.push(h);
    //     }
    //     hs.into_iter().map(|h| {
    //         h.join().unwrap()
    //     }).sum()
    // }).unwrap();

    // let sum: f64 = inputs.iter().map(|pos| {
    // // let sum: f64 = inputs.par_iter().map(|pos| {
    //     let r = match pos.result {
    //         TDOutcome::Win(White) => 1.0,
    //         TDOutcome::Win(Black) => 0.0,
    //         TDOutcome::Draw       => 0.5,
    //         TDOutcome::Stalemate  => 0.5,
    //     };
    //     // let q_score = qsearch_once(&ts, &pos.game, pos.game.state.side_to_move, &ev_mid, &ev_end, ph_rw);
    //     // exhelper.game = pos.game.clone();
    //     // exhelper.side = pos.game.state.side_to_move;

    //     // let q_score = exhelper.qsearch(ts, &pos.game, (0,0), (alpha, beta), &mut stats);
    //     // let q_score = pos.game.state.side_to_move.fold(q_score, -q_score);

    //     let score = pos.game.sum_evaluate(
    //         ts, &ev_mid, &ev_end,
    //         // Some(&exhelper.ph_rw.clone()),
    //         // Some(&x),
    //         Some(&ph_rw),
    //         // None,
    //     );

    //     // assert_eq!(score, q_score);

    //     (r - sigmoid(score as f64, k)).powi(2)
    // }).sum();

    sum / inputs.len() as f64

    // unimplemented!()
}

