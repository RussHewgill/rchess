
use std::str::FromStr;

use crate::types::*;
use crate::tables::*;

impl Game {

    pub fn search_all(&self, ts: &Tables, c: Color) -> Vec<Move> {

        let mut out = vec![];

        out.extend(&self.search_king(&ts, c));

        out.extend(&self.search_sliding(Bishop, &ts, c));
        out.extend(&self.search_sliding(Rook, &ts, c));
        out.extend(&self.search_sliding(Queen, &ts, c));
        out.extend(&self.search_knights(&ts, c));
        out.extend(&self.search_pawns(&ts, c));

        let out = out.into_iter().filter(|m| {
            self.move_is_legal(&ts, m)
        }).collect();


        out
    }

    pub fn search_in_check(&self, ts: &Tables, col: Color) -> Vec<Move> {
        unimplemented!()
    }

    pub fn perft(&self, ts: &Tables, depth: u64, root: bool) -> (u64,u64) {
        let mut nodes = 0;
        let mut captures = 0;

        if depth == 0 { return (1,0); }

        let moves = self.search_all(&ts, self.state.side_to_move);

        // eprintln!("moves.len() = {:?}", moves.len());
        let mut k = 0;
        for m in moves.iter() {
            if let Some(g2) = self.make_move_unchecked(m) {
                let (ns,cs) = g2.perft(ts, depth - 1, false);
                match *m {
                    Move::Capture { .. } => captures += 1,
                    _                    => {},
                }
                if root {
                    eprintln!("{:>2}: {:?}: ({}, {})", k, m, ns, cs);
                }
                captures += cs;
                nodes += ns;
                k += 1;
            } else { panic!("move: {:?}\n{:?}", m, self); }
        }

        (nodes, captures)
    }

}

impl Game {

    pub fn move_is_legal(&self, ts: &Tables, m: &Move) -> bool {

        // TODO: En Passant Captures
        // TODO: Castling

        match self.get_at(m.from()) {
            Some((col,King)) => {
                !self.find_attacks_by_side(&ts, m.to(), !col)
            },
            Some((col,pc)) => {
                unimplemented!()
            },
            None => panic!(),
        }
    }

    pub fn find_check(&self, ts: &Tables, col: Color) -> bool {
        unimplemented!()
    }

    pub fn find_xray_rook(&self, ts: &Tables, p0: Coord, blocks: BitBoard, col: Color) -> BitBoard {
        let attacks = {
            let (_,_,c,d) = self._search_sliding_single(p0, Rook, blocks, &ts, col);
            c | d
        };
        let blocks2 = blocks & attacks;
        let attacks2 = {
            let (_,_,c,d) = self._search_sliding_single(p0, Rook, self.all_occupied() ^ blocks, &ts, col);
            c | d
        };
        let attacks3 = attacks ^ attacks2;
        attacks
    }

    pub fn find_xray_bishop(&self, ts: &Tables, p0: Coord, blocks: BitBoard, col: Color) -> BitBoard {

        let attacks = {
            let (_,_,c,d) = self._search_sliding_single(p0, Bishop, blocks, &ts, col);
            c | d
        };
        // eprintln!("attacks = {:?}", attacks);

        let blocks2 = blocks & attacks;
        let attacks2 = {
            let (_,_,c,d) = self._search_sliding_single(p0, Bishop, self.all_occupied() ^ blocks, &ts, col);
            c | d
        };
        // eprintln!("attacks2 = {:?}", attacks2);

        let attacks3 = attacks ^ attacks2;
        // eprintln!("attacks3 = {:?}", attacks3);

        attacks
        // unimplemented!()
    }

    pub fn find_pins_absolute(&self, ts: &Tables, col: Color) -> BitBoard {
        let king: Coord = self.get(King, col).bitscan().into();

        let mut pinned = BitBoard::empty();

        let mut pinner = self.find_xray_bishop(&ts, king, self.get_color(!col), col);
        // eprintln!("pinner = {:?}", pinner);
        pinner.iter_bitscan(|sq| {
            let ob = self.obstructed(&ts, sq.into(), king);
            if ob.bitscan_reset().0.0 == 0 {
                pinned |= ob & self.get_color(col);
            }
        });

        let mut pinner = self.find_xray_rook(&ts, king, self.get_color(!col), col);
        // eprintln!("pinner = {:?}", pinner);

        pinner.iter_bitscan(|sq| {
            let ob = self.obstructed(&ts, sq.into(), king);
            if ob.bitscan_reset().0.0 == 0 {
                pinned |= ob & self.get_color(col);
            }
        });

        pinned
        // unimplemented!()
    }

