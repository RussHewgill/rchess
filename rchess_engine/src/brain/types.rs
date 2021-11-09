
use crate::tables::*;
use crate::types::*;
use crate::evaluate::*;

use crate::brain::filter::*;

use rand::{Rng,SeedableRng};
use rand::prelude::StdRng;
use rand::distributions::Uniform;

use nalgebra::{SMatrix,SVector,Matrix,Vector};

#[derive(Debug,Clone)]
pub struct GNetwork<T, const IS: usize, const HS: usize, const OS: usize> {
    pub n_hidden:         usize,

    pub weights_in:       SMatrix<T, HS, IS>,
    pub weights:          Vec<SMatrix<T, HS, HS>>,
    pub weights_out:      SMatrix<T, OS, HS>,

    pub biases_in:        SVector<T, HS>,
    pub biases:           Vec<SVector<T, HS>>,
    pub biases_out:       SVector<T, OS>,
}

// pub type Network = GNetwork<f32, 1, 1, 1>;
// pub type Network = GNetwork<f32, 2, 4, 1>;
pub type Network = GNetwork<f32, 2, 3, 1>;
// pub type Network = GNetwork<f32, 2, 3, 2>;
// pub type Network = GNetwork<f32, 3, 4, 1>;

pub type MNNetwork = GNetwork<f32, 784, 16, 10>;

