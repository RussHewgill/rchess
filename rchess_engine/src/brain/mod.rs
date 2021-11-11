
pub mod types;
pub mod filter;
pub mod nnue;
pub mod matrix;

use crate::types::*;
use crate::brain::types::*;
use crate::brain::matrix::*;

use ndarray::prelude::*;

use nalgebra::{SMatrix,SVector,Matrix,Vector,matrix,vector,DVector,DMatrix};
use nalgebra as na;

use rand::{Rng,SeedableRng};
use rand::prelude::StdRng;

pub mod nd {
    use crate::brain::{sigmoid, sigmoid_deriv, types::nd::*};
    use ndarray::*;

    impl NDNetwork {

        pub fn run(&self, input: Array1<f32>) -> Array1<f32> {

            let mut act = input;
            let mut activations = vec![act.clone()];
            let mut zs = vec![];

            for (w,b) in self.weights.iter().zip(self.biases.iter()) {
                let z = w.dot(&act) + b;
                zs.push(z.clone());
                act = z.map(|x| sigmoid(*x));
                activations.push(act.clone());
            }

            act
            // unimplemented!()
        }

        pub fn backprop(&mut self, inputs: Vec<Array1<f32>>, corrects: Vec<Array1<f32>>) {

            for (input, correct) in inputs.into_iter().zip(corrects.into_iter()) {

                let mut ws_new: Vec<Array2<f32>> = self.weights.iter().map(|w| {
                    Array2::zeros(w.dim())
                }).collect();
                let mut bs_new: Vec<Array1<f32>> = self.biases.iter().map(|b| {
                    Array1::zeros(b.dim())
                }).collect();

                let (wlen,blen) = (ws_new.len(),bs_new.len());

                let mut act = input;
                let mut activations = vec![act.clone()];
                let mut zs = vec![];

                for (w,b) in self.weights.iter().zip(self.biases.iter()) {
                    let z = w.dot(&act) + b;
                    zs.push(z.clone());
                    act = z.map(|x| sigmoid(*x));
                    activations.push(act.clone());
                }

                let d_cost = &activations[activations.len() - 1] - correct;
                let delta = d_cost * zs[zs.len() - 1].map(|x| sigmoid_deriv(*x)) ;

                bs_new[blen - 1] = delta.clone();

                let k0 = activations[activations.len() - 2].clone();
                let k0 = k0.insert_axis(Axis(1));

                eprintln!("k0 = {:?}", k0);
                let k1 = k0.t();
                eprintln!("k1 = {:?}", k1);

                // let k1 = activations[activations.len() - 2].t();
                // eprintln!("k1 = {:?}", k1);
                // let k2 = activations[activations.len() - 2].clone().reversed_axes();
                // eprintln!("k2 = {:?}", k2);

                let w_d = delta.dot(&k0);
                eprintln!("w_d = {:?}", w_d);

                // ws_new[wlen - 1] = d_w;

                // let d = self.weights[self.weights.len() - 1].dim();
                // let d = delta 

                // eprintln!("d = {:?}", d);

                unimplemented!()
            }
        }


    }

}


