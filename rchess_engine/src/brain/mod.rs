
use ndarray::prelude::*;

use ndarray_rand::RandomExt;
use ndarray_rand::rand_distr::Uniform;

use rand::{Rng,SeedableRng};
use rand::prelude::StdRng;

use crate::types::*;


const FILTER_MASKS: [[u64; 6]; 6] = {
    [[
        0x0000000000070707,
        0x00000000000e0e0e,
        0x00000000001c1c1c,
        0x0000000000383838,
        0x0000000000707070,
        0x0000000000e0e0e0,
        ],[
        0x0000000007070700,
        0x000000000e0e0e00,
        0x000000001c1c1c00,
        0x0000000038383800,
        0x0000000070707000,
        0x00000000e0e0e000,
        ],[
        0x0000000707070000,
        0x0000000e0e0e0000,
        0x0000001c1c1c0000,
        0x0000003838380000,
        0x0000007070700000,
        0x000000e0e0e00000,
        ],[
        0x0000070707000000,
        0x00000e0e0e000000,
        0x00001c1c1c000000,
        0x0000383838000000,
        0x0000707070000000,
        0x0000e0e0e0000000,
        ],[
        0x0007070700000000,
        0x000e0e0e00000000,
        0x001c1c1c00000000,
        0x0038383800000000,
        0x0070707000000000,
        0x00e0e0e000000000,
        ],[
        0x0707070000000000,
        0x0e0e0e0000000000,
        0x1c1c1c0000000000,
        0x3838380000000000,
        0x7070700000000000,
        0xe0e0e00000000000,
    ]]
};

#[derive(Debug,Clone)]
pub struct Network {
    num_ns:          (usize,usize,usize),
    weights_in:      Array2<f32>,
    weights_out:     Array2<f32>,
    bias:            Array2<f32>,
    bias_out:        Array1<f32>,
}

impl Network {
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

impl Network {

    // [[0.57, 0.90],
    //  [0.49, 0.64]]

    pub fn run(&self, input: Array1<f32>) -> (Array1<f32>,f32) {
        // let x0: ArrayView1<f32> = self.weights_in.slice(s![0, ..]);
        // let layer1: f32 = sigmoid(input.dot(&x0));

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

    // pub fn run(&self, input: Array1<f32>) -> f32 {
    //     let x0: ArrayView1<f32> = self.weights_in.slice(s![0, ..]);
    //     let x1: ArrayView1<f32> = self.weights_in.slice(s![1, ..]);
    //     let layer1: f32 = sigmoid(input.dot(&x0));
    //     let out = &x1 * layer1;
    //     let out = sigmoid(out[0]);
    //     out
    // }

    pub fn backprop(
        &mut self,
        correct:        f32,
        predicted_out:  f32,
        input:          Array1<f32>,
        hidden_out:     Array1<f32>,
    ) -> f32 {

        let error = correct - predicted_out;
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

        // eprintln!("d_hidden = {:?}", d_hidden);
        // eprintln!("self.bias 0 = {:?}", self.bias);
        self.bias = &self.bias + d_hidden.sum();
        // eprintln!("self.bias 1 = {:?}", self.bias);

        error
    }

}

// fn sum_of_squares(correct: Array1<f32>, result: Array1<f32>) -> f32 {
//     let mut out = 0.0;
//     ndarray::Zip::from(&correct)
//         .and(&result)
//         .for_each(|c,r| {
//             out += 0.5 * (c - r).powi(2);
//             // out += (c - r).powi(2);
//         });
//     out
// }

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

pub type InputPlane = BitBoard;

pub struct ConvLayer {
    pub filter: [[u16; 3]; 3],
}

impl ConvLayer {

    pub fn bitboard_section((x,y): (usize,usize), bb: BitBoard) -> [u16; 9] {
        let mask = FILTER_MASKS[x][y];
        let b = bb.0 & mask;

        let mut out = [0; 9];
        let mut mask = 1u64;
        for x in 0..9 {
            let m = mask << x;
            // eprintln!("mask {} = {:#012b}, {:#012b}", x, m, b & m);
            // out[x] = BitBoard(b & m).bitscan() as u16;
        }

        out
        // unimplemented!()
    }

    pub fn scan_bitboard(&self, bb: BitBoard) -> [[u16; 6]; 6] {
        let mut out = [[0; 6]; 6];

        for x in 0..6 {
            for y in 0..6 {
                // out[x][y] = self.dot()
            }
        }

        out
    }

    pub fn as_arr(&self) -> [u16; 9] {
        [self.filter[0][0],
          self.filter[1][0],
          self.filter[2][0],
          self.filter[0][1],
          self.filter[1][1],
          self.filter[2][1],
          self.filter[0][2],
          self.filter[1][2],
          self.filter[2][2],
        ]
    }

    pub fn dot(&self, other: [u16; 9]) -> u16 {
        self.as_arr().iter().zip(other.iter()).map(|(a,b)| a * b).sum()
    }
}

