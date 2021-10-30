
use serde::{Serialize,Deserialize};

use itertools::iproduct;
use rand::Rng;

use crate::types::*;
use crate::tables::*;

// #[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
#[derive(Serialize,Deserialize,Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct Magic {
    pub attacks:   usize,
    pub mask:      BitBoard,
    pub magic:     BitBoard,
    pub shift:     u8,
}

impl Magic {

    pub fn new(attacks: usize, mask: BitBoard, magic: BitBoard, shift: u8) -> Self {
        Self {
            attacks,
            // mask: BitBoard(0xff818181818181ff),
            mask,
            magic,
            shift,
        }
    }

    pub fn index(mask: BitBoard, magic: BitBoard, shift: u32, occ: BitBoard) -> u64 {
        // unsigned lo = unsigned(occupied) & unsigned(mask);
        let lo = occ.0 & mask.0;
        // unsigned hi = unsigned(occupied >> 32) & unsigned(mask >> 32);
        let hi = occ.0.overflowing_shr(32).0 & mask.0.overflowing_shr(32).0;

        // return (lo * unsigned(magic) ^ hi * unsigned(magic >> 32)) >> shift;
        let k0 = lo.overflowing_mul(magic.0.overflowing_pow(hi as u32).0).0
            .overflowing_mul(magic.0.overflowing_shr(32).0).0;
        let k1 = k0.overflowing_shr(shift).0;

        k1
    }

}

