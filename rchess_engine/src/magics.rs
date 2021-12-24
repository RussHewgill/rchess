
use serde::{Serialize,Deserialize};

use itertools::iproduct;
use rand::{prelude::{StdRng,SliceRandom},Rng,SeedableRng};

use crate::types::*;
use crate::tables::*;

// #[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
#[derive(Default,Serialize,Deserialize,Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
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

    // pub fn index(mask: BitBoard, magic: BitBoard, shift: u32, occ: BitBoard) -> u64 {
    //     // unsigned lo = unsigned(occupied) & unsigned(mask);
    //     let lo = occ.0 & mask.0;
    //     // unsigned hi = unsigned(occupied >> 32) & unsigned(mask >> 32);
    //     let hi = occ.0.overflowing_shr(32).0 & mask.0.overflowing_shr(32).0;
    //     // return (lo * unsigned(magic) ^ hi * unsigned(magic >> 32)) >> shift;
    //     let k0 = lo.overflowing_mul(magic.0.overflowing_pow(hi as u32).0).0
    //         .overflowing_mul(magic.0.overflowing_shr(32).0).0;
    //     let k1 = k0.overflowing_shr(shift).0;
    //     k1
    // }

}

pub fn gen_magics(bishop: bool)
                        -> std::result::Result<([Magic; 64], [BitBoard; 0x1480]),
                                               ([Magic; 64], [BitBoard; 0x19000])>
{
    #[cfg(target_feature = "bmi2")]
    return _gen_magics_pext(bishop);
    #[cfg(not(target_feature = "bmi2"))]
    return _gen_magics(bishop);
}

pub fn _gen_magics_pext(bishop: bool)
                        -> std::result::Result<([Magic; 64], [BitBoard; 0x1480]),
                                               ([Magic; 64], [BitBoard; 0x19000])>
{
    let mut rng: StdRng = SeedableRng::seed_from_u64(1234u64);

    let mut reference: [BitBoard; 4096] = [BitBoard::empty(); 4096];
    let mut occupancy: [BitBoard; 4096] = [BitBoard::empty(); 4096];

    let mut table_b: [BitBoard; 0x1480]  = [BitBoard::empty(); 0x1480];
    let mut table_r: [BitBoard; 0x19000] = [BitBoard::empty(); 0x19000];

    // let mut magics: [Option<Magic>; 64] = [None; 64];
    let mut magics: [Magic; 64] = [Magic::default(); 64];

    let (r1bb,r8bb) = (BitBoard::mask_rank(0),BitBoard::mask_rank(7));
    let (f1bb,f8bb) = (BitBoard::mask_file(0),BitBoard::mask_file(7));
    let mut epoch = [0; 4096];
    // let mut epoch = [0; 65536];
    let mut cnt   = 0;
    let mut size: usize = 0;

    for sq in 0u8..64 {
        let c0 = Coord::new_int(sq);

        let edges: BitBoard =
            ((BitBoard::mask_rank(0) | BitBoard::mask_rank(7)) & !BitBoard::mask_rank(c0.rank()))
            | ((BitBoard::mask_file(0) | BitBoard::mask_file(7)) & !BitBoard::mask_file(c0.file()));

        // let mut m: &mut Magic = &mut magics[sq as usize];
        let mut m = Magic::default();


        // let mask = if bishop {
        m.mask = if bishop {
            Tables::gen_blockermask_bishop(c0) & !edges
        } else {
            Tables::gen_blockermask_rook(c0) & !edges
        };

        // let mut attacks_idx: usize = if sq == 0 {
        m.attacks = if sq == 0 {
            0
        } else {
            magics[sq as usize - 1].attacks + size
        };

        let mut b = BitBoard(0);
        size = 0;
        loop {
            occupancy[size] = b;
            reference[size] = Tables::gen_moveboard(b, c0, bishop);

            // #[cfg(target_feature = "bmi2")]
            if bishop {
                table_b[m.attacks + unsafe { pext(b.0, m.mask.0) } as usize] = reference[size];
            } else {
                table_r[m.attacks + unsafe { pext(b.0, m.mask.0) } as usize] = reference[size];
            }

            size += 1;
            b.0 = (b.0 - m.mask.0) & m.mask.0;
            if b.is_empty() { break; }
        }

        magics[sq as usize] = m;

    }

    if bishop {
        Ok((magics, table_b))
    } else {
        Err((magics, table_r))
    }
}