pub fn test_mnist(
    n0:               &MNNetwork,
    mut test_imgs:    Vec<SVector<f32,784>>,
    mut tst_lbl:      Vec<u8>,
    n:                Option<usize>,
) {
    if let Some(n) = n {
        test_imgs.truncate(n);
        tst_lbl.truncate(n);
    }
    let mut out: Vec<(u8,(usize, f32))> = vec![];
    for (img,lbl) in test_imgs.iter().zip(tst_lbl.iter()) {
        // let input = SVector::<f32,784>::from_column_slice(img);
        let pred = n0.run(&img);
        let (k0,k1) = pred.iter().enumerate()
            .max_by(|a,b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap();
        out.push((*lbl,(k0,*k1)));
    }
    let score = out.iter().filter(|(lbl,(k,_))| *lbl as usize == *k)
        .collect::<Vec<_>>().len();
    eprintln!("score = {} / {}: {:.2}",
            score, out.len(), score as f32 / out.len() as f32);
}

impl<const IS: usize, const HS: usize, const OS: usize> GNetwork<f32,IS,HS,OS> {

    pub fn run(&self, input: &SVector<f32,IS>) -> SVector<f32,OS> {
        let (out,_,_) = self._run(input);
        out
    }

    pub fn _run(&self, input: &SVector<f32,IS>)
                -> (SVector<f32,OS>,SVector<f32,OS>,Vec<(SVector<f32,HS>,SVector<f32,HS>)>) {

        // let mut last = input.to_owned();
        let mut activations = vec![];

        let z = self.weights_in * input;
        let z = z + self.biases_in;
        let act = z.map(sigmoid);

        activations.push((act,z));

        let mut last: SVector<f32, HS> = act;

        for (ws,bs) in self.weights.iter().zip(self.biases.iter()) {
            // println!("wat 0");

            let z = ws * last;
            let z = z + bs;
            let act = z.map(sigmoid);

            activations.push((act.clone(),z));

            last = act;
        }

        let pred_z: SVector<f32, OS> = self.weights_out * last;
        let pred_z = pred_z + self.biases_out;
        let pred = pred_z.map(sigmoid);

        (pred,pred_z,activations)
        // unimplemented!()
    }

    #[allow(unused_doc_comments)]
    pub fn backprop_mut(
        &mut self,
        inputs:         Vec<SVector<f32,IS>>,
        corrects:       Vec<SVector<f32,OS>>,
        lr:             f32,
    ) {
        let ins = inputs.iter().zip(corrects.iter());

        let mut predictions = vec![];
        let mut activations = vec![];

        for (input,correct) in ins.clone() {
            let (pred,pred_z,acts) = self._run(input);
            predictions.push((pred,pred_z));
            activations.push(acts);
        }

        let outs = ins.zip(predictions.iter().zip(activations.iter()));

        // let cost: f32 = predictions.iter().zip(corrects.iter())
        //     .map(|(p,c)| (p - c).map(|x| x * x).sum())
        //     .sum();
        // let cost = cost / (predictions.len() as f32);

        // eprintln!("cost = {:?}", cost);

        let mut ws_new: Vec<(SMatrix<f32,HS,IS>,Vec<SMatrix<f32,HS,HS>>,SMatrix<f32,OS,HS>)> = vec![];
        let mut bs_new: Vec<(SVector<f32,HS>,Vec<SVector<f32,HS>>,SVector<f32,OS>)> = vec![];

        for ((input,correct),((pred,pred_z),acts)) in outs {

            // let correct: &SVector<f32,OS> = correct;
            // let pred:    &SVector<f32,OS> = pred;

            // let act1: SVector<f32,HS> = acts[acts.len()-1];

            // let dz_dw = act1;                    // HS,1
            let da_dz = pred.map(sigmoid_deriv); // OS,1
            let dc_da = 2.0 * (pred - correct);  // OS,1
            // let dc_da = (pred - correct).map(|x| x * x);  // OS,1

            // // da_dw
            // let x0 = dz_dw * da_dz.transpose(); // HS,OS
            // let dc_dw = x0 * dc_da;             // HS,1

            let error_pred = dc_da.component_mul(&da_dz); // OS,1

            let delta = pred - correct; // OS,1
            let delta = delta.component_mul(&pred_z.map(sigmoid_deriv));

            // eprintln!("error_pred = {:?}", error_pred);
            // eprintln!("delta = {:?}", delta);

            // if self.n_hidden <= 1 {
            if false {

                let (act,z) = acts[acts.len()-1];
                let w_out: SMatrix<f32,OS,HS> = delta * act.transpose();
                let sp = z.map(sigmoid_deriv);
                let delta2 = self.weights_out.transpose() * delta; // HS,1
                let delta2 = delta2.component_mul(&sp);
                let w_in: SMatrix<f32,HS,IS> = delta2 * input.transpose();

                ws_new.push((w_in,vec![],w_out));
                bs_new.push((delta2,vec![],delta));

                panic!();

            } else {

                let mut prev_error  = SVector::<f32,HS>::zeros();

                let mut ws = vec![];
                let mut bs = vec![];

                let mut w_out: Option<SMatrix<f32,OS,HS>> = None;
                let mut w_in: Option<SMatrix<f32,HS,IS>>  = None;

                let mut prev_delta = SVector::<f32,HS>::zeros();
                let mut delta2: Option<SVector<f32,HS>> = None;

                for k in 0..self.n_hidden+1 {
                    let layer = self.n_hidden - k;
                    // eprintln!("k, layer = {:?}, {:?}", k, layer);

                    if layer == 0 {
                        // println!("wat input: {}", layer);

                        // let d = self.weights[layer-1].transpose() * prev_delta;
                        // let d = d.component_mul(&sp);

                        w_in = Some(prev_delta * input.transpose());

                    } else if layer == self.n_hidden {
                        // println!("wat output: {}", layer);
                        let (act,z) = acts[acts.len()-1];

                        let sp = z.map(sigmoid_deriv);
                        let d = self.weights_out.transpose() * delta; // HS,1
                        let d = d.component_mul(&sp);
                        prev_delta = d;

                        w_out = Some(delta * act.transpose());
                    } else {
                        // println!("wat hidden: {}", layer);

                        let (_,z) = acts[layer];
                        let sp = z.map(sigmoid_deriv);

                        let d = self.weights[layer-1].transpose() * prev_delta;
                        let d = d.component_mul(&sp);
                        prev_delta = d;

                        let (act,_) = acts[layer-1];

                        let w = d * act.transpose();

                        ws.push(w);
                        bs.push(d);

                        // self.weights[layer-1] = self.weights[layer-1] - lr * x1;
                        // self.biases[layer-1]  = self.biases[layer-1] - error;

                    }
                }

                ws_new.push((w_in.unwrap(),ws,w_out.unwrap()));
                bs_new.push((prev_delta,bs,delta));

            }

        }

        let eta = lr / (ws_new.len() as f32 + 2.0);

        let nw0: SMatrix<f32,HS,IS> = ws_new.iter().map(|x| x.0).sum();
        self.weights_in  = self.weights_in - nw0 * eta;

        let nw2: SMatrix<f32,OS,HS> = ws_new.iter().map(|x| x.2).sum();
        self.weights_out = self.weights_out - nw2 * eta;

        for (mut ws, nws) in self.weights.iter_mut().zip(ws_new.into_iter().map(|x| x.1)) {
            let nw: SMatrix<f32,HS,HS> = nws.iter().sum();
            *ws = *ws - nw * eta;
        }

        // for (i,b) in bs_new.iter().map(|x| x.0).enumerate() {
        //     self.biases_in  = self.biases_in - b;
        // }

        let blen = 2.0 + bs_new.len() as f32;

        let nb0: SVector<f32,HS> = bs_new.iter().map(|x| x.0).sum();
        self.biases_in  = self.biases_in - nb0 / blen;
        let nb2: SVector<f32,OS> = bs_new.iter().map(|x| x.2).sum();
        self.biases_out = self.biases_out - nb2 / blen;

        for (mut bs, nbs) in self.biases.iter_mut().zip(bs_new.into_iter().map(|x| x.1)) {
            let nb: SVector<f32,HS> = nbs.iter().sum();
            *bs = *bs - nb / blen;
        }

    }

    // pub fn backprop_mut2(
    //     &mut self,
    //     inputs:         Vec<SVector<f32,IS>>,
    //     corrects:       Vec<SVector<f32,OS>>,
    // // ) -> SVector<f32,OS> {
    // ) {
    //     let ins = inputs.iter().zip(corrects.iter());

    //     let mut predictions = vec![];
    //     let mut activations = vec![];

    //     for (input,correct) in ins.clone() {
    //         let (pred, acts) = self._run(input);
    //         predictions.push(pred);
    //         activations.push(acts);
    //     }

    //     let outs = ins.zip(predictions.iter().zip(activations.iter()));

    //     for ((input,correct),(pred,acts)) in outs {
    //         // println!("wat 0");

    //         let mut acts = acts.iter().rev();
    //         let correct: &SVector<f32,OS> = correct;
    //         let pred: &SVector<f32,OS> = pred;

    //         let act1: &SVector<f32,HS> = acts.next().unwrap();

    //         let error = correct - pred; // OS,1
    //         let d_predicted: SVector<f32,OS> = error.component_mul(&pred.map(sigmoid_deriv)); // OS,1

    //         let loss: f32 = error.map(|x| x * x).sum();

    //         let lr = 0.1;

    //         let x0: SMatrix<f32,OS,HS> = (d_predicted * lr) * act1.transpose(); // OS,HS
    //         self.weights_out = self.weights_out + x0;
    //         self.biases_out = self.biases_out + d_predicted;

    //         let mut last_act = act1;
    //         for (mut ws,mut bs) in self.weights.iter_mut().zip(self.biases.iter_mut()) {

    //             let act = acts.next().unwrap();
    //             let d_act = act.map(sigmoid_deriv);

    //             let e_layer: SMatrix<f32,HS,HS> = ws.transpose() * loss;
    //             let d_layer = e_layer * d_act;

    //             // let x0 = (loss * lr) * act.transpose();
    //             // ws = ws + x0;

    //         }

    //         let e_input = self.weights_out.transpose() * d_predicted; // HS,1
    //         let d_input = e_input.component_mul(&act1.map(sigmoid_deriv)); // HS,1

    //         let x1 = input * d_input.transpose(); // IS,HS XXX: ??
    //         self.weights_in = self.weights_in + x1.transpose() * lr;
    //         self.biases_in = self.biases_in.map(|x| x + d_input.sum());

    //     }

    //     // unimplemented!()
    // }

}

pub fn sigmoid_deriv(x: f32) -> f32 {
    // x * (1.0 - x)
    sigmoid(x) * (1.0 - sigmoid(x))
}

pub fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + f32::exp(-x))
}

