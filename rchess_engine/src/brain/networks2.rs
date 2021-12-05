
pub use self::gradient::*;

use std::ops;
use std::marker::PhantomData;

use ndarray as nd;
use nd::{Array1,ArrayView1,Array2,ArrayView2};
use ndarray_rand::RandomExt;
use ndarray_rand::rand_distr::{Uniform,Normal,StandardNormal};

use rand::{Rng,SeedableRng};
use rand::prelude::{StdRng,Distribution};
use serde::{Serialize,Deserialize};
use serde_big_array::BigArray;

use num_traits::Num;

mod gradient {

    use super::*;

    #[derive(Debug,Default,PartialEq,Eq,PartialOrd,Ord,Clone,Copy,Serialize,Deserialize)]
    pub struct Gradient<T> {
        val:   T,
        m1:    T,
        m2:    T,
    }

    impl Gradient<f32> {
        const BETA1: f32 = 0.9;
        const BETA2: f32 = 0.999;

        pub fn update(&mut self, delta: f32) {
            self.val += delta;
        }

        pub fn calculate(&mut self, eta: f32) -> f32 {
            if self.val == 0.0 { return 0.0 }
            self.m1 = self.m1 * Self::BETA1 + self.val * (1.0 - Self::BETA1);
            self.m1 = self.m2 * Self::BETA2 + (self.val * self.val) * (1.0 - Self::BETA2);

            eta * self.m1 / ((self.m2 as f64).sqrt() + 1e-8) as f32
        }

    }

}

pub trait NNFunc<T> {
    fn act_fn(x: T) -> T;
    fn act_fn_prime(x: T) -> T;
}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub struct NNUE3<T, FN, const IS: usize, const OS: usize, const HL: usize>
    where FN: NNFunc<T>
{
    pub hidden_sizes:    Vec<usize>,
    pub weights:         Vec<Array2<T>>,
    pub biases:          Vec<Array2<T>>,
    pub activations:     Vec<Array2<T>>,
    pub errors:          Vec<Array2<T>>,
    pub w_gradients:     Vec<Array2<Gradient<T>>>,
    pub b_gradients:     Vec<Array2<Gradient<T>>>,
    pub func:            PhantomData<FN>,
}

/// New
impl<FN: NNFunc<f32>,const IS: usize,const OS: usize,const HL: usize> NNUE3<f32,FN,IS,OS,HL> {

    pub fn new(hidden_sizes: Vec<usize>, mut rng: &mut StdRng) -> Self {
        assert_eq!(hidden_sizes.len(), HL);

        let mut weights     = vec![];
        let mut biases      = vec![];
        let mut activations = vec![];
        let mut errors      = vec![];
        let mut w_gradients = vec![];
        let mut b_gradients = vec![];

        let mut input_size = IS;

        let dist = StandardNormal;

        for i in 0..hidden_sizes.len()+1 {
            let output_size = if i == hidden_sizes.len() {
                OS
            } else {
                hidden_sizes[i]
            };

            weights.push(
                Array2::random_using((output_size,input_size), dist, &mut rng));
            biases.push(
                Array2::random_using((output_size,1), dist, &mut rng));
            activations.push(
                Array2::random_using((output_size,1), dist, &mut rng));
            errors.push(
                Array2::random_using((output_size,1), dist, &mut rng));
            w_gradients.push(
                Array2::from_elem((output_size,input_size), Gradient::default()));
            b_gradients.push(
                Array2::from_elem((output_size,input_size), Gradient::default()));
            input_size = output_size;
        }

        // assert_eq!(weights.len(), HL);

        Self {
            hidden_sizes,
            weights,
            biases,
            activations,
            errors,
            w_gradients,
            b_gradients,
            func:  PhantomData,
        }
    }

}

/// Run
impl<FN: NNFunc<f32>,const IS: usize,const OS: usize,const HL: usize> NNUE3<f32,FN,IS,OS,HL> {

    // pub fn run(&mut self, inputs: Array2<f32>) -> Array2<f32> {
    pub fn run(&mut self, inputs: Array1<f32>) -> Array2<f32> {

        let output = &mut self.activations[0];
        let weight = &mut self.weights[0];
        let bias   = &mut self.biases[0];

        output.fill(0.0);

        // for c in output.columns_mut() {
        // }

        // let mut osize = 0;
        // for i in inputs.iter() {
        //     let osize = output.shape()[0] * output.shape()[1];
        //     for j in 0..osize {
        //         output[]
        //     }
        // }

        unimplemented!()
    }

}