    pub fn obstructed(&self, ts: &Tables, c0: Coord, c1: Coord) -> BitBoard {

        let oc = self.all_occupied();
        let m = BitBoard::mask_between(&ts, c0, c1);

        oc & m
    }

    // pub fn mask_between(&self, ts: &Tables, c0: Coord, c1: Coord) -> BitBoard {
    // // pub fn obstructed(&self, ts: &Tables, c0: Coord, c1: Coord) -> BitBoard {

    //     let Coord(x0,y0) = c0;
    //     let Coord(x1,y1) = c1;

    //     if x0 == x1 {
    //         // File
    //         let (x0,x1) = (x0.min(x1),x0.max(x1));
    //         let (y0,y1) = (y0.min(y1),y0.max(y1));
    //         let b0 = BitBoard::single(Coord(x0,y0));
    //         let b1 = BitBoard::single(Coord(x1,y1));
    //         let b = BitBoard(2 * b1.0 - b0.0);
    //         let m = BitBoard::mask_file(x0.into());
    //         (b & m) & !(b0 | b1)
    //     } else if y0 == y1 {
    //         // Rank
    //         let (x0,x1) = (x0.min(x1),x0.max(x1));
    //         let (y0,y1) = (y0.min(y1),y0.max(y1));
    //         let b0 = BitBoard::single(Coord(x0,y0));
    //         let b1 = BitBoard::single(Coord(x1,y1));
    //         let b = BitBoard(2 * b1.0 - b0.0);
    //         let m = BitBoard::mask_rank(y0.into());
    //         (b & m) & !(b0 | b1)
    //     // } else if (x1 - x0) == (y1 - y0) {
    //     } else if (x1 as i64 - x0 as i64).abs() == (y1 as i64 - y0 as i64).abs() {
    //         // Diagonal
    //         let b0 = BitBoard::single(Coord(x0,y0));
    //         let b1 = BitBoard::single(Coord(x1,y1));
    //         // let b = BitBoard::new(&[Coord(x0,y0),Coord(x1,y1)])

    //         let (bb0,bb1) = (b0.0.min(b1.0),b0.0.max(b1.0));

    //         let b = BitBoard(2 * bb1 - bb0);
    //         // let b = BitBoard(2 * b0.0 - b1.0);
    //         // eprintln!("b = {:?}", b);
    //         // let m = BitBoard::mask_rank(y0.into());
    //         let m = ts.get_bishop(c0);

    //         let xx = x1 as i64 - x0 as i64;
    //         let yy = y1 as i64 - y0 as i64;

    //         let m = if xx.signum() == yy.signum() {
    //             m.ne | m.sw
    //         } else {
    //             m.nw | m.se
    //         };

    //         (b & m) & !(b0 | b1)
    //     } else {
    //         // println!("wat 2");
    //         // unimplemented!()
    //         BitBoard::empty()
    //     }
    // }

    pub fn find_attacks_by_side(&self, ts: &Tables, c0: Coord, col: Color) -> bool {

        let moves_k = ts.get_king(c0);
        if (*moves_k & self.get(King, col)).0 != 0 { return true; }

        let moves_p = ts.get_pawn(c0).get_capture(!col);
        if (*moves_p & self.get(Pawn, col)).0 != 0 { return true; }

        let bq = self.get(Queen, col);

        // let moves_b = ts.get_bishop(c0);
        let moves_b = self._search_sliding(Some(c0), Bishop, &ts, col);
        // if ((moves_b & self.get(Bishop, col)) | (moves_b & bq)).0 != 0 { return true; }

        false
        // unimplemented!()
    }

