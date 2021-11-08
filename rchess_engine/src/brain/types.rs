
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
// pub type Network = GNetwork<f32, 2, 3, 1>;
pub type Network = GNetwork<f32, 2, 3, 2>;
// pub type Network = GNetwork<f32, 3, 4, 1>;

impl<const IS: usize, const HS: usize, const OS: usize> GNetwork<f32,IS,HS,OS> {

    pub fn run(&self, input: &SVector<f32,IS>) -> SVector<f32,OS> {
        let (out,_) = self._run(input);
        out
    }

    pub fn _run(&self, input: &SVector<f32,IS>) -> (SVector<f32,OS>,Vec<SVector<f32,HS>>) {

        // let mut last = input.to_owned();
        let mut activations = vec![];

        let act = self.weights_in * input;
        let act = act + self.biases_in;
        let act = act.map(sigmoid);

        activations.push(act);
        let mut last: SVector<f32, HS> = act;

        let mut k = 0;
        for (ws,bs) in self.weights.iter().zip(self.biases.iter()) {
            // println!("wat 0");

            let act = ws * last;
            let act = act + bs;
            let act = act.map(sigmoid);

            activations.push(act.clone());
            last = act;
        }

        let pred: SVector<f32, OS> = self.weights_out * last;
        let pred = pred + self.biases_out;
        let pred = pred.map(sigmoid);

        (pred,activations)
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
            let (pred, acts) = self._run(input);
            predictions.push(pred);
            activations.push(acts);
        }

        let outs = ins.zip(predictions.iter().zip(activations.iter()));

        let cost: f32 = predictions.iter().zip(corrects.iter())
            .map(|(p,c)| (p - c).map(|x| x * x).sum())
            .sum();
        let cost = cost / (predictions.len() as f32);

        for ((input,correct),(pred,acts)) in outs {

            let correct: &SVector<f32,OS> = correct;
            let pred:    &SVector<f32,OS> = pred;

            // let mut acts = acts.iter().rev();
            // let act1: &SVector<f32,HS> = acts.next().unwrap();
            let act1: SVector<f32,HS> = acts[acts.len()-1];

            // let error = pred - correct;           // OS,1
            // let dyh_dv = pred.map(sigmoid_deriv); // OS,1
            // let dc_dv = error.component_mul(&dyh_dv); // OS,1

            let dz_dw = act1;                    // HS,1
            let da_dz = pred.map(sigmoid_deriv); // OS,1
            let dc_da = 2.0 * (pred - correct);  // OS,1

            // let dc_dz = dc_da * da_dz.transpose();

            // da_dw
            let x0 = dz_dw * da_dz.transpose(); // HS,OS
            let dc_dw = x0 * dc_da;             // HS,1

            // eprintln!("dc_dz.shape() = {:?}", dc_dz.shape());

            // let error_pred = pred - correct; // OS,1

            let error_pred = dc_da.component_mul(&da_dz); // OS,1

            // let error_l = ws.t() * error_lplus1 (*) sig' z_l

            for layer in 0..self.weights.len()+2 {
                eprintln!("layer = {:?}", layer);

                if layer == 0 {
                    // input

                } else if layer == self.weights.len()+1 {
                    // output

                    // let error = self.weights_out.transpose() * error_pred;      // HS,1
                    // let error = error.component_mul(&act1.map(sigmoid_deriv));  // HS,1

                    let act_lminus1 = acts[acts.len()-1]; // HS,1

                    // eprintln!("error_pred.shape() = {:?}", error_pred.shape());
                    // eprintln!("error.act_lminus1() = {:?}", act_lminus1.shape());

                    let x1 = act_lminus1 * error_pred.transpose();
                    eprintln!("x1.shape() = {:?}", x1.shape());

                    // OS,HS
                    self.weights_out = self.weights_out - lr * x1.transpose();
                    self.biases_out = self.biases_out - error_pred;

                } else {
                    // hidden

                }
            }

            // let error = self.weights_out.transpose() * error_pred;      // HS,1
            // let error = error.component_mul(&act1.map(sigmoid_deriv));  // HS,1

            // eprintln!("error.shape() = {:?}", error.shape());

            // let error = ( self.weights_out * error_pred ).component_mul(act1.map(sigmoid_deriv))

            // let grad_l = act_lminus1 * error_l;

            // let x1 = 

            // // OS,HS
            // self.weights_out = self.weights_out - lr * x1;

            // // w_new = w_old - lr * dc_dv;
            // self.weights_out = self.weights_out - lr * 

        }

    }

    // fn train_layer(
    //     input:      SVector<f32,IS>,
    //     pred:       SVector<f32,OS>,
    //     correct:    SVector<f32,OS>,
    //     mut ws: &mut SMatrix<f32,HS,HS>,
    //     mut bs: &mut SVector<f32,HS>,
    // ) {
    //     unimplemented!()
    // }

    pub fn backprop_mut2(
        &mut self,
        inputs:         Vec<SVector<f32,IS>>,
        corrects:       Vec<SVector<f32,OS>>,
    // ) -> SVector<f32,OS> {
    ) {
        let ins = inputs.iter().zip(corrects.iter());

        let mut predictions = vec![];
        let mut activations = vec![];

        for (input,correct) in ins.clone() {
            let (pred, acts) = self._run(input);
            predictions.push(pred);
            activations.push(acts);
        }

        let outs = ins.zip(predictions.iter().zip(activations.iter()));

        for ((input,correct),(pred,acts)) in outs {
            // println!("wat 0");

            let mut acts = acts.iter().rev();
            let correct: &SVector<f32,OS> = correct;
            let pred: &SVector<f32,OS> = pred;

            let act1: &SVector<f32,HS> = acts.next().unwrap();

            let error = correct - pred; // OS,1
            let d_predicted: SVector<f32,OS> = error.component_mul(&pred.map(sigmoid_deriv)); // OS,1

            let loss: f32 = error.map(|x| x * x).sum();

            let lr = 0.1;

            let x0: SMatrix<f32,OS,HS> = (d_predicted * lr) * act1.transpose(); // OS,HS
            self.weights_out = self.weights_out + x0;
            self.biases_out = self.biases_out + d_predicted;

            let mut last_act = act1;
            for (mut ws,mut bs) in self.weights.iter_mut().zip(self.biases.iter_mut()) {

                let act = acts.next().unwrap();
                let d_act = act.map(sigmoid_deriv);

                let e_layer: SMatrix<f32,HS,HS> = ws.transpose() * loss;
                let d_layer = e_layer * d_act;

                // let x0 = (loss * lr) * act.transpose();
                // ws = ws + x0;

            }

            let e_input = self.weights_out.transpose() * d_predicted; // HS,1
            let d_input = e_input.component_mul(&act1.map(sigmoid_deriv)); // HS,1

            let x1 = input * d_input.transpose(); // IS,HS XXX: ??
            self.weights_in = self.weights_in + x1.transpose() * lr;
            self.biases_in = self.biases_in.map(|x| x + d_input.sum());

        }

        // unimplemented!()
    }

}

