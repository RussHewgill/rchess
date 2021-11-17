
// use crate::tables::*;
// use crate::types::*;
// use crate::evaluate::*;

use crate::brain::filter::*;

use rand::{Rng,SeedableRng};
use rand::prelude::{StdRng,Distribution};
use rand::distributions::{Uniform,uniform::SampleUniform};
use ndarray_rand::rand_distr::num_traits::Zero;
use ndarray_rand::RandomExt;

use nalgebra::{SMatrix,SVector,Matrix,Vector,DVector,DMatrix,Dynamic,Const};
use nalgebra as na;

use serde::ser::{Serializer,SerializeStruct};
use serde::de::{Deserializer,DeserializeOwned};
use serde::{Serialize,Deserialize};

pub mod nnue {

    use crate::tables::*;
    use crate::types::*;
    use crate::evaluate::*;
    use crate::brain::types::*;
    use crate::brain::matrix::*;
    use crate::brain::accumulator::*;

    use nalgebra::{SMatrix,SVector,Matrix,Vector,DVector,DMatrix,Dynamic,Const};
    use nalgebra as na;

    use ndarray as nd;
    use nd::{Array1,Array2};

    use sprs::CsMat;

    use rand::{Rng,SeedableRng};
    use rand::prelude::{StdRng,Distribution};
    use rand::distributions::Uniform;

    use serde::{Serialize,Deserialize};

    // pub const NNUE_INPUT: usize  = 64 * 63 * 10; // 40320
    // pub const NNUE_INPUT: usize  = 64 * 63 * 10 + 32 + 4; // 40320 + EP + Castling = 40356

    // pub const NNUE_INPUT: usize  = 20480 + 32 + 4; // 20516

    // /// HalfKAv2 = 44352 + 32 + 4 = 44388
    // pub const NNUE_INPUT: usize  = 64 * 63 * 11 + 32 + 4;

    /// HalfKAv2 = 45056 + 32 + 4 = 45092
    pub const NNUE_INPUT: usize  = 64 * 64 * 11 + 32 + 4;

    pub const NNUE_L2: usize     = 256;
    pub const NNUE_L3: usize     = 32;
    pub const NNUE_OUTPUT: usize = 32;

    #[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
    pub struct NNUE {

        pub side:               Color,

        pub en_passant:         Option<Coord>,

        // pub accum:              Accum<10>,

        #[serde(skip,default = "NNUE::def_inputs")]
        pub inputs_own:         CsMat<i16>, // INPUT, 1
        #[serde(skip,default = "NNUE::def_inputs")]
        pub inputs_other:       CsMat<i16>, // INPUT, 1

        #[serde(skip)]
        pub activations_own:    Array2<i16>, // 256 x 1
        #[serde(skip)]
        pub activations_other:  Array2<i16>, // 256 x 1

        pub weights_1:          Array2<i8>, // 256 x INPUT
        pub weights_2:          Array2<i8>, // 32 x 512
        pub weights_3:          Array2<i8>, // 32 x 32
        pub weights_4:          Array2<i8>, // 1 x 32

        pub biases_1:           Array2<i16>, // 256
        pub biases_2:           Array2<i32>, // 32
        pub biases_3:           Array2<i32>, // 32
        pub biases_4:           Array2<i32>, // 1 ??
    }

    impl NNUE {

        fn def_inputs() -> CsMat<i16> {
            sprs::CsMat::empty(sprs::CSC, NNUE_INPUT)
        }

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