    pub fn find_attacks_to<'a>(&'a self, ts: &Tables, c0: Coord, col: Color)
                               -> impl Iterator<Item = Move> + 'a {
        let br = self.get(Rook, col);
        let bq = self.get(Queen, col);
        let bb = self.get(Bishop, col);
        let bn = self.get(Knight, col);
        let bp = self.get(Pawn, col);
        let bk = self.get(King, col);

        let attacks_r = self._search_sliding(Some(c0), Rook, &ts, !col)
            .into_iter().filter(move |m| match m {
                Move::Capture { from, to } => {
                    let t = BitBoard::single(*to);
                    ((br & t).0 != 0)
                        | ((bq & t).0 != 0)
                },
                _                    => false,
        });

        let attacks_b = self._search_sliding(Some(c0), Bishop, &ts, !col)
            .into_iter().filter(move |m| match m {
                Move::Capture { from, to } => {
                    let t = BitBoard::single(*to);
                    ((bb & t).0 != 0)
                        | ((bq & t).0 != 0)
                },
                _                    => false,
            });

        let attacks_n = self._search_knights(Some(c0), &ts, !col)
            .into_iter().filter(move |m| match m {
                Move::Capture { from, to } => {
                    let t = BitBoard::single(*to);
                    (bn & t).0 != 0
                },
                _                    => false,
            });

        let attacks_p = self._search_pawns(Some(c0), &ts, !col)
            .into_iter().filter(move |m| match m {
                Move::Capture { from, to } => {
                    let t = BitBoard::single(*to);
                    (bp & t).0 != 0
                },
                _                    => false,
            });

        let attacks_k = self._search_king_single(c0, &ts, !col, false)
        .into_iter().filter(move |m| match m {
            Move::Capture { from, to } => {
                let t = BitBoard::single(*to);
                (bk & t).0 != 0
            },
            _                    => false,
        });

        attacks_r
            .chain(attacks_b)
            .chain(attacks_n)
            .chain(attacks_p)
            .chain(attacks_k)
            .map(|m| m.reverse())
    }

    // pub fn find_threatened(&self, col: Color) -> BitBoard {
    //     unimplemented!()
    // }
}

impl Game {

    pub fn search_king(&self, ts: &Tables, col: Color) -> Vec<Move> {
        self._search_king(&ts, col, true)
    }

    pub fn _search_king_single(&self, c0: Coord, ts: &Tables, col: Color, forbid_check: bool) -> Vec<Move> {
        // let mut out = vec![];
        let occ = self.all_occupied();

        let moves = ts.get_king(c0);
        // let quiets   = b3 & !oc;
        // let captures = b3 & oc;

        eprintln!("moves = {:?}", moves);

        unimplemented!()
        // out
    }

    pub fn _search_king(&self, ts: &Tables, col: Color, forbid_check: bool) -> Vec<Move> {

        let p0 = self.get(King, col).bitscan();
        if p0 == 64 { return vec![]; }
        let moves = *ts.get_king(p0);

        let oc = self.all_occupied();
        let quiets   = moves & !oc;
        let captures = moves & self.get_color(!col);

        let mut out = vec![];

        quiets.iter_bitscan(|sq| {
            let go = if forbid_check {
                // let mut threats = self.find_attacks_to(&ts, sq.into(), !col);
                // threats.next().is_none()
                !self.find_attacks_by_side(&ts, sq.into(), !col)
            } else {
                true
            };
            if go {
                out.push(Move::Quiet { from: p0.into(), to: sq.into()});
            }
        });

        captures.iter_bitscan(|sq| {
            let go = if forbid_check {
                // let mut threats = self.find_attacks_to(&ts, sq.into(), !col);
                // threats.next().is_none()
                !self.find_attacks_by_side(&ts, sq.into(), !col)
            } else {
                true
            };
            if go {
                out.push(Move::Capture { from: p0.into(), to: sq.into()});
            }
        });

        // b3
        out
    }

    // pub fn search_knight(&self, )

    pub fn search_sliding(&self, pc: Piece, ts: &Tables, col: Color) -> Vec<Move> {
        // self._search_sliding(None, None, pc, &ts, col)
        self._search_sliding(None, pc, &ts, col)
    }

    pub fn _search_sliding(&self,
                       single: Option<Coord>,
                       // blocks: Option<BitBoard>,
                       pc: Piece,
                       ts: &Tables,
                       col: Color
    ) -> Vec<Move> {
        // let (quiets_pos, quiets_neg, captures_pos, captures_neg) =
        // let moves = self._search_sliding_2(single, blocks, pc, ts, col);
        let moves = self._search_sliding_2(single, None, pc, ts, col);
        let mut out = vec![];

        for (p0,(quiets_pos, quiets_neg, captures_pos, captures_neg)) in moves {

            let qs = quiets_pos | quiets_neg;
            let cs = captures_pos | captures_neg;

            cs.iter_bitscan(|sq| {
                out.push(Move::Capture { from: p0, to: sq.into() });
            });
            qs.iter_bitscan(|sq| {
                out.push(Move::Quiet { from: p0, to: sq.into() });
            });

        }


        out
    }

