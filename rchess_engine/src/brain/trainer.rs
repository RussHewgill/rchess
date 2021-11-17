
use crate::tables::*;
use crate::types::*;
use crate::evaluate::*;
// use crate::brain::types::*;
use crate::brain::types::nnue::*;

use ndarray as nd;
use nd::{Array2};

pub fn generate_training_data(ts: &Tables) -> () {
    unimplemented!()
}

/// Backprop
impl NNUE {

    #[allow(unused_doc_comments)]
    pub fn backprop(&mut self, g: &Game, correct: i32, eta: i8) {

        // let mut ins_own   = Array2::zeros((NNUE_INPUT,1));
        // let mut ins_other = Array2::zeros((NNUE_INPUT,1));
        // let mut ins_own = sprs::CsMat::empty(sprs::CSC, NNUE_INPUT);
        // let mut ins_other = sprs::CsMat::empty(sprs::CSC, NNUE_INPUT);

        // let xs = Some((ins_own.view_mut(),ins_other.view_mut()));
        self.init_inputs(g);

        let t0 = std::time::Instant::now();

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
        let d1_own: nd::ArrayView2<i8> = d1.slice(nd::s![..256, ..]); // 256, 1
        let d1_own = &d1_own * &sp1_own;

        // let d1_other = d1.slice(nd::s![256.., ..]); // 256, 1
        // let d1_other = &d1_other * &sp1_other;

        eprintln!("d1_own.shape() = {:?}", d1_own.shape());
        eprintln!("ins_own.shape() = {:?}", ins_own.shape());

        // let mut k0 = 0;
        // let mut v0 = 0;
        // for x in ins_own.iter() {
        //     if *x != 0 {
        //         k0 += 1;
        //     } else {
        //         v0 += 1;
        //     }
        // }
        // eprintln!("k0 = {:?}", k0);
        // eprintln!("v0 = {:?}", v0);

        let t0 = std::time::Instant::now();
        let ws1_own0 = d1_own.dot(&ins_own.t()); // 256, INPUT
        println!("wat 0 in {:.3} seconds", t0.elapsed().as_secs_f64());

        let t0 = std::time::Instant::now();
        let mut ins_own = sprs::CsMat::csc_from_dense(ins_own.view(), 0);
        ins_own.transpose_mut();
        let d1_own  = sprs::CsMat::csc_from_dense(d1_own.view(), 0);
        use ndarray::linalg::Dot;
        let ws1_own1 = &d1_own * &ins_own;
        println!("wat 0 in {:.3} seconds", t0.elapsed().as_secs_f64());

        let ws1_own1 = ws1_own1.to_dense();

        eprintln!("ws1_own0 == ws1_own1 = {:?}", ws1_own0 == ws1_own1);

        // println!("starting");
        // let t0 = std::time::Instant::now();
        // for _ in 0..10 {
        //     let mut ins_own = sprs::CsMat::csc_from_dense(ins_own.view(), 0);
        //     ins_own.transpose_mut();
        //     let d1_own  = sprs::CsMat::csc_from_dense(d1_own.view(), 0);
        //     use ndarray::linalg::Dot;
        //     let ws1_own = &d1_own * &ins_own;
        // }
        // println!("wat 0 in {:.3} seconds", t0.elapsed().as_secs_f64());

        // println!("starting");
        // let t0 = std::time::Instant::now();
        // for _ in 0..10 {
        //     let ws1_own = d1_own.dot(&ins_own.t()); // 256, INPUT
        // }
        // println!("wat 0 in {:.3} seconds", t0.elapsed().as_secs_f64());


        // let ws1_own = d1_own.dot(&ins_own.t()); // 256, INPUT
        // println!("wat 5 in {:.3} seconds", t0.elapsed().as_secs_f64());

        // let ws1_other = d1_other.dot(&ins_other.t()); // 256, INPUT
        // println!("wat 6 in {:.3} seconds", t0.elapsed().as_secs_f64());

        // eprintln!("ws1_own.shape() = {:?}", ws1_own.shape());

    }



}



impl NNUE {
    pub fn train(&mut self) {
    }

    pub fn train_single(&mut self, g: &Game) {
    }

}