    impl NNUE {
        pub fn new(side: Color, mut rng: &mut StdRng) -> Self {

            // let dist0 = Uniform::new(i16::MIN,i16::MAX);
            let dist0 = Uniform::new(i8::MIN as i16,i8::MAX as i16);
            let dist1 = Uniform::new(i8::MIN,i8::MAX);
            let dist2 = Uniform::new(i8::MIN as i32,i8::MAX as i32);

            let inputs_own = sprs::CsMat::empty(sprs::CSC, NNUE_INPUT);
            let inputs_other = sprs::CsMat::empty(sprs::CSC, NNUE_INPUT);

            // let inputs_own        = Array2::zeros((NNUE_INPUT, 1));
            // let inputs_other      = Array2::zeros((NNUE_INPUT, 1));
            let activations_own   = Array2::zeros((NNUE_L2, 1));
            let activations_other = Array2::zeros((NNUE_L2, 1));

            // let weights_in_other = Array2::<i16>::random_using((NNUE_L2,NNUE_INPUT), dist0, &mut rng);

            // let weights_1 = Array2::random_using((NNUE_L2,NNUE_INPUT), dist0, &mut rng);

            // let weights_1_own   = Array2::random_using((NNUE_L2,NNUE_INPUT), dist1, &mut rng);
            // let weights_1_other = Array2::random_using((NNUE_L2,NNUE_INPUT), dist1, &mut rng);

            let weights_1 = Array2::random_using((NNUE_L2,NNUE_INPUT), dist1, &mut rng);
            let weights_2 = Array2::random_using((NNUE_L3,NNUE_L2 * 2), dist1, &mut rng);
            let weights_3 = Array2::random_using((NNUE_OUTPUT,NNUE_L3), dist1, &mut rng);
            let weights_4 = Array2::random_using((1,NNUE_OUTPUT), dist1, &mut rng);

            // let biases_1 = Array2::random_using((NNUE_L2 * 2,1), dist1, &mut rng);

            // let biases_1_own = Array2::random_using((NNUE_L2,1), dist1, &mut rng);
            // let biases_1_other = Array2::random_using((NNUE_L2,1), dist1, &mut rng);

            let biases_1 = Array2::random_using((NNUE_L2,1), dist0, &mut rng);
            let biases_2 = Array2::random_using((NNUE_L3,1), dist2, &mut rng);
            let biases_3 = Array2::random_using((NNUE_OUTPUT,1), dist2, &mut rng);
            let biases_4 = Array2::random_using((1,1), dist2, &mut rng);

            Self {
                side,

                en_passant: None,

                inputs_own,
                inputs_other,

                // accum: Accum::<10>::new(side),
                activations_own,
                activations_other,

                // weights_1_own,
                // weights_1_other,
                weights_1,
                weights_2,
                weights_3,
                weights_4,

                // biases_1_own,
                // biases_1_other,
                biases_1,
                biases_2,
                biases_3,
                biases_4,
            }
        }
    }

        fn gen_vec<T>(n: usize, dist: Uniform<T>, mut rng: &mut StdRng) -> Vec<T>
        where T: rand::distributions::uniform::SampleUniform,
        {
            (0..n).map(|_| dist.sample(&mut rng)).collect()
        }

}

pub mod nnue2 {

    use crate::tables::*;
    use crate::types::*;
    use crate::evaluate::*;
    use crate::brain::types::*;
    use crate::brain::matrix::*;

    use nalgebra::{SMatrix,SVector,Matrix,Vector,DVector,DMatrix,Dynamic,Const};
    use nalgebra as na;

    use ndarray as nd;
    use nd::{Array1,Array2};

    use rand::{Rng,SeedableRng};
    use rand::prelude::{StdRng,Distribution};
    use rand::distributions::Uniform;

    use serde::{Serialize,Deserialize};

    // pub const NNUE_INPUT: usize  = 64 * 63 * 10; // 40320
    pub const NNUE_INPUT: usize  = 64 * 63 * 10 + 32 + 4; // 40320 + EP + Castling = 40356
    pub const NNUE_L2: usize     = 256;
    pub const NNUE_L3: usize     = 32;
    pub const NNUE_OUTPUT: usize = 32;

    #[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
    pub struct GNNUE<I,H> {
        pub dirty:              bool,

        pub side:               Color,

        pub en_passant:         Option<Coord>,

        // TODO: biases

        #[serde(skip)]
        pub inputs_own:         Array2<I>,
        #[serde(skip)]
        pub inputs_other:       Array2<I>,

