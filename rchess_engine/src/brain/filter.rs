
use crate::types::*;

use itertools::Itertools;
use ndarray::prelude::*;
use ndarray::concatenate;

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

pub struct ConvFilter {
    filter: Array3<u16>,
}

impl ConvFilter {
    pub fn new(filter: Array2<u16>) -> Self {
        let filter = filter.insert_axis(Axis(2));
        Self {
            filter,
        }
    }
}

pub fn bitboard_to_arr(bb: BitBoard) -> Array2<u16> {
    let mut out = Array2::zeros((8,8));
    bb.into_iter().for_each(|sq| {
        let c0: Coord = sq.into();
        // XXX: flipping make it wrong, but print correctly
        // out[[7 - c0.0 as usize, c0.1 as usize]] = 1;
        out[[c0.0 as usize, c0.1 as usize]] = 1;
    });
    out
}

impl ConvFilter {

    pub fn bitboard_section((x,y): (usize,usize), bb: BitBoard) -> [u16; 9] {
        // let mask = FILTER_MASKS[x][y];
        // let b = bb.0 & mask;

        // let mut out = [0; 9];
        // let mut mask = 1u64;
        // for x in 0..9 {
        //     let m = mask << x;
        //     // eprintln!("mask {} = {:#012b}, {:#012b}", x, m, b & m);
        //     // out[x] = BitBoard(b & m).bitscan() as u16;
        // }

        // out
        unimplemented!()
    }

    pub fn scan(&self, input: Array3<u16>) -> Array2<u16> {
        let mut out: Array2<u16> = Array2::zeros((6,6));
        let s = input.shape();

        for x in 0..s[0]-2 {
            for y in 0..s[1]-2 {
                let slice: ArrayView3<u16> = input.slice(s![x..x+3, y..y+3, ..]);
                let k = &self.filter * &slice;
                out[[x,y]] = k.sum();
            }
        }

        out
    }

    pub fn scan_bitboard(&self, bbs: &[BitBoard]) -> Array2<u16> {
        let mut out: Array2<u16> = Array2::zeros((6,6));

        // let bb2 = bitboard_to_arr(bb);

        let mut bb: Array3<u16> = Array3::zeros((8,8,0));
        bbs.into_iter()
            .map(|x| bitboard_to_arr(*x))
            .for_each(|b| {
                let b2 = b.insert_axis(Axis(2));
                bb.append(Axis(2), b2.view()).unwrap();
            });

        // let ff = self.filter.to_shape(9).unwrap();

        // let s = bb.shape();
        // eprintln!("s = {:?}", s);

        // let xx = s[0];

        // eprintln!("bb2 = {:?}", bb2);

        for x in 0..6 {
            for y in 0..6 {

                let slice: ArrayView3<u16> = bb.slice(s![x..x+3, y..y+3, ..]);
                // let slice = slice.to_shape(9).unwrap();

                // let k = self.filter.dot(&slice);
                let k = &self.filter * &slice;

                // eprintln!("k = {:?}", k.sum());

                // eprintln!("k = {:?}", k);
                out[[x,y]] = k.sum();
            }
        }

        out
        // unimplemented!()
    }

    // pub fn as_arr(&self) -> [u16; 9] {
    //     [self.filter[0][0],
    //       self.filter[1][0],
    //       self.filter[2][0],
    //       self.filter[0][1],
    //       self.filter[1][1],
    //       self.filter[2][1],
    //       self.filter[0][2],
    //       self.filter[1][2],
    //       self.filter[2][2],
    //     ]
    // }

    // pub fn dot(&self, other: [u16; 9]) -> u16 {
    //     self.as_arr().iter().zip(other.iter()).map(|(a,b)| a * b).sum()
    // }

}