pub fn sigmoid_deriv(x: f32) -> f32 {
    x * (1.0 - x)
}

pub fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + f32::exp(-x))
}

impl<const IS: usize, const HS: usize, const OS: usize> GNetwork<f32,IS,HS,OS> {
    pub fn new(n_hidden: usize) -> Self {
        let mut rng: StdRng = SeedableRng::seed_from_u64(18105974836011991331);
        let dist = Uniform::new(0.0f32,1.0);

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

// impl Network<f32> {
//     pub fn new(
//         size_inputs:      usize,
//         size_hidden:      usize,
//         n_hidden:         usize,
//         size_output:      usize,
//     ) -> Self {
//         let mut rng: StdRng = SeedableRng::seed_from_u64(18105974836011991331);

//         let mut weights = vec![
//             Array2::random_using(
//                 (size_hidden,size_inputs), Uniform::new(0.0,1.0), &mut rng)
//         ];

//         (0..n_hidden+1).for_each(|x| {
//             let a = Array2::random_using(
//                 (size_hidden, size_hidden), Uniform::new(0.0,1.0), &mut rng);
//             weights.push(a);
//         });

//         let weights_out = Array2::random_using(
//             (size_output,size_hidden), Uniform::new(0.0,1.0), &mut rng);

//         let mut biases: Vec<Array1<f32>> = vec![];
//         (0..n_hidden).for_each(|x| {
//             let a = Array1::random_using(
//                 size_hidden, Uniform::new(0.0,1.0), &mut rng);
//             biases.push(a);
//         });
//         let biases_out = Array1::random_using(
//             size_output, Uniform::new(0.0,1.0), &mut rng);

//         Self {
//             size_inputs,
//             size_hidden,
//             n_hidden,
//             size_output,
//             weights,
//             weights_out,
//             biases,
//             biases_out,
//         }
//     }
// }


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