        #[serde(skip)]
        pub activations_own:    Array2<H>,
        #[serde(skip)]
        pub activations_other:  Array2<H>,

        // pub weights_1_white:    Array2<I>, // 256 x 40356
        // pub weights_1_black:    Array2<I>, // 256 x 40356

        pub weights_1_own:      Array2<I>, // 256 x 40356
        pub weights_1_other:    Array2<I>, // 256 x 40356

        pub weights_2:          Array2<H>, // 32 x 512
        pub weights_3:          Array2<H>, // 32 x 32
        pub weights_4:          Array2<H>, // 1 x 32

        // pub biases_1:           Array2<H>, // 256 * 2

        pub biases_1_own:       Array2<H>, // 256
        pub biases_1_other:     Array2<H>, // 256

        pub biases_2:           Array2<i32>, // 32
        pub biases_3:           Array2<i32>, // 32
        pub biases_4:           Array2<i32>, // 1 ??

        pub test_nn:            DNetwork<f32,512,1>,

    }

    // pub type NNUE = GNNUE<f32,f32>;
    pub type NNUE = GNNUE<i16,i8>;

    impl<I,T> GNNUE<I,T>
        where I: PartialEq + Serialize + DeserializeOwned + Default,
              T: PartialEq + Serialize + DeserializeOwned + Default,
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

    // impl<I,T> GNNUE<I,T>
    // where I: Serialize + DeserializeOwned + Default + Clone + Zero,
    //       T: Serialize + DeserializeOwned + Default + Clone + Zero,
    // {
    //     pub fn empty(side: Color) -> Self {
    //         let inputs_own   = nd::Array2::zeros((NNUE_INPUT, 1));
    //         let inputs_other = nd::Array2::zeros((NNUE_INPUT, 1));
    //         let activations1_own   = nd::Array2::zeros((NNUE_L2, 1));
    //         let activations1_other = nd::Array2::zeros((NNUE_L2, 1));
    //         let weights_1 = nd::Array2::zeros((NNUE_L2,NNUE_INPUT));
    //         let weights_2 = nd::Array2::zeros((NNUE_L3,NNUE_L2 * 2));
    //         let weights_3 = nd::Array2::zeros((NNUE_OUTPUT,NNUE_L3));
    //         let weights_4 = nd::Array2::zeros((1,NNUE_OUTPUT));
    //         Self {
    //             dirty: true,
    //             side,
    //             en_passant: None,
    //             inputs_own,
    //             inputs_other,
    //             activations_own: activations1_own,
    //             activations_other: activations1_other,
    //             weights_1,
    //             weights_2,
    //             weights_3,
    //             weights_4,
    //             weights_1,
    //             weights_2,
    //             weights_3,
    //             weights_4,
    //         }
    //     }
    // }

    impl GNNUE<i16,i8> {

