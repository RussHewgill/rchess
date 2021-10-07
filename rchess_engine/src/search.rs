
use crate::types::*;
use crate::tables::*;

impl Game {
    pub fn search_all(&self, ts: &Tables, c: Color) -> Vec<Move> {

        let mut out = vec![];

        out.extend(&self.search_king(c));
        out.extend(&self.search_rooks(&ts, c));
        // out.extend(&self.search_bishops(&ts, c));
        // out.extend(&self.search_knights(&ts, c));
        // out.extend(&self.search_queen(&ts, c));
        out.extend(&self.search_pawns(c));

        out
    }
}

impl Game {

    pub fn search_king(&self, c: Color) -> Vec<Move> {
        let b0 = self.get(King, c);
        let b1 = b0
            | b0.shift(W)
            | b0.shift(E);
        let b2 = b1
            | b1.shift(N)
            | b1.shift(S);

        let b3 = b2 & !(self.get_color(c));

        let oc = self.all_occupied();
        let quiets   = b3 & !oc;
        let captures = b3 & oc;

        let mut out = vec![];

        quiets.iter_bitscan(|sq| {
            out.push(Move::Quiet { from: b0.bitscan().into(), to: sq.into()});
        });

        captures.iter_bitscan(|sq| {
            out.push(Move::Capture { from: b0.bitscan().into(), to: sq.into()});
        });

        // b3
        out
    }

    // pub fn search_knight(&self, )

    pub fn search_rooks(&self, ts: &Tables, c: Color) -> Vec<Move> {
        let mut rooks = self.get(Rook, c);
        let mut out = vec![];
        let occ = self.all_occupied();

        rooks.iter_bitscan(|p0| {
            // let ms: &MoveSetRook = ts.rook_moves.get(&p0.into()).unwrap();
            let ms: &MoveSetRook = ts.get_rook(p0.into());

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

        });

        out
    }

    pub fn search_bishops(&self, ts: &Tables, c: Color) -> Vec<Move> {
        let bishops = self.get(Bishop, c);
        let mut out = vec![];
        let occ = self.all_occupied();

        bishops.iter_bitscan(|p0| {
            let ms: &MoveSetBishop = ts.get_bishop(p0.into());

            for (dir,moves) in ms.to_vec().iter() {
                match dir {
                    NE | NW => {
                        let blocks = *moves & occ;
                        if blocks.0 != 0 {
                            let square = blocks.bitscan_isolate();
                            let sq: Coord = square.bitscan().into();
                            let nots = ts.get_bishop(sq).get_dir(*dir);
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
                    SE | SW => {
                        let blocks = *moves & occ;
                        if blocks.0 != 0 {
                            let square = blocks.bitscan_rev_isolate();
                            let sq: Coord = square.bitscan_rev().into();
                            let nots = ts.get_bishop(sq).get_dir(*dir);
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
                    _ => panic!("MoveSetBishop::get Rank or File Bishop?")
                }
            }

        });

        out
    }

    pub fn search_knights(&self, ts: &Tables, c: Color) -> Vec<Move> {
        let ks = self.get(Knight, c);
        let mut out = vec![];
        let oc = self.all_occupied();

        ks.iter_bitscan(|sq| {
            let ms = ts.get_knight(sq.into());

            let quiets   = *ms & !oc;
            let captures = *ms & self.get_color(!c);

            quiets.iter_bitscan(|sq| {
                out.push(Move::Quiet { from: sq.into(), to: sq.into()});
            });

            captures.iter_bitscan(|sq| {
                out.push(Move::Capture { from: sq.into(), to: sq.into()});
            });

        });

        out
    }

    pub fn search_queen(&self, ts: &Tables, c: Color) -> Vec<Move> {
        unimplemented!()
    }

    pub fn search_pawns(&self, c: Color) -> Vec<Move> {
        let ps = self.get(Pawn, c);
        let mut out = vec![];
        let oc = self.all_occupied();

        let (dir,dw,de) = match c {
            White => (N,NW,NE),
            Black => (S,SW,SE),
        };

        let pushes = ps.shift(dir);
        let pushes = pushes & !(oc);

        let doubles = ps & BitBoard::mask_rank(if c == White { 1 } else { 6 });
        let doubles = doubles.shift_mult(dir, 2);
        let doubles = doubles & !(oc) & (!(oc)).shift(dir);

        // let b = doubles;
        // eprintln!("{:?}", b);

        doubles.iter_bitscan(|t| {
            let f = BitBoard::single(t.into()).shift_mult(!dir, 2);
            out.push(Move::PawnDouble { from: f.bitscan().into(), to: t.into() })
        });

        pushes.iter_bitscan(|t| {
            let t = t.into();
            if let Some(f) = (!dir).shift_coord(t) {
                out.push(Move::Quiet { from: f, to: t });
            }
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