pub fn _gen_magics(bishop: bool)
                    -> std::result::Result<([Magic; 64], [BitBoard; 0x1480]),
                                            ([Magic; 64], [BitBoard; 0x19000])>
{
    let mut rng = rand::thread_rng();
    let mut reference: [BitBoard; 4096] = [BitBoard::empty(); 4096];

    let mut table_b: [BitBoard; 0x1480]  = [BitBoard::empty(); 0x1480];
    let mut table_r: [BitBoard; 0x19000] = [BitBoard::empty(); 0x19000];

    let mut magics: [Option<Magic>; 64] = [None; 64];
    let (r1bb,r8bb) = (BitBoard::mask_rank(0),BitBoard::mask_rank(7));
    let (f1bb,f8bb) = (BitBoard::mask_file(0),BitBoard::mask_file(7));
    // let mut epoch = [0; 4096];
    let mut epoch = [0; 65536];
    let mut cnt   = 0;
    let mut size: usize = 0;

    // let n1r = vec![
    //     // "B7".into(),
    // ];
    // let n1b = vec![
    // ];

    for sq in 0u8..64 {
    // for sq in 0u8..1 {
        // let c0: Coord = "B7".into();
        // let sq: u32   = c0.into();
        // eprintln!("sq = {:?}", sq);
        // eprintln!("c0 = {:?}", c0);

        let c0: Coord = sq.into();

        let edges: BitBoard =
            ((BitBoard::mask_rank(0) | BitBoard::mask_rank(7)) & !BitBoard::mask_rank(c0.1 as u8))
            | ((BitBoard::mask_file(0) | BitBoard::mask_file(7)) & !BitBoard::mask_file(c0.0 as u8));

        let mask = if bishop {
            Tables::gen_blockermask_bishop(c0) & !edges
        } else {
            Tables::gen_blockermask_rook(c0) & !edges
        };

        // let shift = 64 - mask.popcount();

        let attacks = if sq == 0 {
            0
        } else {
            magics[sq as usize - 1].unwrap().attacks + size
        };

        let mbs = mask.iter_subsets();

        // let n = if bishop && n1b.contains(&c0) {
        //     debug!("bishop magic N-1: {:?}", c0);
        //     mask.popcount() - 1
        // } else if !bishop && n1r.contains(&c0) {
        //     debug!("rook magic N-1: {:?}", c0);
        //     mask.popcount() - 1
        // } else {
        //     mask.popcount()
        // };

        for (s,b) in mbs.iter().enumerate() {
            if bishop {
                reference[s] = Tables::gen_moveboard_bishop(*b, c0);
            } else {
                reference[s] = Tables::gen_moveboard_rook(*b, c0);
            }
            size = s + 1;
        }
        let mut mm: u64;

        // let mut n1s = vec![];
        // let mut n = mask.popcount() - 1;
        // let mut n1 = true;

        let n = mask.popcount();
        let shift = 64 - n;

        let t0 = std::time::Instant::now();

        // let mut xs = vec![];
        let mut done;
        'outer: loop {
        // 'outer: for _ in 0..1_000_000 {
            // mm = rng.gen();
            // mm = 0x48FFFE99FECFAA00;
            // mm = 0x90a207c5e7ae23ff;

            // if t0.elapsed().as_secs_f64() > 1.0 {
            //     n = n + 1;
            //     n1 = false;
            //     epoch = [0; 65536];
            //     shift = 64 - n;
            // }

            loop {
                mm = Tables::sparse_rand(&mut rng);
                let k0 = mm.overflowing_mul(mask.0).0;
                let k1 = k0.overflowing_shr(56).0;
                if BitBoard(k1).popcount() < 6 {
                    break;
                }
            }

            done = true;
            cnt += 1;
            'inner: for (s,b) in mbs.iter().enumerate() {
                let result = reference[s];

                let idx = b.0.overflowing_mul(mm).0;
                let idx = idx.overflowing_shr(shift as u32).0 as usize;

                let tb = if bishop {
                    table_b[attacks + idx]
                } else {
                    table_r[attacks + idx]
                };
                if epoch[idx] < cnt {
                    epoch[idx] = cnt;
                    if bishop {
                        table_b[attacks + idx] = result;
                    } else {
                        table_r[attacks + idx] = result;
                    }
                    // xs.push(attacks + idx);
                } else if tb.0 != result.0 {
                    done = false;
                    break 'inner;
                }

                // let tb = table[attacks + idx];
                // if tb.is_empty() {
                //     table[attacks + idx] = result;
                //     xs.push(attacks + idx);
                // } else if tb.0 != result.0 {
                //     done = false;
                //     break 'inner;
                // }

            };

            if done {
                break 'outer
            }

            // if done {
            //     break 'outer
            // } else {
            //     for i in xs.iter() {
            //         table[*i] = BitBoard::empty();
            //     }
            //     xs.clear()
            // }

        }

        // if n1 {
        //     debug!("Found N-1 for {:?}", c0);
        //     n1s.push((n1, sq, c0));
        // } else {
        //     debug!("Found only N for {:?}", c0);
        // }

        if mm == 0 {
            panic!("wot");
        }

        let m = Magic::new(attacks, mask, BitBoard(mm), shift);
        magics[sq as usize] = Some(m);

        // for (idx, result) in results.into_iter() {
        //     table[idx] = result;
        // }

        // eprintln!("n = {:?}", n);
        // eprintln!("2^n = {:?}", 2u64.pow(n));

        // eprintln!("mbs.len() = {:?}", mbs.len());

        // let mbs = vec![
        //     BitBoard::new(&["A6","D1"]),
        // ];

        // let mut rng = rand::thread_rng();
        // let mut mm: u64;
        // for (s,b) in mbs.iter().enumerate() {

            // loop {
            //     mm = rng.gen();
            //     let k0 = b.0.overflowing_mul(mm).0;
            //     let k1 = 2u64.pow(mask.popcount());
            //     let k2 = 64u64.overflowing_sub(k1).0;
            //     let k3 = k0.overflowing_shr(k2 as u32).0;
            //     if k3 != 0 { break; }
            // }

            // // eprintln!("magic? = {:?}", BitBoard(mm));
            // m.magic = BitBoard(mm);
            // cnt += 1;
            // for i in 0..size {
            //     let idx = Magic::index(mask, BitBoard(mm), shift, occupancy[i]) as usize;
            //     if epoch[idx] < cnt {
            //         epoch[idx] = cnt;
            //         table[attacks + idx] = reference[i];
            //     } else if table[attacks + idx] != reference[i] {
            //         break;
            //     }
            // }

        // }

    }

    // let magics: [Magic; 64] = array_init::array_init(|x| magics[x].unwrap());
    let magics: [Magic; 64] = array_init::array_init(
        |x| magics[x].unwrap_or(Magic::new(0, BitBoard::empty(), BitBoard::empty(), 0))
    );

    if bishop {
        Ok((magics, table_b))
    } else {
        Err((magics, table_r))
    }

    // unimplemented!()

}