    pub fn _search_sliding_2(&self,
                         single: Option<Coord>,
                         blocks: Option<BitBoard>,
                         pc: Piece,
                         ts: &Tables,
                         col: Color,
    ) -> Vec<(Coord, (BitBoard,BitBoard,BitBoard,BitBoard))> {
        let mut out = vec![];
        // let occ = self.all_occupied();
        let occ = match blocks {
            Some(oc) => oc,
            None     => self.all_occupied(),
        };

        let mut pieces = match single {
            None     => self.get(pc, col),
            Some(c0) => BitBoard::single(c0),
        };

        pieces.iter_bitscan(|p0| {
            let (out_quiets_pos,out_quiets_neg,out_captures_pos,out_captures_neg) =
                self._search_sliding_single(p0.into(), pc, occ, &ts, col);
            out.push((p0.into(),(out_quiets_pos,out_quiets_neg,out_captures_pos,out_captures_neg)))
        });

        out
    }


    pub fn _search_sliding_single(&self,
                              p0:       Coord,
                              pc:       Piece,
                              blocks:   BitBoard,
                              ts:       &Tables,
                              col:      Color,
    ) -> (BitBoard,BitBoard,BitBoard,BitBoard) {

        let mut out_quiets_pos   = BitBoard::empty();
        let mut out_quiets_neg   = BitBoard::empty();
        let mut out_captures_pos = BitBoard::empty();
        let mut out_captures_neg = BitBoard::empty();

        let ms = match pc {
            Rook   => ts.get_rook(p0).to_vec(),
            Bishop => ts.get_bishop(p0).to_vec(),
            Queen  => {
                let mut m: Vec<(D, BitBoard)> = ts.get_bishop(p0).to_vec();
                m.append(&mut ts.get_rook(p0).to_vec());
                m
            },
            _      => panic!("search_sliding: wrong piece: {:?}", pc),
        };

        for (dir,moves) in ms {
            match dir {

                // Rook Positive
                N | E => {
                    let blocks = moves & blocks;
                    if blocks.0 != 0 {
                        let square = blocks.bitscan_isolate();
                        let sq: Coord = square.bitscan().into();
                        let nots = ts.get_rook(sq).get_dir(dir);
                        let mm = moves ^ *nots;
                        let mm = mm & !square;
                        if (square & self.get_color(!col)).0 != 0 {
                            // capture
                            // out.push(Move::Capture { from: p0.into(), to: sq });
                            let ss: Coord = sq.into();
                            // eprintln!("ss = {:?}", ss);
                            out_captures_pos.set_one_mut(sq.into());
                        };
                        // mm.iter_bitscan(|t| {
                        //     // out.push(Move::Quiet { from: p0.into(), to: t.into() });
                        //     out_quiets_pos.set_one(t.into());
                        // });
                        out_quiets_pos |= mm;
                    } else {
                        // moves.iter_bitscan(|t| {
                        //     // out.push(Move::Quiet { from: p0.into(), to: t.into() });
                        //     out_quiets_pos.set_one(t.into());
                        // });
                        out_quiets_pos |= moves;
                    }
                },

                // Rook Negative
                S | W => {
                    let blocks = moves & blocks;
                    if blocks.0 != 0 {
                        let square = blocks.bitscan_rev_isolate();
                        let sq: Coord = square.bitscan_rev().into();
                        let nots = ts.get_rook(sq).get_dir(dir);
                        let mm = moves ^ *nots;
                        let mm = mm & !square;
                        if (square & self.get_color(!col)).0 != 0 {
                            // capture
                            // out.push(Move::Capture { from: p0.into(), to: sq });
                            out_captures_neg.set_one_mut(sq.into());
                        }
                        // mm.iter_bitscan_rev(|t| {
                        //     // out.push(Move::Quiet { from: p0.into(), to: t.into() });
                        //     out_quiets_neg.set_one(t.into());
                        // });
                        out_quiets_neg |= mm;
                    } else {
                        // moves.iter_bitscan_rev(|t| {
                        //     // out.push(Move::Quiet { from: p0.into(), to: t.into() });
                        //     out_quiets_neg.set_one(t.into());
                        // });
                        out_quiets_neg |= moves;
                    }
                },

                // Bishop Positive
                NE | NW => {
                    let blocks = moves & blocks;
                    if blocks.0 != 0 {
                        let square = blocks.bitscan_isolate();
                        let sq: Coord = square.bitscan().into();
                        let nots = ts.get_bishop(sq).get_dir(dir);
                        let mm = moves ^ *nots;
                        let mm = mm & !square;
                        if (square & self.get_color(!col)).0 != 0 {
                            // capture
                            // out.push(Move::Capture { from: p0.into(), to: sq });
                            out_captures_pos.set_one_mut(sq.into());
                        }
                        // mm.iter_bitscan(|t| {
                        //     // out.push(Move::Quiet { from: p0.into(), to: t.into() });
                        //     out_quiets_pos.set_one(t.into());
                        // });
                        out_quiets_pos |= mm;
                    } else {
                        // moves.iter_bitscan(|t| {
                        //     // out.push(Move::Quiet { from: p0.into(), to: t.into() });
                        //     out_quiets_pos.set_one(t.into());
                        // });
                        out_quiets_pos |= moves;
                    }
                },

                // Bishop Negative
                SE | SW => {
                    let blocks = moves & blocks;
                    if blocks.0 != 0 {
                        let square = blocks.bitscan_rev_isolate();
                        let sq: Coord = square.bitscan_rev().into();
                        let nots = ts.get_bishop(sq).get_dir(dir);
                        let mm = moves ^ *nots;
                        let mm = mm & !square;
                        if (square & self.get_color(!col)).0 != 0 {
                            // capture
                            // out.push(Move::Capture { from: p0.into(), to: sq });
                            out_captures_neg.set_one_mut(sq.into());
                        }
                        // mm.iter_bitscan_rev(|t| {
                        //     // out.push(Move::Quiet { from: p0.into(), to: t.into() });
                        //     out_quiets_neg.set_one(t.into());
                        // });
                        out_quiets_neg |= mm;
                    } else {
                        // moves.iter_bitscan_rev(|t| {
                        //     // out.push(Move::Quiet { from: p0.into(), to: t.into() });
                        //     out_quiets_neg.set_one(t.into());
                        // });
                        out_quiets_neg |= moves;
                    }
                },
            }
        }

        // out.push((p0.into(),(out_quiets_pos,out_quiets_neg,out_captures_pos,out_captures_neg)))
        (out_quiets_pos, out_quiets_neg, out_captures_pos, out_captures_neg)
    }

