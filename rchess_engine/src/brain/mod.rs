
pub mod types;
pub mod filter;
pub mod nnue;

use crate::types::*;

use ndarray::prelude::*;

use ndarray_rand::RandomExt;
use ndarray_rand::rand_distr::Uniform;

use rand::{Rng,SeedableRng};
use rand::prelude::StdRng;

#[derive(Debug,Clone)]
pub struct TestNetwork {
    num_ns:          (usize,usize,usize),
    weights_in:      Array2<f32>,
    weights_out:     Array2<f32>,
    bias:            Array2<f32>,
    bias_out:        Array1<f32>,
}

impl TestNetwork {
    pub fn new(input_ns: usize, hidden_ns: usize, output_ns: usize) -> Self {
        let mut rng: StdRng = SeedableRng::seed_from_u64(18105974836011991331);

        let weights_in  = Array2::random_using(
            (input_ns, hidden_ns), Uniform::new(0.0,1.0), &mut rng);
        let weights_out = Array2::random_using(
            (hidden_ns,output_ns), Uniform::new(0.0,1.0), &mut rng);

        let bias     = Array2::random_using(
            (1,hidden_ns), Uniform::new(0.0,1.0), &mut rng);
        let bias_out = Array1::random_using(output_ns, Uniform::new(0.0,1.0), &mut rng);

        Self {
            num_ns:      (input_ns,hidden_ns,output_ns),
            weights_in,
            weights_out,
            bias,
            bias_out,
        }

    }
}

impl TestNetwork {

    pub fn run(&self, input: Array1<f32>) -> (Array1<f32>,f32) {

        let hidden_activation = input.dot(&self.weights_in);
        let hidden_activation = hidden_activation + &self.bias;

        let hidden_out = hidden_activation.map(|x| sigmoid(*x));

        let output_activation = hidden_out.dot(&self.weights_out);
        let output_activation = output_activation + &self.bias_out;
        let out = output_activation.map(|x| sigmoid(*x));

        let hidden_out = hidden_out.index_axis(Axis(0), 0).to_owned();

        (hidden_out,out[[0,0]])
        // unimplemented!()
    }

    pub fn backprop(
        &mut self,
        correct:        f32,
        predicted_out:  f32,
        input:          Array1<f32>,
        hidden_out:     Array1<f32>,
    ) -> f32 {

        let error = correct - predicted_out;
        // let error = (predicted_out - correct).powi(2);
        let d_predicted = error * sigmoid_deriv(predicted_out);

        let error_hidden = &self.weights_out.t() * d_predicted;
        let d_hidden = error_hidden * hidden_out.map(|x| sigmoid_deriv(*x));

        // let lr = 0.5 * error * error;
        let lr = 0.1;

        let x0 = (d_predicted * lr) * &hidden_out.t();
        self.weights_out = &self.weights_out + x0.insert_axis(Axis(1));

        self.bias_out = &self.bias_out + d_predicted;

        let input: Array2<f32>    = input.insert_axis(Axis(0));
        let x2 = lr * input.t().dot(&d_hidden);
        self.weights_in = &self.weights_in + x2;

        self.bias = &self.bias + d_hidden.sum();

        error
    }

}

fn sigmoid_deriv(x: f32) -> f32 {
    x * (1.0 - x)
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + f32::exp(-x))
}

// pub struct Network<const I: usize, const O: usize> {
//     pub input:     [f32; I],
//     pub hidden:    []
//     pub output:    [f32; O],
// }

// pub struct InputPlane()

// pub type InputPlane = BitBoard;

// pub struct ConvLayer {
//     pub filter: [[u16; 3]; 3],
// }