pub fn test_mnist(
    n0:               &MNNetwork,
    mut data:         Vec<(SVector<f32,784>,u8)>,
    n:                Option<usize>,
) {
    if let Some(n) = n { data.truncate(n); }
    let mut out: Vec<(u8,(usize, f32))> = vec![];
    for (img,lbl) in data.iter() {
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

pub fn test_mnist2(
    n0:               &DNetwork<f32,784,10>,
    mut data:         Vec<(SVector<f32,784>,u8)>,
    n:                Option<usize>,
) {
    if let Some(n) = n { data.truncate(n); }
    let mut out: Vec<(u8,(usize, f32))> = vec![];
    for (img,lbl) in data.iter() {

        let img = img.into_iter().copied().collect::<Vec<_>>();
        let img: DVector<f32> = DVector::from_vec(img);

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

impl<const IS: usize, const OS: usize> DNetwork<f32,IS,OS> {
    pub fn run(&self, input: &DVector<f32>) -> DVector<f32> {
        let (out,_,_) = self._run(input);
        out
    }

    pub fn _run(&self, input: &DVector<f32>) -> (DVector<f32>,Vec<DVector<f32>>,Vec<DVector<f32>>) {
        let mut activations: Vec<DVector<f32>> = vec![];
        let mut zs: Vec<DVector<f32>>          = vec![];

        let mut last = input.clone();

        activations.push(input.clone());

        for (ws,bs) in self.weights.iter().zip(self.biases.iter()) {
            let z = ws * last;
            let z = z + bs;
            let act = z.map(sigmoid);

            activations.push(act.clone());
            zs.push(z);

            last = act;
        }
        let pred = last;
        (pred,activations,zs)
    }

    fn backprop_mut(
        &mut self,
        inputs:         Vec<&DVector<f32>>,
        corrects:       Vec<DVector<f32>>,
        lr:             f32,
    ) {
        let ins = inputs.iter().zip(corrects.iter());

        let mut outs = vec![];
        for (input,correct) in ins.clone() {
            let (pred,acts,zs) = self._run(input);
            outs.push((pred,acts,zs))
        }
        let outs = ins.zip(outs.into_iter());

        let mut ws_new: Vec<Vec<DMatrix<f32>>> = vec![];
        let mut bs_new: Vec<Vec<DVector<f32>>> = vec![];

        for ((input,correct),(pred,acts,zs)) in outs {

            let delta = pred - correct; // OS,1
            let delta = delta.component_mul(&zs[zs.len() - 1].map(sigmoid_deriv));

            let mut prev_delta = delta.clone();

            let d = &delta * acts[acts.len() - 2].transpose();
            let mut new_ws: Vec<DMatrix<f32>> = vec![d];
            let mut new_bs: Vec<DVector<f32>> = vec![delta];

            for k in 0..self.sizes.len()-1 {
                let layer = self.sizes.len() - 1 - k;

                // eprintln!("k, layer = {:?}, {:?}", k, layer);

                let z  = &zs[layer-1];
                let sp = z.map(sigmoid_deriv);

                let d = self.weights[layer].transpose() * prev_delta;

                let d = d.component_mul(&sp);
                prev_delta = d.clone();

                let act = &acts[layer - 1];
                let w = &d * act.transpose();

                new_ws.push(w);
                new_bs.push(d);

                // if layer == 0 {
                //     w_in = Some(prev_delta * input.transpose());
                // } else if layer == self.sizes.len()-1 {
                //     // println!("wat output: {}", layer);
                //     let (act,z) = acts[acts.len()-1];
                //     let sp = z.map(sigmoid_deriv);
                //     let d = self.weights_out.transpose() * delta; // HS,1
                //     let d = d.component_mul(&sp);
                //     prev_delta = d;
                //     w_out = Some(delta * act.transpose());
                // } else {
                //     // println!("wat hidden: {}", layer);
                //     let (_,z) = acts[layer];
                //     let sp = z.map(sigmoid_deriv);
                //     let d = self.weights[layer-1].transpose() * prev_delta;
                //     let d = d.component_mul(&sp);
                //     prev_delta = d;
                //     let (act,_) = acts[layer-1];
                //     let w = d * act.transpose();
                //     ws.push(w);
                //     bs.push(d);
                //     // self.weights[layer-1] = self.weights[layer-1] - lr * x1;
                //     // self.biases[layer-1]  = self.biases[layer-1] - error;
                // }

            }

            // ws_new.push(new_ws);
            // bs_new.push(new_bs);

            for (mut ws, nw) in self.weights.iter_mut().zip(new_ws.into_iter().rev()) {
                *ws -= nw * lr;
            }

            for (mut bs, nb) in self.biases.iter_mut().zip(new_bs.into_iter().rev()) {
                *bs -= nb;
            }

        }

        // println!("wat 0");
        // let eta = lr / (ws_new.len() as f32);

        // let nw0: DMatrix<f32> = ws_new.iter().map(|x| x.0).sum();
        // self.weights_in  = self.weights_in - nw0 * eta;
        // let nw2: DMatrix<f32> = ws_new.iter().map(|x| x.2).sum();
        // self.weights_out = self.weights_out - nw2 * eta;

        // for nws in ws_new.into_iter() {
        //     for (mut ws, nw) in self.weights.iter_mut().zip(nws) {
        //         *ws -= nw * eta;
        //     }
        // }

        // for (k,(w,nws)) in self.weights.iter().zip(ws_new.into_iter()).enumerate() {
        //     eprintln!("k = {:?}", k);
        //     eprintln!("w.shape() = {:?}", w.shape());
        //     let nw: DMatrix<f32> = nws.iter().sum();
        //     eprintln!("nw.shape() = {:?}", nw.shape());
        // }

        // println!("wat 1");
        // for (mut ws, nws) in self.weights.iter_mut().zip(ws_new.into_iter()) {
        //     let nw: DMatrix<f32> = nws.iter().sum();
        //     *ws -= nw * eta;
        // }
        // println!("wat 2");

        // let blen = bs_new.len() as f32;

        // let nb0: DVector<f32> = bs_new.iter().map(|x| x.0).sum();
        // self.biases_in  = self.biases_in - nb0 / blen;
        // let nb2: DVector<f32> = bs_new.iter().map(|x| x.2).sum();
        // self.biases_out = self.biases_out - nb2 / blen;

        // println!("wat 3");
        // for (mut bs, nbs) in self.biases.iter_mut().zip(bs_new.into_iter()) {
        //     let nb: DVector<f32> = nbs.iter().sum();
        //     *bs -= nb / blen;
        // }

    }

    pub fn fill_input_matrix(
        size:  usize,
        // ins:   &Vec<(&DVector<f32>,DVector<f32>)>,
        // ins:   &[(&DVector<f32>,DVector<f32>)],
        ins:   &[(DVector<f32>,DVector<f32>)],
    ) -> (DMatrix<f32>,DMatrix<f32>) {
        let mut inputs = DMatrix::zeros(IS,size);
        let mut cors   = DMatrix::zeros(OS,size);
        for (k,(i,c)) in ins.iter().enumerate() {
            inputs.set_column(k, i);
            cors.set_column(k, c);
        }
        (inputs,cors)
    }

    pub fn backprop_mut_matrix(
        &mut self,
        // ins:            &Vec<(&DVector<f32>,DVector<f32>)>,
        // ins:            &[(&DVector<f32>,DVector<f32>)],
        ins:            &[(DVector<f32>,DVector<f32>)],
        lr:             f32,
    ) {
        let input_size = ins.len();
        let (inputs,corrects) = Self::fill_input_matrix(input_size, ins);

        // println!("inputs = {}", inputs);

        let mut acts: Vec<DMatrix<f32>> = vec![];
        let mut zs: Vec<DMatrix<f32>>          = vec![];

        acts.push(inputs.clone());

        let mut last = inputs;

        for (ws,bs) in self.weights.iter().zip(self.biases.iter()) {
            let mut z = ws * last;
            for i in 0..z.shape().1 {
                let mut c = z.column_mut(i);
                c += bs;
            }
            zs.push(z.clone());

            let act = z.map(sigmoid);
            acts.push(act.clone());

            last = act;
        }
        let pred = last;

        let delta = pred - corrects; // OS,ISS
        let delta = delta.component_mul(&zs[zs.len() - 1].map(sigmoid_deriv));

        let mut prev_delta = delta.clone();

        let d = &delta * acts[acts.len() - 2].transpose();

        // let d2 = (acts[acts.len() - 2]).ref_ndarray2();
        // let d2 = d2.t();
        // let d = delta.ref_ndarray2().dot(&d2);
        // let d = d.into_nalgebra();

        let mut new_ws: Vec<DMatrix<f32>> = vec![d];
        let mut new_bs: Vec<DMatrix<f32>> = vec![delta];

        for k in 1..self.sizes.len()-1 {
            let layer = self.sizes.len() - 1 - k;
            // eprintln!("k {} = layer {:?}", k, layer);

            let z = &zs[layer-1];
            let sp = z.map(sigmoid_deriv);

            let d = self.weights[layer].transpose() * prev_delta;
            let d = d.component_mul(&sp);
            prev_delta = d.clone();

            // let d1 = self.weights[layer].ref_ndarray2();
            // let d1 = d1.t();
            // let d = d1.dot(&prev_delta.ref_ndarray2());
            // let d2 = sp.ref_ndarray2();
            // let d = d * d2;
            // let d = d.into_nalgebra();
            // prev_delta = d.clone();

            // let act = &acts[layer - 1].ref_ndarray2();
            // let w   = &d.dot(&act.t());
            // let w = w.clone().into_nalgebra();

            let act = &acts[layer - 1];
            let w = &d * act.transpose();

            new_ws.push(w);
            new_bs.push(d);

        }

        for (mut ws, nw) in self.weights.iter_mut().zip(new_ws.into_iter().rev()) {
            *ws -= lr * nw;
        }

        // for (mut bs, nb) in self.biases.iter_mut().zip(new_bs.into_iter().rev()) {
        //     let m = nb.column_sum().map(|x| x / input_size as f32);
        //     *bs -= m;
        // }

    }

}

impl<const IS: usize, const HS: usize, const OS: usize> GNetwork<f32,IS,HS,OS> {

    pub fn fill_input_matrix<const ISS: usize>(
        ins: &Vec<(&SVector<f32,IS>,SVector<f32,OS>)>,
    ) -> (SMatrix<f32,IS,ISS>,SMatrix<f32,OS,ISS>) {
        let mut inputs: SMatrix<f32,IS,ISS> = SMatrix::zeros();
        let mut cors: SMatrix<f32,OS,ISS>   = SMatrix::zeros();
        // XXX: WTF is take(10) doing?
        // for (k,(i,c)) in ins.iter().take(10).enumerate() {
        for (k,(i,c)) in ins.iter().enumerate() {
            inputs.set_column(k, i);
            cors.set_column(k, c);
        }
        (inputs,cors)
    }

    fn repeat<const NN: usize, const ISS: usize>(v: &SVector<f32,NN>) -> SMatrix<f32,NN,ISS> {
        SMatrix::<f32,NN,ISS>::from_columns(&[*v; ISS])
        // let mut out = SMatrix::<f32,NN,ISS>::zeros();
        // for i in 0..ISS {
            // out.set_column(i, &v);
        // }
        // out
    }

    pub fn run_matrix<const ISS: usize>(
        &mut self,
        inputs:         &SMatrix<f32,IS,ISS>,
        corrects:       &SMatrix<f32,OS,ISS>,
        // ins:            &Vec<(&SVector<f32,IS>,SVector<f32,OS>)>,
    ) -> SMatrix<f32,OS,ISS> {
        // let (inputs,corrects) = Self::fill_input_matrix::<ISS>(&ins);

        let mut activations: Vec<SMatrix<f32,HS,ISS>> = vec![];
        let mut zs: Vec<SMatrix<f32,HS,ISS>>          = vec![];

        let b0 = [self.biases_in; ISS];

        // let z0 = self.weights_in * inputs; // HS,ISS
        // let z0 = z0 + Self::repeat(&self.biases_in);

        let z0 = self.weights_in * inputs + Self::repeat(&self.biases_in); // HS,ISS
        zs.push(z0);

        let act0 = z0.map(sigmoid);
        activations.push(act0);

        let mut last = act0;

        for (ws,bs) in self.weights.iter().zip(self.biases.iter()) {
            // let z = ws * last;
            // let z = z + Self::repeat(bs);
            let z = ws * last + Self::repeat(bs);
            zs.push(z);

            let act = z.map(sigmoid);
            activations.push(act);

            last = act;
            // eprintln!("z.shape() = {:?}", z.shape());
        }

        // let pred_z = self.weights_out * last;
        // let pred_z = pred_z + Self::repeat(&self.biases_out);
        let pred_z = self.weights_out * last + Self::repeat(&self.biases_out);
        let pred = pred_z.map(sigmoid); // OS,ISS

        pred
    }

    #[allow(unused_doc_comments)]
    pub fn backprop_mut_matrix<const ISS: usize>(
        &mut self,
        ins:            &Vec<(&SVector<f32,IS>,SVector<f32,OS>)>,
        lr:             f32,
    ) {

        let (inputs,corrects) = Self::fill_input_matrix::<ISS>(&ins);

        let mut activations: Vec<SMatrix<f32,HS,ISS>> = vec![];
        let mut zs: Vec<SMatrix<f32,HS,ISS>>          = vec![];

        // let b0 = [self.biases_in; ISS];

        let z0 = self.weights_in * inputs; // HS,ISS
        let z0 = z0 + Self::repeat(&self.biases_in);
        zs.push(z0.clone());

        let mut act = z0.map(sigmoid);
        activations.push(act);

        for (ws,bs) in self.weights.iter().zip(self.biases.iter()) {
            let z = ws * act;
            let z = z + Self::repeat(bs);
            zs.push(z.clone());

            let act = z.map(sigmoid);
            activations.push(act);
            // eprintln!("z.shape() = {:?}", z.shape());
        }

        let pred_z = self.weights_out * act;
        let pred_z = pred_z + Self::repeat(&self.biases_out);
        let pred = pred_z.map(sigmoid); // OS,ISS

        let delta = pred - corrects; // OS,ISS
        let delta = delta.component_mul(&pred_z.map(sigmoid_deriv));

        // XXX: Backprop

        let mut ws = vec![];
        let mut bs = vec![];

        let mut w_out: Option<SMatrix<f32,OS,HS>> = None;
        let mut w_in:  Option<SMatrix<f32,HS,IS>> = None;

        let mut prev_delta = SMatrix::<f32,HS,ISS>::zeros();

        for k in 0..self.n_hidden+1 {
            let layer = self.n_hidden - k;

            // eprintln!("{} = layer {:?}", k, layer);

            if layer == 0 {
                w_in = Some(prev_delta * inputs.transpose());
            } else if layer == self.n_hidden {

                let act = activations[activations.len()-1]; // HS,ISS
                let z   = zs[zs.len()-1]; // HS,ISS

                let sp = z.map(sigmoid_deriv);

                let d = self.weights_out.transpose() * delta; // HS,ISS
                let d = d.component_mul(&sp);
                prev_delta = d;

                w_out = Some(delta * act.transpose());

            } else {

                let z = zs[layer];
                let sp = z.map(sigmoid_deriv);

                let d = self.weights[layer-1].transpose() * prev_delta;
                let d = d.component_mul(&sp);
                prev_delta = d;

                let act = activations[layer-1];

                let w = d * act.transpose();

                ws.push(w);
                bs.push(d);

            }
        }

        // let eta  = lr / (ws_new.len() as f32 + 2.0);
        // let blen = 2.0 + bs_new.len() as f32;

        self.weights_in  = self.weights_in - lr * w_in.unwrap();
        for (mut ws, nw) in self.weights.iter_mut().zip(ws.into_iter()) {
            // *ws = *ws - lr * nw / ws.len() as f32;
            // *ws = *ws - lr * nw;
            *ws = *ws - (lr / ISS as f32) * nw;
        }
        self.weights_out = self.weights_out - lr * w_out.unwrap();

        // delta:      OS,ISS
        // prev_delta: HS,ISS
        // biases_in:  HS,1

        // let k = 1.0 / prev_delta.shape().1 as f32;
        // self.biases_in  = self.biases_in - k * prev_delta.column_sum();

        // for (mut ws, nb) in self.biases.iter_mut().zip(bs.into_iter()) {
        //     *ws = *ws - lr * nb.column_sum();
        // }

        // let k = 1.0 / delta.shape().1 as f32;
        // self.biases_out  = self.biases_out - k * delta.column_sum();

        // unimplemented!()
    }

}

impl<const IS: usize, const HS: usize, const OS: usize> GNetwork<f32,IS,HS,OS> {

    pub fn run(&self, input: &SVector<f32,IS>) -> SVector<f32,OS> {
        let (out,_,_) = self._run(input);
        out
    }

    pub fn _run(&self, input: &SVector<f32,IS>)
                -> (SVector<f32,OS>,SVector<f32,OS>,Vec<(SVector<f32,HS>,SVector<f32,HS>)>) {

        // let mut last = input.to_owned();
        // let mut activations = vec![];
        let mut activations: Vec<(SVector<f32,HS>,SVector<f32,HS>)> = vec![];

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
        inputs:         Vec<&SVector<f32,IS>>,
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


            let mut prev_error  = SVector::<f32,HS>::zeros();

            let mut ws = vec![];
            let mut bs = vec![];

            let mut w_out: Option<SMatrix<f32,OS,HS>> = None;
            let mut w_in: Option<SMatrix<f32,HS,IS>>  = None;

            let mut prev_delta = SVector::<f32,HS>::zeros();
            // let mut delta2: Option<SVector<f32,HS>> = None;

            for k in 0..self.n_hidden+1 {
                let layer = self.n_hidden - k;
                // eprintln!("k, layer = {:?}, {:?}", k, layer);

                if layer == 0 {

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

        let eta = lr / (ws_new.len() as f32 + 2.0);

        let nw0: SMatrix<f32,HS,IS> = ws_new.iter().map(|x| x.0).sum();
        self.weights_in  = self.weights_in - nw0 * eta;

        let nw2: SMatrix<f32,OS,HS> = ws_new.iter().map(|x| x.2).sum();
        self.weights_out = self.weights_out - nw2 * eta;

        for (mut ws, nws) in self.weights.iter_mut().zip(ws_new.into_iter().map(|x| x.1)) {
            let nw: SMatrix<f32,HS,HS> = nws.iter().sum();
            *ws = *ws - nw * eta;
        }

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

}

pub fn sigmoid_deriv(x: f32) -> f32 {
    // x * (1.0 - x)
    sigmoid(x) * (1.0 - sigmoid(x))
}

pub fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + f32::exp(-x))
}