    pub fn search_knights(&self, ts: &Tables, col: Color) -> Vec<Move> {
        self._search_knights(None, ts, col)
    }

    pub fn _search_knights(&self, single: Option<Coord>, ts: &Tables, col: Color) -> Vec<Move> {
        let mut out = vec![];
        let oc = self.all_occupied();

        let ks = match single {
            Some(c0) => BitBoard::single(c0),
            None     => self.get(Knight, col),
        };

        ks.iter_bitscan(|sq| {
            let ms = ts.get_knight(sq);

            let quiets   = *ms & !oc;
            let captures = *ms & self.get_color(!col);

            quiets.iter_bitscan(|t| {
                out.push(Move::Quiet { from: sq.into(), to: t.into()});
            });

            captures.iter_bitscan(|t| {
                out.push(Move::Capture { from: sq.into(), to: t.into()});
            });

        });

        out
    }

    pub fn search_pawns(&self, ts: &Tables, col: Color) -> Vec<Move> {
        self._search_pawns(None, &ts, col)
    }

    pub fn _search_pawns(&self, single: Option<Coord>, ts: &Tables, col: Color) -> Vec<Move> {
        let mut out = vec![];
        let oc = self.all_occupied();

        let ps = match single {
            Some(c0) => BitBoard::single(c0),
            None     => self.get(Pawn, col),
        };

        let (dir,dw,de) = match col {
            White => (N,NW,NE),
            Black => (S,SW,SE),
        };

        let pushes = ps.shift(dir);
        let pushes = pushes & !(oc);

        let doubles = ps & BitBoard::mask_rank(if col == White { 1 } else { 6 });
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
        // let captures = captures & self.get_color(!col);

        // eprintln!("{:?}", ps);

        ps.iter_bitscan(|p0| {
            let f  = BitBoard::index_bit(p0);
            let bb = BitBoard::empty().flip(f);
            let mut cs = (bb.shift(dw) & self.get_color(!col))
                | (bb.shift(de) & self.get_color(!col));
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

    /*
    fn search_rooks2(&self, ts: &Tables, c: Color) -> Vec<Move> {
        let mut rooks = self.get(Rook, c);
        let mut out = vec![];
        let occ = self.all_occupied();

        rooks.iter_bitscan(|p0| {
            // let ms: &MoveSetRook = ts.rook_moves.get(&p0.into()).unwrap();
            let ms: &MoveSetRook = ts.get_rook(p0);

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
    fn search_bishops2(&self, ts: &Tables, c: Color) -> Vec<Move> {
        let bishops = self.get(Bishop, c);
        let mut out = vec![];
        let occ = self.all_occupied();

        bishops.iter_bitscan(|p0| {
            let ms: &MoveSetBishop = ts.get_bishop(p0);

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
    */

}