impl<const IS: usize, const HS: usize, const OS: usize> GNetwork<f32,IS,HS,OS> {

    pub fn new(n_hidden: usize) -> Self {
        Self::_new(n_hidden, (0.0, 1.0), Some(18105974836011991331))
    }

    pub fn new_range(n_hidden: usize, mm: (f32,f32)) -> Self {
        Self::_new(n_hidden, mm, Some(18105974836011991331))
    }

    pub fn _new(n_hidden: usize, mm: (f32,f32), seed: Option<u64>) -> Self {
        assert!(n_hidden > 0);

        let mut rng: StdRng = if let Some(seed) = seed {
            // SeedableRng::seed_from_u64(18105974836011991331)
            SeedableRng::seed_from_u64(seed)
        } else {
            // SeedableRng::seed_from_u64(18105974836011991331)
            let mut r = rand::thread_rng();
            SeedableRng::from_rng(r).unwrap()
        };

        // let dist = Uniform::new(0.0f32,1.0);
        let dist = Uniform::new(mm.0,mm.1);

        let weights_in = SMatrix::<f32,HS,IS>::from_distribution(&dist, &mut rng);
        let mut weights = vec![];
        (0..n_hidden-1).for_each(|x| {
            let a = SMatrix::<f32,HS,HS>::from_distribution(&dist, &mut rng);
            weights.push(a);
        });
        let weights_out = SMatrix::<f32,OS,HS>::from_distribution(&dist, &mut rng);

        let biases_in = SVector::<f32,HS>::from_distribution(&dist, &mut rng);
        let mut biases = vec![];
        (0..n_hidden-1).for_each(|x| {
            let a = SVector::<f32,HS>::from_distribution(&dist, &mut rng);
            biases.push(a);
        });
        let biases_out = SVector::<f32,OS>::from_distribution(&dist, &mut rng);

        Self {
            n_hidden,

            weights_in,
            weights,
            weights_out,

            biases_in,
            biases,
            biases_out,
        }
    }
}