        // pub fn new(side: Color, mut rng: &mut StdRng) -> Self {
        pub fn new(side: Color, mut rng: &mut StdRng, nn: DNetwork<f32,512,1>) -> Self {

            // let dist0 = Uniform::new(i16::MIN,i16::MAX);
            let dist0 = Uniform::new(i8::MIN as i16,i8::MAX as i16);
            let dist1 = Uniform::new(i8::MIN,i8::MAX);
            let dist2 = Uniform::new(i8::MIN as i32,i8::MAX as i32);

            let inputs_own        = Array2::zeros((NNUE_INPUT, 1));
            let inputs_other      = Array2::zeros((NNUE_INPUT, 1));
            let activations_own   = Array2::zeros((NNUE_L2, 1));
            let activations_other = Array2::zeros((NNUE_L2, 1));

            // let weights_in_other = Array2::<i16>::random_using((NNUE_L2,NNUE_INPUT), dist0, &mut rng);

            // let weights_1 = Array2::random_using((NNUE_L2,NNUE_INPUT), dist0, &mut rng);

            let weights_1_own   = Array2::random_using((NNUE_L2,NNUE_INPUT), dist0, &mut rng);
            let weights_1_other = Array2::random_using((NNUE_L2,NNUE_INPUT), dist0, &mut rng);

            let weights_2 = Array2::random_using((NNUE_L3,NNUE_L2 * 2), dist1, &mut rng);
            let weights_3 = Array2::random_using((NNUE_OUTPUT,NNUE_L3), dist1, &mut rng);
            let weights_4 = Array2::random_using((1,NNUE_OUTPUT), dist1, &mut rng);

            // let biases_1 = Array2::random_using((NNUE_L2 * 2,1), dist1, &mut rng);

            let biases_1_own = Array2::random_using((NNUE_L2,1), dist1, &mut rng);
            let biases_1_other = Array2::random_using((NNUE_L2,1), dist1, &mut rng);

            let biases_2 = Array2::random_using((NNUE_L3,1), dist2, &mut rng);
            let biases_3 = Array2::random_using((NNUE_OUTPUT,1), dist2, &mut rng);
            let biases_4 = Array2::random_using((1,1), dist2, &mut rng);

            Self {
                dirty: true,
                side,

                en_passant: None,

                inputs_own,
                inputs_other,

                activations_own,
                activations_other,

                weights_1_own,
                weights_1_other,
                weights_2,
                weights_3,
                weights_4,

                biases_1_own,
                biases_1_other,
                biases_2,
                biases_3,
                biases_4,

                test_nn: nn,
            }
        }

        fn gen_vec<T>(n: usize, dist: Uniform<T>, mut rng: &mut StdRng) -> Vec<T>
        where T: rand::distributions::uniform::SampleUniform,
        {
            (0..n).map(|_| dist.sample(&mut rng)).collect()
        }

    }

    // impl GNNUE<f32,f32> {

    //     pub fn new(side: Color, mut rng: &mut StdRng) -> Self {

    //         let dist0 = Uniform::new(-1.0,1.0);

    //         let inputs_own   = DVector::zeros(NNUE_INPUT);
    //         let inputs_other = DVector::zeros(NNUE_INPUT);

    //         let activations1_own   = DVector::zeros(NNUE_L2);
    //         let activations1_other = DVector::zeros(NNUE_L2);

    //         let weights_in_own = DMatrix::from_vec(
    //             NNUE_L2,NNUE_INPUT,Self::gen_vec(NNUE_L2 * NNUE_INPUT,dist0,&mut rng));
    //         let weights_in_other = DMatrix::from_vec(
    //             NNUE_L2,NNUE_INPUT,Self::gen_vec(NNUE_L2 * NNUE_INPUT,dist0,&mut rng));

    //         let weights_l2 = DMatrix::from_vec(
    //             NNUE_L3,NNUE_L2 * 2,Self::gen_vec(NNUE_L3 * NNUE_L2 * 2,dist0,&mut rng));

    //         let weights_l3 = DMatrix::from_vec(
    //             NNUE_OUTPUT,NNUE_L3,Self::gen_vec(NNUE_OUTPUT * NNUE_L3,dist0,&mut rng));

    //         let weights_out = DMatrix::from_vec(
    //             1,NNUE_OUTPUT,Self::gen_vec(NNUE_OUTPUT,dist0,&mut rng));

    //         let inputs_own   = inputs_own.ref_ndarray2().to_owned();
    //         let inputs_other = inputs_other.ref_ndarray2().to_owned();

    //         let activations1_own   = activations1_own.ref_ndarray2().to_owned();
    //         let activations1_other = activations1_other.ref_ndarray2().to_owned();

    //         let weights_1 = weights_in_own.ref_ndarray2().to_owned();
    //         let weights_2 = weights_l2.ref_ndarray2().to_owned();
    //         let weights_3 = weights_l3.ref_ndarray2().to_owned();
    //         let weights_4 = weights_out.ref_ndarray2().to_owned();

