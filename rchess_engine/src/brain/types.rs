
use crate::tables::*;
use crate::types::*;
use crate::evaluate::*;

use crate::brain::filter::*;


use ndarray::prelude::*;



pub struct ConvNetwork {
    filters:      Vec<ConvFilter>,
}


impl ConvNetwork {

    pub fn run(&self, ts: &Tables, g: &Game) -> Score {
        let bbs = Self::split_bitboards(g);

        let mut conv_layer = Array3::zeros((6,6,0));
        for filt in self.filters.iter() {
            let x = filt.scan_bitboard(&bbs);
            let x = x.insert_axis(Axis(2));
            conv_layer.append(Axis(2), x.view()).unwrap();
        }

        unimplemented!()
    }

}

impl ConvNetwork {

    /// 0-6:      own pieces
    /// 6-12:     other pieces
    fn split_bitboards(g: &Game) -> Vec<BitBoard> {
        let mut out = vec![];

        let sides = if g.state.side_to_move == White { [White,Black] } else { [Black,White] };

        for side in sides {
            for pc in Piece::iter_pieces() {
                out.push(g.get(pc, side));
            }
        }

        out
    }

}






