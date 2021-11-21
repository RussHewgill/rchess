
use crate::explore::*;
use crate::opening_book::OpeningBook;
use crate::tables::*;
use crate::types::*;
use crate::evaluate::*;
use crate::alphabeta::*;
// use crate::brain::types::*;
use crate::brain::types::nnue::*;

pub use self::packed_move::*;

use std::collections::HashMap;
use std::path::Path;

use ndarray as nd;
use nd::{Array2};
use ndarray::linalg::Dot;

use serde::{Serialize,Deserialize};

mod packed_move {
    use super::*;
    use packed_struct::prelude::*;

    // #[derive(Debug,Eq,PartialEq,Clone,Copy)]
    #[derive(Debug,Eq,PartialEq,Clone,Copy,PackedStruct)]
    #[packed_struct(bit_numbering = "msb0")]
    pub struct PackedMove {
        #[packed_field(bits = "0..6")]
        from:   Integer<u8, packed_bits::Bits::<6>>,
        #[packed_field(bits = "6..12")]
        to:     Integer<u8, packed_bits::Bits::<6>>,
        #[packed_field(bits = "13..15")]
        prom:   Integer<u8, packed_bits::Bits::<3>>,
    }

    impl PackedMove {
        pub fn new(from: u8, to: u8, prom: Option<Piece>) -> Self {
            Self {
                from:  from.into(),
                to:    to.into(),
                prom:  Self::convert_from_piece(prom).into(),
            }
        }

        fn convert_from_piece(pc: Option<Piece>) -> u8 {
            match pc {
                None         => 0,
                Some(Knight) => 1,
                Some(Bishop) => 2,
                Some(Rook)   => 3,
                Some(Queen)  => 4,
                _            => panic!("PackedMove: bad promotion: {:?}", pc),
            }
        }

        fn convert_to_piece(pc: u8) -> Option<Piece> {
            match pc {
                0 => None,
                1 => Some(Knight),
                2 => Some(Bishop),
                3 => Some(Rook),
                4 => Some(Queen),
                _ => unimplemented!(),
            }
        }

    }

}

mod packed_move2 {
    use super::*;
    use bitvec::prelude::*;

    #[derive(Debug,Eq,PartialEq,Clone,Copy)]
    // pub struct PackedMove(BitArr!(for 16));
    // pub struct PackedMove(BitArray<Lsb0, [u8; 16]>);
    pub struct PackedMove(u16);

    impl PackedMove {
        const FROM: u16 = 0b000_000_111;
        const TO: u16   = 0b000_111_000;
        const PROM: u16 = 0b111_000_000;

        pub fn get(&self) -> u16 {
            self.0
        }
        pub fn empty() -> Self {
            Self(Self::FROM | Self::TO | Self::PROM)
        }

        pub fn new<T: Into<u16>>(from: T, to: T, prom: Option<Piece>) -> Self {
            let mut out = 0;
            out |= Self::FROM & from.into();
            out |= Self::TO & to.into();
            // out |= Self::PROM & prom;
            Self(out)
        }

        pub fn set_from(&mut self, from: u16) {
            self.0 &= !Self::FROM;
            self.0 |= Self::FROM & from;
        }
        pub fn set_to(&mut self, to: u16) {
            self.0 &= !Self::TO;
            self.0 |= Self::TO & (to << 3);
        }
        pub fn set_prom(&mut self, prom: u16) {
            self.0 &= !Self::PROM;
            self.0 |= Self::PROM & (prom << 6);
        }

        pub fn get_from(&self) -> u16 {
            self.0 & Self::FROM
        }
        pub fn get_to(&self) -> u16 {
            (self.0 & Self::TO) >> 3
        }
        pub fn get_prom(&self) -> u16 {
            (self.0 & Self::PROM) >> 6
        }
    }

}

// #[derive(Debug,Eq,PartialEq,Clone,Serialize,Deserialize)]
#[derive(Debug,Eq,PartialEq,Clone)]
pub struct TrainingData2 {
    // pub opening:      Vec<PackedMove>,
}


#[derive(Debug,Eq,PartialEq,Clone,Serialize,Deserialize)]
pub struct TrainingData {
    pub result:   GameEnd,
    pub opening:  Vec<Move>,
    pub moves:    Vec<TDEntry>,
}

#[derive(Debug,Eq,PartialEq,Clone,Copy,Serialize,Deserialize)]
pub struct TDEntry {
    mv:       Move,
    eval:     Score,
}

impl TDEntry {
    pub fn new(mv: Move, eval: Score) -> Self {
        Self {
            mv,
            eval,
        }
    }
}

/// Load, Save
impl TrainingData {

    pub fn save_all<P: AsRef<Path>>(path: P, xs: &[Self]) -> std::io::Result<()> {
        use std::io::Write;
        let mut buf: Vec<u8> = vec![];

        for x in xs.iter() {
            let b: Vec<u8> = bincode::serialize(&x).unwrap();
            buf.extend(b.into_iter());
        }

        let mut file = std::fs::File::open(path)?;
        file.write_all(&buf)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        use std::io::Write;
        let b: Vec<u8> = bincode::serialize(&self).unwrap();
        let mut file = std::fs::File::open(path)?;
        file.write_all(&b)
    }

    pub fn load<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        use std::io::Write;
        let mut b = std::fs::read(path)?;
        let out: Self = bincode::deserialize(&b).unwrap();
        Ok(out)
    }

}

/// Generate
impl TrainingData {