#[allow(unreachable_code)]
pub fn _gen_magics(bishop: bool)
                    -> std::result::Result<([Magic; 64], [BitBoard; 0x1480]),
                                            ([Magic; 64], [BitBoard; 0x19000])>
{
    // let mut rng = rand::thread_rng();
    let mut rng: StdRng = SeedableRng::seed_from_u64(1234u64);

    let mut reference: [BitBoard; 4096] = [BitBoard::empty(); 4096];
    let mut occupancy: [BitBoard; 4096] = [BitBoard::empty(); 4096];

    let mut table_b: [BitBoard; 0x1480]  = [BitBoard::empty(); 0x1480];
    let mut table_r: [BitBoard; 0x19000] = [BitBoard::empty(); 0x19000];

    let mut magics: [Option<Magic>; 64] = [None; 64];
    let (r1bb,r8bb) = (BitBoard::mask_rank(0),BitBoard::mask_rank(7));
    let (f1bb,f8bb) = (BitBoard::mask_file(0),BitBoard::mask_file(7));
    let mut epoch = [0; 4096];
    // let mut epoch = [0; 65536];
    let mut cnt   = 0;
    let mut size: usize = 0;

    // let n1r = vec![
    //     // "B7".into(),
    // ];
    // let n1b = vec![
    // ];

    // println!("wat 0");
    for sq in 0u8..64 {
    // for sq in 0u8..1 {
        // let c0: Coord = "B7".into();
        // let sq: u32   = c0.into();
        // eprintln!("sq = {:?}", sq);
        // eprintln!("c0 = {:?}", c0);

        // let c0: Coord = sq.into();
        let c0 = Coord::new_int(sq);

        let edges: BitBoard =
            ((BitBoard::mask_rank(0) | BitBoard::mask_rank(7)) & !BitBoard::mask_rank(c0.rank()))
            | ((BitBoard::mask_file(0) | BitBoard::mask_file(7)) & !BitBoard::mask_file(c0.file()));

        let mask = if bishop {
            Tables::gen_blockermask_bishop(c0) & !edges
        } else {
            Tables::gen_blockermask_rook(c0) & !edges
        };

        // let shift = 64 - mask.popcount();

        // #[cfg(not(target_feature = "bmi2"))]
        let mut attacks: usize = if sq == 0 {
            0
        } else {
            magics[sq as usize - 1].unwrap().attacks + size
        };

        // let mut attacks: &mut [BitBoard] = if sq == 0 {
        //     &mut (if bishop { table_b[..] } else { table_r[..] } )
        // } else {
        //     let idx = magics[sq as usize - 1].unwrap().attacks + size;
        //     &mut (if bishop { table_b[idx..] } else { table_r[idx..] } )
        // };

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

        // let mut b = 0;
        // size = 0;
        // loop {
        //     occupancy[size] = BitBoard(b);
        //     reference[size] = Tables::gen_moveboard(BitBoard(b), c0, bishop);
        //     #[cfg(target_feature = "bmi2")]
        //     if b == 0 { break; }
        // }

        // #[cfg(target_feature = "bmi2")]
        // continue;

        for (s,b) in mbs.iter().enumerate() {
            if bishop {
                reference[s] = Tables::gen_moveboard_bishop(*b, c0);
                // #[cfg(target_feature = "bmi2")]
                // {
                //     table_b[pext(b, )]
                // }
            } else {
                reference[s] = Tables::gen_moveboard_rook(*b, c0);
            }
            size = s + 1;
        }
        let mut mm: u64;

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

            };

            if done {
                break 'outer
            }

        }

        if mm == 0 {
            panic!("wot");
        }

        let m = Magic::new(attacks, mask, BitBoard(mm), shift);
        magics[sq as usize] = Some(m);

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

unsafe fn pext(a: u64, mask: u64) -> u64 {
    #[cfg(not(target_feature = "bmi2"))]
    panic!("pext but no bmi2 instructions");
    #[cfg(target_feature = "bmi2")]
    core::arch::x86_64::_pext_u64(a, mask)
}

impl Tables {

    // #[cfg(feature = "nope")]
    pub fn attacks_rook(&self, c0: Coord, occ: BitBoard) -> BitBoard {
        let m = self.magics_rook[c0];
        #[cfg(target_feature = "bmi2")]
        unsafe {
            let idx = pext(occ.0, m.mask.0);
            self.table_rook[m.attacks + idx as usize]
        }
        #[cfg(not(target_feature = "bmi2"))]
        {
            let mut occ = occ;
            let occ = (occ & m.mask).0;
            let occ = occ.overflowing_mul(m.magic.0).0;
            let occ = occ.overflowing_shr(m.shift as u32).0;
            self.table_rook[m.attacks + occ as usize]
        }
    }

    // #[cfg(feature = "nope")]
    pub fn attacks_bishop(&self, c0: Coord, occ: BitBoard) -> BitBoard {
        let m = self.magics_bishop[c0];
        #[cfg(target_feature = "bmi2")]
        unsafe {
            let idx = pext(occ.0, m.mask.0);
            self.table_bishop[m.attacks + idx as usize]
        }
        #[cfg(not(target_feature = "bmi2"))]
        {
            let mut occ = occ;
            let occ = (occ & m.mask).0;
            let occ = occ.overflowing_mul(m.magic.0).0;
            let occ = occ.overflowing_shr(m.shift as u32).0;
            self.table_bishop[m.attacks + occ as usize]
        }
    }

    #[cfg(feature = "nope")]
    pub fn attacks_rook(&self, c0: Coord, occ: BitBoard) -> BitBoard {
        let sq: u32 = c0.into();
        let m = self.magics_rook[sq as usize];
        // if m.magic.0 == 0 {
        //     trace!("Magics not initialized");
        //     return BitBoard::empty();
        // }
        let mut occ = occ;
        let occ = (occ & m.mask).0;
        let occ = occ.overflowing_mul(m.magic.0).0;
        let occ = occ.overflowing_shr(m.shift as u32).0;
        self.table_rook[m.attacks + occ as usize]
    }

    #[cfg(feature = "nope")]
    pub fn attacks_bishop(&self, c0: Coord, occ: BitBoard) -> BitBoard {
        let sq: u32 = c0.into();
        let m = self.magics_bishop[sq as usize];
        let mut occ = occ;
        let occ = (occ & m.mask).0;
        let occ = occ.overflowing_mul(m.magic.0).0;
        let occ = occ.overflowing_shr(m.shift as u32).0;
        self.table_bishop[m.attacks + occ as usize]
    }

    fn sparse_rand(rng: &mut StdRng) -> u64 {
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
        let b1 = BitBoard::mask_file(c0.file())
            | BitBoard::mask_rank(c0.rank());
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

    pub fn gen_moveboard(occ: BitBoard, c0: Coord, bishop: bool) -> BitBoard {
        if bishop {
            Tables::gen_moveboard_bishop(occ, c0)
        } else {
            Tables::gen_moveboard_rook(occ, c0)
        }
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


