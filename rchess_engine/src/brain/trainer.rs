
use crate::explore::*;
use crate::opening_book::*;
use crate::tables::*;
use crate::texel::TxPosition;
use crate::types::*;
use crate::evaluate::*;
use crate::alphabeta::*;
// use crate::brain::types::*;
use crate::brain::types::nnue::*;

pub use crate::brain::gensfen::*;

use rayon::prelude::*;

use ndarray as nd;
use nd::{Array2};
use ndarray::linalg::Dot;

use rand::{prelude::{StdRng,SliceRandom},Rng,SeedableRng};
use rand::distributions::{Uniform,uniform::SampleUniform};

pub struct NNTrainingParams {
    pub eta:   i8,
}

/// Train
impl NNUE {

    pub fn train(&mut self, tds: Vec<TrainingData>) {

        // self.init_inputs(g)

        unimplemented!()
    }

    pub fn train_single(
        &mut self,
        ts:             &Tables,
        params:         &NNTrainingParams,
        mut rng:        &mut StdRng,
        td:             &TrainingData,
    ) {
        let mut g = if let Some(g) = td.init_opening(&ts) { g } else { return; };

        self.init_inputs(&g);

        for te in td.moves.iter() {
            if let Ok(g2) = g.make_move_unchecked(&ts, te.mv) {
                g = g2;
                self.update_move(&g, false);
            } else { break; }

            let k: u8 = rng.gen_range(0..4);
            if k == 0 {
                self.backprop(None, te.eval, params.eta);
            }
        }

        unimplemented!()
    }
}

/// Error
impl NNUE {

    pub fn mean_sq_error(
        &self,
        ts:             &Tables,
        tds:            &[TrainingData],
    ) -> f64 {

        let tds2 = tds.chunks(tds.len() / num_cpus::get())
            .map(|xs| (self.clone(), xs)).collect::<Vec<_>>();

        let error: f64 = tds2.par_iter().map(|(nn, xs)| {
            let mut nn = nn.clone();
            let e: f64 = xs.iter().flat_map(|td| {
                let mut g = if let Some(g) = td.init_opening(&ts) { g } else { return None; };
                nn.init_inputs(&g);

                // let mut errs = vec![];

                for te in td.moves.iter() {
                    if let Ok(g2) = g.make_move_unchecked(&ts, te.mv) {
                        g = g2;
                        let eval = nn.update_move(&g, true).unwrap();

                        // errs.push((te.eval - eval).pow)

                    } else { break; }
                }

                Some(1.0)
            }).sum();
            e
        }).sum();

        unimplemented!()
    }

}

/// Backprop
impl NNUE {

    #[allow(unused_doc_comments)]
    pub fn backprop(&mut self, g: Option<&Game>, correct: i32, eta: i8) -> i32 {

        if let Some(g) = g {
            self.init_inputs(g);
        }

        let (pred, ((act1,act2,act3),(z1,z2,z3,z_out))) = self._run_partial();

        /// XXX: No activation function for last layer, so no derivative
        let delta_out = pred - correct;
        let delta_out: Array2<i8> = nd::array![[delta_out.clamp(-127,127) as i8]];

        /// L4
        let ws4 = delta_out.dot(&act3.t()); // 1,32
        let bs4 = delta_out.clone();

        /// L3
        let sp3 = z3.map(Self::act_d);

        let mut d3 = self.weights_4.t().dot(&delta_out); // 32,1
        d3 *= &sp3;

        let ws3 = d3.dot(&act2.t()); // 32,32
        let bs3 = d3.clone();        // 1,1

        /// L2
        let sp2 = z2.map(Self::act_d);

        let mut d2 = self.weights_3.t().dot(&d3); // 32,1
        d2 *= &sp2;

        let ws2 = d2.dot(&act1.t()); // 32,512
        let bs2 = d2.clone();        // 32,1

        let sp1_own: Array2<i16> = self.activations_own.map(Self::act_d); // 256,1
        let sp1_own = sp1_own.map(|x| (*x).clamp(-127, 127) as i8);       // 256,1

        let sp1_other: Array2<i16> = self.activations_other.map(Self::act_d); // 256,1
        let sp1_other = sp1_other.map(|x| (*x).clamp(-127, 127) as i8);       // 256,1

        let d1 = self.weights_2.t().dot(&d2); // 512,1
        let d1_own0: nd::ArrayView2<i8> = d1.slice(nd::s![..256, ..]); // 256, 1
        let d1_own0 = &d1_own0 * &sp1_own;

        let d1_other0: nd::ArrayView2<i8> = d1.slice(nd::s![256.., ..]); // 256, 1
        let d1_other0 = &d1_other0 * &sp1_other;

        let d1_own  = sprs::CsMat::csc_from_dense(d1_own0.view(), 0);
        let d1_other = sprs::CsMat::csc_from_dense(d1_other0.view(), 0);

        let ws1_own = &d1_own * &self.inputs_own.transpose_view();
        let ws1_other = &d1_other * &self.inputs_other.transpose_view();

        self.biases_1_own   -= &d1_own0.map(|x| *x as i16);
        self.biases_1_other -= &d1_other0.map(|x| *x as i16);

        // self.weights_1_own   -= &(ws1_own / eta);
        // self.weights_1_other -= &(ws1_other / eta);

        // println!("wat 0 in {:.3} seconds", t0.elapsed().as_secs_f64());
        // let t0 = std::time::Instant::now();
        // println!("wat 0 in {:.3} seconds", t0.elapsed().as_secs_f64());

        for (c,cv) in ws1_own.outer_iterator().enumerate() {
            for (r,rv) in cv.iter() {
                self.weights_1_own[(r,c)] -= rv / eta;
            }
        }
        for (c,cv) in ws1_other.outer_iterator().enumerate() {
            for (r,rv) in cv.iter() {
                self.weights_1_other[(r,c)] -= rv / eta;
            }
        }

        self.weights_2 -= &(ws2 / eta);
        self.weights_3 -= &(ws3 / eta);
        self.weights_4 -= &(ws4 / eta);

        self.biases_2 -= &bs2.map(|x| *x as i32);
        self.biases_3 -= &bs3.map(|x| *x as i32);
        self.biases_4 -= &bs4.map(|x| *x as i32);

        pred
    }

}