    pub fn generate_single(ts: &Tables, opening: Vec<Move>) -> Self {

        let mut g = Game::from_fen(ts, STARTPOS).unwrap();
        let mut moves = vec![];

        for &mv in opening.iter() {
            g = g.clone().make_move_unchecked(ts, mv).unwrap();
        }

        // let fen = "6k1/4Q3/8/8/8/5K2/8/8 w - - 6 4"; // Queen endgame, #4
        // // let fen = "7k/4Q3/8/4K3/8/8/8/8 w - - 8 5"; // Queen endgame, #2
        // let mut g = Game::from_fen(ts, fen).unwrap();

        // let mut g = g.flip_sides(ts);

        let max_depth = 5;
        let t = 0.5;

        let stop = Arc::new(AtomicBool::new(false));
        let timesettings = TimeSettings::new_f64(0.0,t);
        let mut ex = Explorer::new(g.state.side_to_move, g.clone(), max_depth, stop, timesettings);

        ex.load_syzygy("/home/me/code/rust/rchess/tables/syzygy/").unwrap();

        let mut prevs: HashMap<Zobrist, u8> = HashMap::default();

        debug!("generate_single starting...");
        let result = loop {
            if let (Some((mv,score)),stats) = ex.explore(&ts, None) {
                g = g.clone().make_move_unchecked(ts, mv).unwrap();

                eprintln!("{:?}\n{:?}\n{:?}", mv, g, g.to_fen());
                eprintln!("score.score = {:?}", score.score);

                if score.score > CHECKMATE_VALUE - 100 {
                    break GameEnd::Checkmate { win: !g.state.side_to_move };
                } else if score.score.abs() > CHECKMATE_VALUE - 100 {
                    break GameEnd::Checkmate { win: g.state.side_to_move };
                }

                ex.game = g.clone();
                ex.side = g.state.side_to_move;

                let e = TDEntry::new(mv, score.score);
                moves.push(e);
            } else { break GameEnd::Error; }
        };

        Self {
            result,
            opening,
            moves,
        }
    }

}

pub fn generate_training_data(ts: &Tables, ob: &OpeningBook) -> () {
    let mut ob = ob.clone();

    unimplemented!()
}

impl NNUE {
    pub fn train(&mut self) {
    }
    pub fn train_single(&mut self, g: &Game) {
    }
}

/// Backprop
impl NNUE {

    #[allow(unused_doc_comments)]
    pub fn backprop(&mut self, g: Option<&Game>, correct: i32, eta: i8) -> i32 {

        if let Some(g) = g {
            self.init_inputs(g);
        }

        let (pred, ((act1,act2,act3),(z1,z2,z3,z_out))) = self._run_partial();

        /// XXX: No activation function for last layer, so no derivative
        let delta_out = pred - correct;
        let delta_out: Array2<i8> = nd::array![[delta_out.clamp(-127,127) as i8]];

        /// L4
        let ws4 = delta_out.dot(&act3.t()); // 1,32
        let bs4 = delta_out.clone();

        /// L3
        let sp3 = z3.map(Self::act_d);

        let mut d3 = self.weights_4.t().dot(&delta_out); // 32,1
        d3 *= &sp3;

        let ws3 = d3.dot(&act2.t()); // 32,32
        let bs3 = d3.clone();        // 1,1

        /// L2
        let sp2 = z2.map(Self::act_d);

        let mut d2 = self.weights_3.t().dot(&d3); // 32,1
        d2 *= &sp2;

        let ws2 = d2.dot(&act1.t()); // 32,512
        let bs2 = d2.clone();        // 32,1

        let sp1_own: Array2<i16> = self.activations_own.map(Self::act_d); // 256,1
        let sp1_own = sp1_own.map(|x| (*x).clamp(-127, 127) as i8);       // 256,1

        let sp1_other: Array2<i16> = self.activations_other.map(Self::act_d); // 256,1
        let sp1_other = sp1_other.map(|x| (*x).clamp(-127, 127) as i8);       // 256,1

        let d1 = self.weights_2.t().dot(&d2); // 512,1
        let d1_own0: nd::ArrayView2<i8> = d1.slice(nd::s![..256, ..]); // 256, 1
        let d1_own0 = &d1_own0 * &sp1_own;

        let d1_other0: nd::ArrayView2<i8> = d1.slice(nd::s![256.., ..]); // 256, 1
        let d1_other0 = &d1_other0 * &sp1_other;

        let d1_own  = sprs::CsMat::csc_from_dense(d1_own0.view(), 0);
        let d1_other = sprs::CsMat::csc_from_dense(d1_other0.view(), 0);

        let ws1_own = &d1_own * &self.inputs_own.transpose_view();
        let ws1_other = &d1_other * &self.inputs_other.transpose_view();

        self.biases_1_own   -= &d1_own0.map(|x| *x as i16);
        self.biases_1_other -= &d1_other0.map(|x| *x as i16);

        // self.weights_1_own   -= &(ws1_own / eta);
        // self.weights_1_other -= &(ws1_other / eta);

        // println!("wat 0 in {:.3} seconds", t0.elapsed().as_secs_f64());
        // let t0 = std::time::Instant::now();
        // println!("wat 0 in {:.3} seconds", t0.elapsed().as_secs_f64());

        for (c,cv) in ws1_own.outer_iterator().enumerate() {
            for (r,rv) in cv.iter() {
                self.weights_1_own[(r,c)] -= rv / eta;
            }
        }
        for (c,cv) in ws1_other.outer_iterator().enumerate() {
            for (r,rv) in cv.iter() {
                self.weights_1_other[(r,c)] -= rv / eta;
            }
        }

        self.weights_2 -= &(ws2 / eta);
        self.weights_3 -= &(ws3 / eta);
        self.weights_4 -= &(ws4 / eta);

        self.biases_2 -= &bs2.map(|x| *x as i32);
        self.biases_3 -= &bs3.map(|x| *x as i32);
        self.biases_4 -= &bs4.map(|x| *x as i32);

        pred
    }

}