impl Tables {

    pub fn attacks_rook(&self, c0: Coord, occ: BitBoard) -> BitBoard {
        let sq: u32 = c0.into();
        let m = self.magics_rook[sq as usize];
        if m.magic.0 == 0 {
            trace!("Magics not initialized");
            return BitBoard::empty();
        }
        let mut occ = occ;
        let occ = (occ & m.mask).0;
        let occ = occ.overflowing_mul(m.magic.0).0;
        let occ = occ.overflowing_shr(m.shift as u32).0;
        self.table_rook[m.attacks + occ as usize]
    }

    pub fn attacks_bishop(&self, c0: Coord, occ: BitBoard) -> BitBoard {
        let sq: u32 = c0.into();
        let m = self.magics_bishop[sq as usize];
        if m.magic.0 == 0 {
            trace!("Magics not initialized");
            return BitBoard::empty();
        }
        let mut occ = occ;
        let occ = (occ & m.mask).0;
        let occ = occ.overflowing_mul(m.magic.0).0;
        let occ = occ.overflowing_shr(m.shift as u32).0;
        self.table_bishop[m.attacks + occ as usize]
    }

    fn sparse_rand(rng: &mut rand::rngs::ThreadRng) -> u64 {
        let x0: u64 = rng.gen();
        let x1: u64 = rng.gen();
        let x2: u64 = rng.gen();
        // let x3: u64 = rng.gen();
        // x0 & x1 & x2 & x3
        x0 & x1 & x2
    }

    pub fn gen_magics() -> (([Magic; 64], [BitBoard; 0x1480]), ([Magic; 64], [BitBoard; 0x19000])) {

        let (magics_b,table_b) = _gen_magics(true).unwrap();
        if let Err((magics_r,table_r)) = _gen_magics(false) {
            ((magics_b,table_b),(magics_r,table_r))
        } else { panic!("gen_magics") }

        // unimplemented!()
    }

    pub fn gen_blockermask_rook(c0: Coord) -> BitBoard {
        // let b0 = BitBoard(0xff818181818181ff);
        let b1 = BitBoard::mask_file(c0.0 as u8)
            | BitBoard::mask_rank(c0.1 as u8);
        // (!b0 & b1).set_zero(c0)
        b1.set_zero(c0)
    }

    pub fn gen_blockermask_bishop(c0: Coord) -> BitBoard {
        // let b0 = BitBoard(0xff818181818181ff);

        let b1 = Self::gen_diagonal(c0, true)
            | Self::gen_diagonal(c0, false)
            | Self::gen_antidiagonal(c0, true)
            | Self::gen_antidiagonal(c0, false);

        // (!b0 & b1).set_zero(c0)
        b1.set_zero(c0)
    }

    pub fn gen_moveboard_rook(occ: BitBoard, c0: Coord) -> BitBoard {
        let mut out = BitBoard::empty();
        let ds_rook   = [N,E,W,S];
        for d in ds_rook.iter() {
            let mut c1 = c0;
            loop {
                if let Some(c2) = d.shift_coord(c1) {
                    if (c1.square_dist(c2) <= 2) & occ.is_zero_at(c1) {
                        out.set_one_mut(c2);
                        c1 = c2;
                    } else { break; }
                } else { break; }
            }
        }
        out
    }

    pub fn gen_moveboard_bishop(occ: BitBoard, c0: Coord) -> BitBoard {
        let mut out = BitBoard::empty();
        let ds_bishop = [NE,NW,SE,SW];
        for d in ds_bishop.iter() {
            let mut c1 = c0;
            loop {
                if let Some(c2) = d.shift_coord(c1) {
                    if (c1.square_dist(c2) <= 2) & occ.is_zero_at(c1) {
                        out.set_one_mut(c2);
                        c1 = c2;
                    } else { break; }
                } else { break; }
            }
        }
        out
    }

    fn gen_blockerboard(blockermask: BitBoard, index: usize) -> BitBoard {
        unimplemented!()
    }

}