    //         let biases_1 = Array2::random_using((NNUE_L2,1), dist0, &mut rng);
    //         let biases_2 = Array2::random_using((NNUE_L3,1), dist1, &mut rng);
    //         let biases_3 = Array2::random_using((NNUE_OUTPUT,1), dist1, &mut rng);
    //         let biases_4 = Array2::random_using((1,1), dist1, &mut rng);

    //         Self {
    //             dirty: true,
    //             side,

    //             en_passant: None,

    //             inputs_own,
    //             inputs_other,

    //             activations_own: activations1_own,
    //             activations_other: activations1_other,

    //             weights_1: weights_in,
    //             // weights_in_other,
    //             // weights_l2_own,
    //             // weights_l2_other,
    //             weights_2: weights_l2,
    //             weights_3: weights_l3,
    //             weights_4: weights_out,
    //         }
    //     }

    //     fn gen_vec<T>(n: usize, dist: Uniform<T>, mut rng: &mut StdRng) -> Vec<T>
    //         where T: rand::distributions::uniform::SampleUniform,
    //     {
    //         (0..n).map(|_| dist.sample(&mut rng)).collect()
    //     }
    // }

}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub struct DNetwork<T, const IS: usize, const OS: usize>
where T: nalgebra::Scalar + PartialEq + Serialize,
{
    pub sizes:         Vec<usize>,

    pub weights:       Vec<DMatrix<T>>,
    pub biases:        Vec<DVector<T>>,
}

// impl<const IS: usize, const HS: usize, const OS: usize> DNetwork<f32,IS,HS,OS> {
impl<const IS: usize, const OS: usize> DNetwork<f32,IS,OS> {
    pub fn new(sizes: Vec<usize>) -> Self {
        Self::_new(sizes, (0.0, 1.0), Some(18105974836011991331))
    }

    pub fn new_range(sizes: Vec<usize>, mm: (f32,f32)) -> Self {
        Self::_new(sizes, mm, Some(18105974836011991331))
    }

    pub fn _new_rng(sizes: Vec<usize>, mm: (f32,f32), mut rng: &mut StdRng) -> Self {

        let dist = rand_distr::Normal::new(0.0, 1.0).unwrap();
        // let dist = Uniform::new(mm.0,mm.1);

        let mut weights = vec![];
        let mut biases = vec![];

        for n in 1..sizes.len() {
            let s0 = sizes[n-1];
            let s1 = sizes[n];
            let ws = DMatrix::from_vec(s1,s0,Self::gen_vec(s0*s1,dist,&mut rng));
            weights.push(ws);
            let bs = DVector::from_vec(Self::gen_vec(s1, dist, &mut rng));
            biases.push(bs);
        }

        Self {
            sizes,
            weights,
            biases,
        }
    }

    pub fn _new(sizes: Vec<usize>, mm: (f32,f32), seed: Option<u64>) -> Self {
        assert!(sizes.len() > 2);

        assert!(sizes[0] == IS);
        assert!(sizes[sizes.len() - 1] == OS);

        let mut rng: StdRng = if let Some(seed) = seed {
            SeedableRng::seed_from_u64(seed)
        } else {
            // SeedableRng::seed_from_u64(18105974836011991331)
            let mut r = rand::thread_rng();
            SeedableRng::from_rng(r).unwrap()
        };

        Self::_new_rng(sizes, mm, &mut rng)
    }

    // fn gen_vec<D>(n: usize, dist: Uniform<f32>, mut rng: &mut StdRng) -> Vec<f32>
    fn gen_vec<D>(n: usize, dist: D, mut rng: &mut StdRng) -> Vec<f32>
        where D: rand::distributions::Distribution<f32>
    {
        (0..n).map(|_| dist.sample(&mut rng)).collect()
    }

}

// pub type GMatrix<T,const R: usize,const C: usize> = na::OMatrix<T,Const<R>,Const<C>>;
// pub type GVector<T,const D: usize> = na::OMatrix<T,Const<D>,na::U1>;

// #[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
#[derive(Debug,Clone,Serialize,Deserialize)]
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






