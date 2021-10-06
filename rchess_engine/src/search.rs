
use crate::types::*;
use crate::tables::*;

impl Game {

    pub fn search_king(&self, c: Color) -> BitBoard {
        let b0 = self.get(King, c);
        let b1 = b0
            | b0.shift(W)
            | b0.shift(E);
        let b2 = b1
            | b1.shift(N)
            | b1.shift(S);

        let b3 = b2 & !(self.get_color(c));

        b3
    }

    // pub fn search_knight(&self, )

    pub fn search_rooks(&self, ts: &Tables, c: Color) -> Vec<Move> {
        let mut rooks = self.get(Rook, c);
        let mut out = vec![];

        rooks.iter_bitscan(|p0| {
            let ms = ts.rook_moves.get(&p0.into()).unwrap();

            let oc = self.all_occupied();

            for (d,bb) in ms.to_vec().iter() {
                // TODO: needs reverse bitscan
                let blocks = *bb & oc;
                if blocks.0 == 0 {
                    bb.iter_bitscan(|t| {
                        out.push(Move::Quiet { from: p0.into(), to: t.into() });
                    });
                } else {
                    let capture = blocks.bitscan_isolate() & self.get_color(!c);

                    // if capture.0 != 0 {
                    //     out.push(Move::Capture { from: p0.into(), to: capture.bitscan().into() });
                    // }

                    // TODO: mask squares past first block?

                    bb.iter_bitscan(|t| {
                        // unimplemented!()
                    });

                }
            }


        });

        out
        // unimplemented!()
    }

    pub fn search_pawns(&self, c: Color) -> Vec<Move> {
        let ps = self.get(Pawn, c);
        let mut out = vec![];

        let (dir,dw,de) = match c {
            White => (N,NW,NE),
            Black => (S,SW,SE),
        };

        let pushes = ps.shift(dir);
        let pushes = pushes & !(self.all_occupied());

        // let doubles = ps.shift_mult(&vec![dir,dir]);
        // let doubles = doubles & !(self.all_occupied());

        // let b = pushes;
        // eprintln!("{:?}", b);

        pushes.iter_bitscan(|t| {
            let f = (!dir).shift(t);
            out.push(Move::Quiet { from: f.into(), to: t.into() });
        });

        // let captures = ps.shift(dw) | ps.shift(de);
        // let captures = captures & self.get_color(!c);

        // eprintln!("{:?}", ps);

        ps.iter_bitscan(|p0| {
            let f  = BitBoard::index_bit(p0);
            let bb = BitBoard::empty().flip(f);
            let mut cs = (bb.shift(dw) & self.get_color(!c))
                | (bb.shift(de) & self.get_color(!c));
            while cs.0 != 0 {
                let t = cs.bitscan_reset_mut();
                out.push(Move::Capture { from: f, to: t.into() });
            }
        });

        // pushes.serialize()
        // unimplemented!()
        // vec![]
        out
    }

}




