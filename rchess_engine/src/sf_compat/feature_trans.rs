
use crate::evaluate::Score;
use crate::types::*;

use super::HALF_DIMS;
use super::accumulator::NNAccum;

use std::io::{self, Read,BufReader};
use std::fs::File;
use std::path::Path;

use byteorder::{ReadBytesExt, LittleEndian};

#[derive(Debug,PartialEq,Clone)]
pub struct NNFeatureTrans {
    biases:         Vec<i16>,
    weights:        Vec<i16>,
    psqt_weights:   Vec<i32>,

    accum:          NNAccum,

}

/// Consts, Init
impl NNFeatureTrans {
    const DIMS_HALF: usize = 1024;

    const DIMS_IN: usize = 64 * 11 * 64 / 2;
    const DIMS_OUT: usize = Self::DIMS_HALF * 2;

    const PSQT_BUCKETS: usize = 8;
    const LAYER_STACKS: usize = 8;

    pub fn new() -> Self {
        Self {
            // nn,
            biases:         vec![0; Self::DIMS_HALF],
            weights:        vec![0; Self::DIMS_HALF * Self::DIMS_IN],
            psqt_weights:   vec![0; Self::DIMS_IN * Self::PSQT_BUCKETS],

            accum:          NNAccum::new(),
        }
    }

    pub fn read_parameters(&mut self, mut rdr: &mut BufReader<File>) -> io::Result<()> {
        // println!("wat NNFeatureTrans");

        for mut x in self.biases.iter_mut() {
            *x = rdr.read_i16::<LittleEndian>()?;
        }

        for mut x in self.weights.iter_mut() {
            *x = rdr.read_i16::<LittleEndian>()?;
        }

        for mut x in self.psqt_weights.iter_mut() {
            *x = rdr.read_i32::<LittleEndian>()?;
        }

        Ok(())
    }

}

/// Update Accum
impl NNFeatureTrans {

    pub fn transform(&mut self, g: &Game, output: &mut [u8], bucket: usize) -> Score {
        self.update_accum(g, White);
        self.update_accum(g, Black);

        let persps: [Color; 2] = [g.state.side_to_move, !g.state.side_to_move];

        let accum      = &mut self.accum.accum;
        let psqt_accum = &mut self.accum.psqt;

        let psqt = (psqt_accum[persps[0]][bucket] - psqt_accum[persps[1]][bucket]) / 2;

        for p in 0..2 {
            let offset = HALF_DIMS * p;
            for j in 0..HALF_DIMS {
                let mut sum = accum[persps[p]][j];
                output[offset + j] = sum.clamp(0, 127) as u8;
            }
        }

        psqt
    }

    pub fn update_accum(&mut self, g: &Game, persp: Color) {

        // eprintln!("self.biases.len() = {:?}", self.biases.len());
        // let k = HALF_DIMS * std::mem::size_of::<i16>();
        // eprintln!("k = {:?}", k);

        // assert!(self.biases.len() == HALF_DIMS * std::mem::size_of::<i16>());
        self.accum.accum[persp].copy_from_slice(&self.biases);

        for k in 0..Self::PSQT_BUCKETS {
            self.accum.psqt[persp][k] = 0;
        }

        let mut active = vec![];
        self.accum.append_active(g, persp, &mut active);

        for idx in active.into_iter() {
            let offset = HALF_DIMS * idx;

            for j in 0..HALF_DIMS {
                self.accum.accum[persp][j] += self.weights[offset + j];
            }

            for k in 0..Self::PSQT_BUCKETS {
                self.accum.psqt[persp][k] += self.psqt_weights[idx * Self::PSQT_BUCKETS + k];
            }

        }
    }
}