mod network2 {

    use ndarray::prelude::*;
    use ndarray_rand::RandomExt;
    use ndarray_rand::rand_distr::Uniform;

    use rand::{Rng,SeedableRng};
    use rand::prelude::StdRng;

    #[derive(Debug,Clone)]
    pub struct Network2<T> {
        size_inputs:      usize,
        size_hidden:      usize,
        n_hidden:         usize,
        size_output:      usize,

        weights:          Vec<Array2<T>>,
        weights_out:      Array2<T>,
        biases:           Vec<Array1<T>>,
        biases_out:       Array1<T>,
    }

    impl Network2<f32> {

        pub fn run(&self, input: ArrayView1<f32>) -> Array1<f32> {
            let (out,_) = self._run(input);
            out
        }

        pub fn _run(&self, input: ArrayView1<f32>) -> (Array1<f32>,Vec<Array1<f32>>) {

            // let mut last = input.reversed_axes();
            let mut last = input.to_owned();
            let mut activations = vec![];

            // eprintln!("last.shape() = {:?}", last.shape());

            let mut k = 0;
            for (ws,bs) in self.weights.iter().zip(self.biases.iter()) {
                // eprintln!("ws.shape() = {:?}", ws.shape());
                // eprintln!("bs.shape() = {:?}", bs.shape());

                let act: Array1<f32> = ws.dot(&last);
                let act = act + bs;
                let act = act.map(|x| Self::sigmoid(*x));

                activations.push(act.clone());
                last = act;
            }

            let out = self.weights_out.dot(&last);
            let out = out + &self.biases_out;

            (out,activations)
        }

