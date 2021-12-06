



use crate::types::*;

use std::io::{self, Read,BufReader};
use std::fs::File;
use std::path::Path;

use byteorder::{ReadBytesExt, LittleEndian};

#[derive(Debug,PartialEq,Clone)]
pub struct NNFeatureTrans {
    biases:         Vec<i16>,
    weights:        Vec<i16>,
    psqt_weights:   Vec<i32>,

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

    pub fn transform(&self, g: &Game, output: &mut [u8], bucket: usize) -> i32 {
        self.update_accum(g, White);
        self.update_accum(g, Black);

        let persps: [Color; 2] = [g.state.side_to_move, !g.state.side_to_move];

        // let accum = 

        unimplemented!()
    }

    pub fn evaluate(&mut self, g: &Game) {
        let bucket = (g.state.material.count() - 1) / 4;

        // let psqt = self.f

    }

    pub fn update_accum(&mut self, g: &Game, persp: Color) {
        unimplemented!()
    }
}






