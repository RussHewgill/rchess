
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
            // let ms: &MoveSetRook = ts.rook_moves.get(&p0.into()).unwrap();
            let ms: &MoveSetRook = ts.get_rook(p0.into());

            let occ = self.all_occupied();

            for (dir,moves) in ms.to_vec().iter() {
                match dir {
                    N | E => {
                        let blocks = *moves & occ;
                        if blocks.0 != 0 {
                            let square = blocks.bitscan_isolate();
                            let sq: Coord = square.bitscan().into();
                            let nots = ts.get_rook(sq).get_dir(*dir);
                            let mm = *moves ^ *nots;
                            let mm = mm & !square;
                            if (square & self.get_color(!c)).0 != 0 {
                                // capture
                                out.push(Move::Capture { from: p0.into(), to: sq });
                            }
                            mm.iter_bitscan(|t| {
                                out.push(Move::Quiet { from: p0.into(), to: t.into() });
                            });
                        } else {
                            moves.iter_bitscan(|t| {
                                out.push(Move::Quiet { from: p0.into(), to: t.into() });
                            });
                        }
                    },
                    S | W => {
                        let blocks = *moves & occ;
                        if blocks.0 != 0 {
                            let square = blocks.bitscan_rev_isolate();
                            let sq: Coord = square.bitscan_rev().into();
                            let nots = ts.get_rook(sq).get_dir(*dir);
                            let mm = *moves ^ *nots;
                            let mm = mm & !square;
                            if (square & self.get_color(!c)).0 != 0 {
                                // capture
                                out.push(Move::Capture { from: p0.into(), to: sq });
                            }
                            mm.iter_bitscan_rev(|t| {
                                out.push(Move::Quiet { from: p0.into(), to: t.into() });
                            });
                        } else {
                            moves.iter_bitscan_rev(|t| {
                                out.push(Move::Quiet { from: p0.into(), to: t.into() });
                            });
                        }
                    },
                    _ => panic!("search_rooks: Diagonal rook?")
                }

            }

            //     // TODO: needs reverse bitscan
            //     let blocks = *moves & occ;
            //     if blocks.0 == 0 {
            //         moves.iter_bitscan(|t| {
            //             out.push(Move::Quiet { from: p0.into(), to: t.into() });
            //         });
            //     } else {
            //         match dir {
            //             N | E => {
            //                 let capture = blocks.bitscan_isolate() & self.get_color(!c);
            //                 if capture.0 != 0 {
            //                     // let capture_sq = capture.bitscan_isolate().0.next_power_of_two() - 1;
            //                     // println!("capture_sq = {:?}", BitBoard(capture_sq));
            //                     // let ms = *moves & BitBoard(capture_sq);
            //                     eprintln!("{:?}", moves);
            //                 }

            //                 moves.iter_bitscan(|t| {
            //                     // unimplemented!()
            //                 });
            //             },
            //             S | W => {
            //             },
            //             _     => panic!("diagonal rook?"),
            //         }

            //         // if capture.0 != 0 {
            //         //     out.push(Move::Capture { from: p0.into(), to: capture.bitscan().into() });
            //         // }

            //         // TODO: mask squares past first block?


            //     }
            // }


        });

        out
        // unimplemented!()
    }

    pub fn search_bishops(&self, c: Color) -> Vec<Move> {
        unimplemented!()
    }

    pub fn search_knights(&self, c: Color) -> Vec<Move> {
        unimplemented!()
    }

    pub fn search_queen(&self, c: Color) -> Vec<Move> {
        unimplemented!()
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