        pub fn backprop(
            &mut self,
            inputs:           Vec<Array1<f32>>,
            correct:          Vec<Array1<f32>>,
        ) -> Array1<f32> {

            // let error = correct - prediction;

            let ins = inputs.iter().zip(correct.iter());

            let mut predictions = vec![];
            let mut activations = vec![];

            for (input,correct) in ins.clone() {
                let (pred, acts) = self._run(input.view());
                predictions.push(pred);
                activations.push(acts);
            }

            let outs = ins.zip(predictions.iter().zip(activations.iter()));

            for ((input,correct),(pred,acts)) in outs {
                println!("wat 0");

                let error: Array1<f32> = correct - pred;
                // let error: Array1<f32> = error.map(|x| x * x);

                // let xs0     = acts.iter().rev();
                // let mut xs1 = self.weights

                println!("wat 1");
                let mut acts = acts.iter().rev();
                let act1 = acts.next().unwrap();

                let d_predicted: Array1<f32> = 2.0 * &error * pred.map(|x| Self::sigmoid_deriv(*x));

                // let x0: Array1<f32> = 2.0 * &error * pred.map(|x| Self::sigmoid_deriv(*x));

                // let ws2: Array1<f32> = act1.t().dot(&x0);
                // let ws2 = act1.t().dot(&x0);

                // let w = self.weights_out.t();
                // let ws1 = input.t().dot(
                //     &(x0.dot(&w) * act1.map(|x| Self::sigmoid_deriv(*x)))
                // );

                // let s0 = self.weights_out.shape();
                // let s1 = ws1.shape();
                // let s2 = ws2.shape();

                // eprintln!("s0 = {:?}", s0);
                // eprintln!("s1 = {:?}", s1);
                // eprintln!("s2 = {:?}", s2);

                // for act in acts.iter().rev() {
                //     // let act: ArrayView1<f32> = act.t();
                //     let ws = act.t().dot(
                //         &(2.0 * &error * pred.map(|x| Self::sigmoid_deriv(*x))));
                // }

            }

            unimplemented!()
        }

        fn sigmoid_deriv(x: f32) -> f32 {
            x * (1.0 - x)
        }

        fn sigmoid(x: f32) -> f32 {
            1.0 / (1.0 + f32::exp(-x))
        }

    }

    impl Network2<f32> {
        pub fn new(
            size_inputs:      usize,
            size_hidden:      usize,
            n_hidden:         usize,
            size_output:      usize,
        ) -> Self {
            let mut rng: StdRng = SeedableRng::seed_from_u64(18105974836011991331);

            let mut weights = vec![
                Array2::random_using(
                    (size_hidden,size_inputs), Uniform::new(0.0,1.0), &mut rng)
            ];

            (0..n_hidden+1).for_each(|x| {
                let a = Array2::random_using(
                    (size_hidden, size_hidden), Uniform::new(0.0,1.0), &mut rng);
                weights.push(a);
            });

            let weights_out = Array2::random_using(
                (size_output,size_hidden), Uniform::new(0.0,1.0), &mut rng);

            let mut biases: Vec<Array1<f32>> = vec![];
            (0..n_hidden).for_each(|x| {
                let a = Array1::random_using(
                    size_hidden, Uniform::new(0.0,1.0), &mut rng);
                biases.push(a);
            });
            let biases_out = Array1::random_using(
                size_output, Uniform::new(0.0,1.0), &mut rng);

            Self {
                size_inputs,
                size_hidden,
                n_hidden,
                size_output,
                weights,
                weights_out,
                biases,
                biases_out,
            }
        }
    }

}


// pub struct ConvNetwork {
//     filters:      Vec<ConvFilter>,
// }


// impl ConvNetwork {

//     pub fn run(&self, ts: &Tables, g: &Game) -> Score {
//         let bbs = Self::split_bitboards(g);

//         let mut conv_layer = Array3::zeros((6,6,0));
//         for filt in self.filters.iter() {
//             let x = filt.scan_bitboard(&bbs);
//             let x = x.insert_axis(Axis(2));
//             conv_layer.append(Axis(2), x.view()).unwrap();
//         }

//         unimplemented!()
//     }

// }

// impl ConvNetwork {

//     /// 0-6:      own pieces
//     /// 6-12:     other pieces
//     fn split_bitboards(g: &Game) -> Vec<BitBoard> {
//         let mut out = vec![];

//         let sides = if g.state.side_to_move == White { [White,Black] } else { [Black,White] };

//         for side in sides {
//             for pc in Piece::iter_pieces() {
//                 out.push(g.get(pc, side));
//             }
//         }

//         out
//     }

// }






