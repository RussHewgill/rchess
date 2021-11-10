
use crate::tables::*;
use crate::types::*;
use crate::evaluate::*;

use crate::brain::filter::*;

// pub use self::nd::*;

use rand::{Rng,SeedableRng};
use rand::prelude::StdRng;
use rand::distributions::Uniform;

use nalgebra::{SMatrix,SVector,Matrix,Vector};

use serde::ser::{Serializer,SerializeStruct};
use serde::de::{Deserializer,DeserializeOwned};
use serde::{Serialize,Deserialize};

pub mod nd {
    use serde::{Serialize,Deserialize};

    use rand::{Rng,SeedableRng};
    use rand::prelude::StdRng;
    use rand::distributions::Uniform;

    use ndarray::*;
    use ndarray_rand::RandomExt;

    #[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
    pub struct NDNetwork {
        // pub n_hidden:   usize,
        pub sizes:       Vec<usize>,

        pub weights:     Vec<Array2<f32>>,
        pub biases:      Vec<Array1<f32>>,
    }

    impl NDNetwork {
        pub fn new(
            size_inputs:      usize,
            size_hidden:      usize,
            n_hidden:         usize,
            size_output:      usize,
        ) -> Self {
            let mut rng: StdRng = SeedableRng::seed_from_u64(18105974836011991331);
            let dist = Uniform::new(0.0,1.0);

            let mut weights = vec![
                Array2::random_using(
                    (size_hidden,size_inputs), dist, &mut rng)
            ];

            (0..n_hidden+1).for_each(|x| {
                let a = Array2::random_using(
                    (size_hidden, size_hidden), dist, &mut rng);
                weights.push(a);
            });

            let weights_out = Array2::random_using(
                (size_output,size_hidden), dist, &mut rng);
            weights.push(weights_out);

            let mut biases: Vec<Array1<f32>> = vec![];

            (0..n_hidden).for_each(|x| {
                let a = Array1::random_using(
                    size_hidden, dist, &mut rng);
                biases.push(a);
            });
            let biases_out = Array1::random_using(
                size_output, dist, &mut rng);
            biases.push(biases_out);

            let mut sizes = vec![size_inputs];
            for _ in 0..n_hidden {
                sizes.push(size_hidden);
            }
            sizes.push(size_output);

            Self {
                sizes,

                weights,
                biases,
            }

        }
    }

}


#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub struct GNetwork<T, const IS: usize, const HS: usize, const OS: usize>
where T: nalgebra::Scalar + PartialEq + Serialize,
{
    pub n_hidden:         usize,

    pub weights_in:       SMatrix<T, HS, IS>,
    pub weights:          Vec<SMatrix<T, HS, HS>>,
    pub weights_out:      SMatrix<T, OS, HS>,

    pub biases_in:        SVector<T, HS>,
    pub biases:           Vec<SVector<T, HS>>,
    pub biases_out:       SVector<T, OS>,
}


impl<T, const IS: usize, const HS: usize, const OS: usize> GNetwork<T,IS,HS,OS>
where T: nalgebra::Scalar + PartialEq + Serialize + DeserializeOwned
{
    pub fn write_to_file(&self, path: &str, backup: Option<&str>) -> std::io::Result<()> {
        use std::io::Write;
        let b: Vec<u8> = bincode::serialize(&self).unwrap();
        if let Some(backup) = backup {
            std::fs::rename(path, backup)?;
        }
        let mut f = std::fs::File::create(&path)?;
        f.write_all(&b)
    }

    pub fn read_from_file(path: &str) -> std::io::Result<Self> {
        let f = std::fs::read(&path)?;
        let nn: Self = bincode::deserialize(&f).unwrap();
        Ok(nn)
    }
}

// pub type Network = GNetwork<f32, 2, 3, 1>;
pub type Network = GNetwork<f32, 2, 3, 1>;

pub type MNNetwork = GNetwork<f32, 784, 16, 10>;

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






