
use std::str::FromStr;

use crate::types::*;
use crate::tables::*;

use rayon::prelude::*;

impl Game {

    // XXX: slower than could be
    pub fn search_all_single(&self, ts: &Tables, c0: Coord, col: Color) -> Outcome {
        // let mut out = vec![];
        // match self.get_at(c0) {
        //     Some((col1,Pawn))                  => {

        //     },
        //     Some((col1,King))                  => {},
        //     Some((col1,Knight))                => {},
        //     Some((col1,Bishop | Rook | Queen)) => {},
        //     None                               => {},
        // }
        // out

        let ms = self.search_all(&ts, col);
        match ms {
            Outcome::Checkmate(win) => Outcome::Checkmate(win),
            Outcome::Stalemate      => Outcome::Stalemate,
            Outcome::Moves(ms)      => {
                let out: Vec<Move> = ms.into_iter()
                    .filter(|m| m.sq_from() == c0)
                    .collect();
                Outcome::Moves(out)
            },
        }
    }

    // pub fn search_all(&self, ts: &Tables, col: Color) -> Option<Vec<Move>> {
    pub fn search_all(&self, ts: &Tables, col: Color) -> Outcome {
        match self.state.checkers {
            Some(cs) if !cs.is_empty() => {
                // if let Some(win) = self.is_checkmate(&ts) {
                //     return Outcome::Checkmate(win);
                // }
                // println!("wat check");
                self._search_in_check(&ts, col)
            },
            _                          => {
                // println!("wat all");
                self._search_all(&ts, col)
            },
        }
    }

    // pub fn _search_all_test(&self, ts: &Tables, col: Color, test: bool) -> Outcome {

    //     // let out = if test {
    //     //     let mut k = self.search_king(&ts, col);
    //     //     k.extend(self.search_sliding_iter(&ts, Bishop, col));
    //     //     k.extend(self.search_sliding_iter(&ts, Rook, col));
    //     //     k.extend(self.search_sliding_iter(&ts, Queen, col));
    //     //     k
    //     // } else {
    //     //     let k = self.search_king(&ts, col);
    //     //     let b = self.search_sliding(&ts, Bishop, col);
    //     //     let r = self.search_sliding(&ts, Rook, col);
    //     //     let q = self.search_sliding(&ts, Queen, col);
    //     //     vec![k,b,r,q].concat()
    //     // };

    //     // // Worse
    //     // let mut k = self.search_king(&ts, col);
    //     // let b = self.search_sliding(&ts, Bishop, col);
    //     // k.extend(b.into_iter().filter(|m| self.move_is_legal(&ts, m)));
    //     // let r = self.search_sliding(&ts, Rook, col);
    //     // k.extend(r.into_iter().filter(|m| self.move_is_legal(&ts, m)));
    //     // let q = self.search_sliding(&ts, Queen, col);
    //     // k.extend(q.into_iter().filter(|m| self.move_is_legal(&ts, m)));
    //     // let n = self.search_knights(&ts, col);
    //     // k.extend(n.into_iter().filter(|m| self.move_is_legal(&ts, m)));
    //     // let p = self.search_pawns(&ts, col);
    //     // k.extend(p.into_iter().filter(|m| self.move_is_legal(&ts, m)));
    //     // let pp = self._search_promotions(&ts, None, col);
    //     // k.extend(pp.into_iter().filter(|m| self.move_is_legal(&ts, m)));
    //     // let cs = self._search_castles(&ts);
    //     // k.extend(cs.into_iter().filter(|m| self.move_is_legal(&ts, m)));

    //     let out: Vec<Move> = out.into_iter().filter(|m| {
    //         // let out: Vec<Move> = k.into_iter().filter(|m| {
    //         self.move_is_legal(&ts, m)
    //     }).collect();

    //     if out.is_empty() {
    //         Outcome::Stalemate
    //     } else {
    //         Outcome::Moves(out)
    //     }
    // }

    pub fn _search_all(&self, ts: &Tables, col: Color) -> Outcome {

        let k = self.search_king(&ts, col);
        let b = self.search_sliding(&ts, Bishop, col);
        let r = self.search_sliding(&ts, Rook, col);
        let q = self.search_sliding(&ts, Queen, col);
        let n = self.search_knights(&ts, col);
        let p = self.search_pawns(&ts, col);
        let pp = self._search_promotions(&ts, None, col);
        let cs = self._search_castles(&ts);
        let out = vec![k,b,r,q,n,p,pp,cs].concat();

        // k.extend(iter)

        // let out: Vec<Move> = out.into_par_iter().filter(|m| {
        let out: Vec<Move> = out.into_iter().filter(|m| {
            self.move_is_legal(&ts, m)
        }).collect();

        if out.is_empty() {
            Outcome::Stalemate
        } else {
            Outcome::Moves(out)
        }
    }

    pub fn _search_in_check(&self, ts: &Tables, col: Color) -> Outcome {
        let mut out = vec![];

        out.extend(&self.search_king(&ts, col));

        let mut x = 0;
        self.state.checkers.unwrap().iter_bitscan(|sq| x += 1);
        if x == 1 {
            out.extend(&self.search_sliding(&ts, Bishop, col));
            out.extend(&self.search_sliding(&ts, Rook, col));
            out.extend(&self.search_sliding(&ts, Queen, col));
            out.extend(&self.search_knights(&ts, col));
            out.extend(&self.search_pawns(&ts, col));
            out.extend(&self._search_promotions(&ts, None, col));
        }

        let out: Vec<Move> = out.into_iter().filter(|m| {
            self.move_is_legal(&ts, m)
        }).collect();

        if out.is_empty() {
            Outcome::Checkmate(!self.state.side_to_move)
        } else {
            Outcome::Moves(out)
        }
    }

    pub fn perft(&self, ts: &Tables, depth: u64) -> (u64,Vec<(Move,u64)>) {
        // let mut nodes = 0;
        // let mut captures = 0;

        if depth == 0 { return (1,vec![]); }

        let moves = self.search_all(&ts, self.state.side_to_move);
        if moves.is_end() { return (0,vec![]); }
        let moves = moves.get_moves_unsafe();

        // eprintln!("moves.len() = {:?}", moves.len());
        // let mut k = 0;
        // let mut out = vec![];

        // let out = moves.into_iter().flat_map(|m| {
        let out = moves.into_par_iter().flat_map(|m| {
            if let Ok(g2) = self.make_move_unchecked(&ts, &m) {
                let (ns,cs) = g2._perft(ts, depth - 1, false);
                // match m {
                //     Move::Capture { .. } => captures += 1,
                //     _                    => {},
                // }
                // if root {
                //     eprintln!("{:>2}: {:?}: ({}, {})", k, m, ns, cs);
                // }
                // captures += cs;
                // nodes += ns;
                // out.push((m, ns));
                // k += 1;
                Some((m,ns))
            } else {
                // panic!("move: {:?}\n{:?}", m, self);
                None
            }
        });

        let out: Vec<(Move,u64)> = out.collect();
        let nodes = out.clone().into_iter().map(|x| x.1).sum();

        // for m in moves.into_iter() {
        //     if let Ok(g2) = self.make_move_unchecked(&ts, &m) {
        //         let (ns,cs) = g2._perft(ts, depth - 1, false);
        //         match m {
        //             Move::Capture { .. } => captures += 1,
        //             _                    => {},
        //         }
        //         // if root {
        //         //     eprintln!("{:>2}: {:?}: ({}, {})", k, m, ns, cs);
        //         // }
        //         captures += cs;
        //         nodes += ns;
        //         out.push((m, ns));
        //         k += 1;
        //     } else {
        //         // panic!("move: {:?}\n{:?}", m, self);
        //     }
        // }

        (nodes, out)
    }

    pub fn _perft(&self, ts: &Tables, depth: u64, root: bool) -> (u64,u64) {
        let mut nodes = 0;
        let mut captures = 0;

        if depth == 0 { return (1,0); }

        let moves = self.search_all(&ts, self.state.side_to_move);
        if moves.is_end() { return (0,0); }

        // eprintln!("moves.len() = {:?}", moves.len());
        let mut k = 0;
        for m in moves.into_iter() {
            if let Ok(g2) = self.make_move_unchecked(&ts, &m) {
                let (ns,cs) = g2._perft(ts, depth - 1, false);
                match m {
                    Move::Capture { .. } => captures += 1,
                    _                    => {},
                }
                if root {
                    eprintln!("{:>2}: {:?}: ({}, {})", k, m, ns, cs);
                }
                captures += cs;
                nodes += ns;
                k += 1;
            } else {
                // panic!("move: {:?}\n{:?}", m, self);
            }
        }

        (nodes, captures)
    }

}

impl Game {

    pub fn move_is_legal(&self, ts: &Tables, m: &Move) -> bool {

        // TODO: En Passant Captures
        // TODO: Castling

        if m.filter_en_passant() {
            if self.state.en_passant.is_none() {
                return false;
            } else if let Some(g2) = self.clone()._make_move_unchecked(&ts, &m) {
            // } else if let Ok(g2) = self.clone().make_move_unchecked(&ts, &m) {

                let checks = g2.find_checkers(&ts, self.state.side_to_move);
                return checks.is_empty();

                // let b = g2.state.checkers.unwrap();
                // return b.is_empty();

                // unimplemented!()
            } else {
                return false;
            }
        }

        match self.get_at(m.sq_from()) {
            Some((col,King)) => {
                !self.find_attacks_by_side(&ts, m.sq_to(), !col, true)
            },
            Some((col,pc)) => {
                let pins = self.get_pins(col);

                // let k = BitBoard::single(m.sq_from()) & pins;
                // eprintln!("k = {:?}", k);

                let x =
                    // Not pinned
                    ((BitBoard::single(m.sq_from()) & pins).0 == 0)
                    // OR moving along pin ray
                    | (ts.aligned(m.sq_from(), m.sq_to(), self.get(King, col).bitscan().into()).0 != 0);

                // not in check
                let x0 = x & self.find_checkers(&ts, col).is_empty();

                // OR capturing checking piece
                let x1 = m.sq_to() == self.state.checkers.unwrap().bitscan().into();

                // OR (Not pinned & Blocking check)
                let x2 = {
                    (BitBoard::single(m.sq_to()) & self.state.check_block_mask.unwrap()).is_not_empty()
                };

                // eprintln!("x = {:?}", x);
                // eprintln!("x0 = {:?}", x0);
                // eprintln!("x1 = {:?}", x1);

                x0 | (x & x1) | (x & x2)
                // unimplemented!()
                // (x & self.find_checkers(&ts, col).is_empty())
                //     | (m.filter_all_captures() & (m.to() == ))
                // unimplemented!()
            },
            None => panic!(),
        }
    }

    pub fn find_checkers(&self, ts: &Tables, col: Color) -> BitBoard {
        let p0: Coord = self.get(King, col).bitscan().into();

        let moves = self.find_attackers_to(&ts, p0, col);
        let moves = moves & self.get_color(!col);
        moves
    }

    pub fn find_slider_blockers(&self, ts: &Tables, c0: Coord, col: Color) -> (BitBoard, BitBoard) {
        let mut blockers = BitBoard::empty();
        let mut pinners = BitBoard::empty();

        let snipers = ts.get_rook(c0).concat() & (self.get_piece(Rook) | self.get_piece(Queen))
            | ts.get_bishop(c0).concat() & (self.get_piece(Bishop) | self.get_piece(Queen));

        let snipers = snipers & self.get_color(!col);

        // let mut snipers = ts.get_rook(c0).concat() & (self.get_piece(Rook) | self.get_piece(Queen));
        // eprintln!("snipers = {:?}", snipers);

        // let mut snipers = {
        //     let (a,b,c,d) = self._search_sliding_single(c0, Rook, blocks, &ts, col);
        //     let (e,f,g,h) = self._search_sliding_single(c0, Bishop, blocks, &ts, col);
        //     ((a | b | c | d) & (self.get_piece(Rook) | self.get_piece(Queen)))
        //         | ((e | f | g | h) & (self.get_piece(Bishop) | self.get_piece(Queen)))
        // };

        let occ = self.all_occupied() ^ snipers;
        // eprintln!("occ = {:?}", occ);

        // let (col0, _) = self.get_at(c0).unwrap();

        snipers.iter_bitscan(|sq| {
            let b = ts.between(c0, sq.into()) & occ;
            // eprintln!("b = {:?}", b);
            // let cc: Coord = sq.into();
            // eprintln!("cc = {:?}", cc);

            if b.is_not_empty() & !b.more_than_one() {
                // eprintln!("b = {:?}", b);
                blockers |= b;
                if let Some((col1,_)) = self.get_at(sq.into()) {
                    // println!("wat 1");
                    if col != col1 {
                        pinners.set_one_mut(sq.into());
                    }
                }
            }


            // // if (b.0 != 0) & !((b & BitBoard(b.0 - 1)).0 != 0) {
            // if (b.0 != 0) & !((b & BitBoard(b.0.overflowing_sub(1).0)).0 != 0) {
            //     // println!("wat 0");
            //     blockers |= b;
            //     if let Some((col1,_)) = self.get_at(sq.into()) {
            //         // println!("wat 1");
            //         pinners.set_one_mut(sq.into());
            //     }
            // }

        });
        (blockers, pinners)
    }

    pub fn obstructed(&self, ts: &Tables, c0: Coord, c1: Coord) -> BitBoard {

        let oc = self.all_occupied();
        let m = BitBoard::mask_between(&ts, c0, c1);

        oc & m
    }

    pub fn find_attacks_by_side(&self, ts: &Tables, c0: Coord, col: Color, king: bool) -> bool {

        let moves_k = ts.get_king(c0);
        if (*moves_k & self.get(King, col)).is_not_empty() { return true; }

        let moves_p = ts.get_pawn(c0).get_capture(!col);
        if (*moves_p & self.get(Pawn, col)).is_not_empty() { return true; }

        let bq = self.get(Queen, col);

        // let moves_b = ts.get_bishop(c0);
        // let moves_b = self._search_sliding(Some(c0), Bishop, &ts, col);
        // let moves_r = self._search_sliding(Some(c0), Rook, &ts, col);

        let moves_n = ts.get_knight(c0);
        // eprintln!("moves_n = {:?}", moves_n);
        if (*moves_n & self.get(Knight, col)).is_not_empty() { return true; }

        let occ = if king {
            self.all_occupied() & !self.get(King, !col)
        } else {
            self.all_occupied()
        };

        // let moves_r = {
        //     let (a,b,c,d) = self._search_sliding_single(&ts, c0, Rook, occ, &ts, !col);
        //     a | b | c | d
        // };
        let moves_r = self._search_sliding_single(&ts, Rook, c0, !col, Some(occ));
        if ((moves_r & self.get(Rook, col)).is_not_empty())
            | ((moves_r & self.get(Queen, col)).is_not_empty()) { return true; }

        // let moves_b = {
        //     let (a,b,c,d) = self._search_sliding_single(&ts, c0, Bishop, occ, &ts, !col);
        //     a | b | c | d
        // };
        let moves_b = self._search_sliding_single(&ts, Bishop, c0, !col, Some(occ));
        if ((moves_b & self.get(Bishop, col)).is_not_empty())
            | ((moves_b & self.get(Queen, col)).is_not_empty()) { return true; }

        false
        // unimplemented!()
    }

    pub fn find_attackers_to(&self, ts: &Tables, c0: Coord, col: Color) -> BitBoard {

        let pawns = ts.get_pawn(c0);
        // let pawns = *pawns.get_capture(White) | *pawns.get_capture(Black);
        let pawns = *pawns.get_capture(col);
        let pawns = pawns & self.get_piece(Pawn);

        let knights = *ts.get_knight(c0) & self.get_piece(Knight);

        // let moves_r = {
        //     let (a,b,c,d) = self._search_sliding_single(&ts, c0, Rook, self.all_occupied(), White);
        //     let (e,f,g,h) = self._search_sliding_single(&ts, c0, Rook, self.all_occupied(), Black);
        //     a | b | c | d | e | f | g | h
        // };
        let moves_r = self._search_sliding_single(&ts, Rook, c0, col, None);
            // | self._search_sliding_single(&ts, Rook, c0, !col, Some(occ));
        let rooks = moves_r & (self.get_piece(Rook) | self.get_piece(Queen));

        // let moves_b = {
        //     let (a,b,c,d) = self._search_sliding_single(&ts, c0, Bishop, self.all_occupied(), White);
        //     let (e,f,g,h) = self._search_sliding_single(&ts, c0, Bishop, self.all_occupied(), Black);
        //     a | b | c | d | e | f | g | h
        // };
        let moves_b = self._search_sliding_single(&ts, Bishop, c0, col, None);
            // | self._search_sliding_single(&ts, Bishop, c0, !col, Some(occ));
        let bishops = moves_b & (self.get_piece(Bishop) | self.get_piece(Queen));

        // let king = self._search_king_single(p0, &ts, col, false);
        let king = self._search_king_attacks(&ts, c0, col);
        let king = king & self.get_piece(King);

        pawns
            | knights
            | rooks
            | bishops
            | king
    }

}

/// Sliding
impl Game {

    pub fn search_sliding_iter<'a>(
        &'a self,
        ts:           &'a Tables,
        pc:           Piece,
        col:          Color,
    ) -> impl Iterator<Item = Move> + 'a {
        let pieces = self.get(pc, col);

        let moves = pieces.into_iter().flat_map(move |sq| {
            let ms = self._search_sliding_single(&ts, pc, sq.into(), col, None);
            let sq2: Coord = sq.into();
            // (sq,ms)
            ms.into_iter().map(move |to| {
                Move::Quiet { from: sq2, to: to.into() }
            })
        });

        // let moves = moves.flat_map(|(sq,ms)| {
        //     // let out = ms.into_iter().map(|to| {
        //     //     Move::Quiet { from: sq.into(), to: to.into() }
        //     // });
        //     // out
        //     let sq2: Coord = sq.into();
        //     ms.into_iter().map(move |to| {
        //         Move::Quiet { from: sq2, to: to.into() }
        //     })
        // });

        // let moves = pieces.into_iter().map(|sq| {
        //     let moves   = self._search_sliding_single(&ts, pc, sq.into(), col, None);
        //     // let attacks = moves & self.get_color(!col);
        //     // let quiets  = moves & self.all_empty();
        //     // attacks.iter_bitscan(|sq2| {
        //     //     out.push(Move::Capture { from: sq.into(), to: sq2.into() });
        //     // });
        //     // quiets.iter_bitscan(|sq2| {
        //     //     out.push(Move::Quiet { from: sq.into(), to: sq2.into() });
        //     // });
        //     // let out = moves.into_iter().map(|to| {
        //     //     Move::Quiet { from: sq.into(), to: to.into() }
        //     // });
        //     // out
        //     moves
        // });

        // let moves = moves.map(|bb| {
        //     let sq = bb.bitscan();
        //     Move::Quiet { from: sq.into(), to: sq.into() }
        // });
        // let moves = pieces.into_iter().map(|sq| {
        //     Move::Quiet { from: sq.into(), to: sq.into() }
        // });

            // .flatten()
        // unimplemented!()
        moves
    }

    pub fn search_sliding(&self, ts: &Tables, pc: Piece, col: Color) -> Vec<Move> {
        let mut out = vec![];
        let pieces = self.get(pc, col);
        pieces.iter_bitscan(|sq| {
            let moves   = self._search_sliding_single(&ts, pc, sq.into(), col, None);
            let attacks = moves & self.get_color(!col);
            let quiets  = moves & self.all_empty();
            attacks.iter_bitscan(|sq2| {
                out.push(Move::Capture { from: sq.into(), to: sq2.into() });
            });
            quiets.iter_bitscan(|sq2| {
                out.push(Move::Quiet { from: sq.into(), to: sq2.into() });
            });
        });
        out
    }

    pub fn _search_sliding_single(&self,
                                  ts: &Tables,
                                  pc: Piece,
                                  c0: Coord,
                                  col: Color,
                                  occ: Option<BitBoard>
    ) -> BitBoard {
        let occ = match occ {
            None    => self.all_occupied(),
            Some(b) => b,
        };
        let moves = match pc {
            Rook   => ts.attacks_rook(c0, occ),
            Bishop => ts.attacks_bishop(c0, occ),
            Queen  => ts.attacks_bishop(c0, occ) | ts.attacks_rook(c0, occ),
            _      => panic!("search sliding: {:?}", pc),
        };
        moves & !self.get_color(col)
    }

}

/// Other pieces
impl Game {

    pub fn _search_castles(&self, ts: &Tables) -> Vec<Move> {
        let mut out = vec![];
        let col = self.state.side_to_move;
        let (kingside,queenside) = self.state.castling.get_color(col);

        if self.state.checkers.unwrap().is_not_empty() { return out; }

        let king: Coord = self.get(King, col).bitscan().into();

        if kingside {
            // let rook: Coord = if col == White { "H1".into() } else { "H8".into() };
            let rook: Coord = if col == White { Coord(7,0) } else { Coord(7,7) };
            if let Some((_,Rook)) = self.get_at(rook) {
                let between = ts.between_exclusive(king, rook);

                if (between & self.all_occupied()).is_empty() {
                    let mut go = true;
                    for sq in between.into_iter() {
                        if self.find_attacks_by_side(&ts, sq.into(), !col, true) {
                            go = false;
                            break;
                        }
                    }
                    if go {
                        // let to = if col == White { "G1".into() } else { "G8".into() };
                        // let rook_to = if col == White { "F1".into() } else { "F8".into() };
                        let to = if col == White { Coord(6,0) } else { Coord(6,7) };
                        let rook_to = if col == White { Coord(5,0) } else { Coord(5,7) };
                        out.push(Move::Castle { from: king, to, rook_from: rook, rook_to });
                    }
                }
            }

        }
        if queenside {
            // let rook: Coord = if col == White { "A1".into() } else { "A8".into() };
            let rook: Coord = if col == White { Coord(0,0) } else { Coord(0,7) };
            if let Some((_,Rook)) = self.get_at(rook) {
                let between = ts.between_exclusive(king, rook);

                if (between & self.all_occupied()).is_empty() {
                    let mut go = true;
                    let king_moves = if col == White { BitBoard(0xc) } else { BitBoard(0xc00000000000000) };
                    for sq in king_moves.into_iter() {
                        // let cc: Coord = sq.into();
                        // eprintln!("cc = {:?}", cc);

                        if self.find_attacks_by_side(&ts, sq.into(), !col, true) {
                            go = false;
                            break;
                        }
                    }
                    if go {
                        // let to      = if col == White { "C1".into() } else { "C8".into() };
                        // let rook_to = if col == White { "D1".into() } else { "D8".into() };
                        let to      = if col == White { Coord(2,0) } else { Coord(2,7) };
                        let rook_to = if col == White { Coord(3,0) } else { Coord(3,7) };
                        out.push(Move::Castle { from: king, to, rook_from: rook, rook_to });
                    }
                }
            }
        }

        out
    }

    pub fn _search_king_attacks(&self, ts: &Tables, c0: Coord, col: Color) -> BitBoard {
        let c0 = self.get(King, col).bitscan();
        let mut out = BitBoard::empty();

        let moves = *ts.get_king(c0);
        let moves = moves & !self.get_color(col);
        moves

        // let moves = *ts.get_king(c0);
        // if let Some((col,_)) = self.get_at(c0.into()) {
        //     let captures = moves & self.get_color(!col);
        //     out |= captures;
        // }

        // kings.iter_bitscan(|sq| {
        //     let moves = *ts.get_king(sq);
        //     if let Some((col,_)) = self.get_at(sq.into()) {
        //         let captures = moves & self.get_color(!col);
        //         out |= captures;
        //     }
        // });

        // out
    }

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
                !self.find_attacks_by_side(&ts, sq.into(), !col, false)
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
                !self.find_attacks_by_side(&ts, sq.into(), !col, false)
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
        self._search_pawns(&ts, None, col)
    }

    pub fn _search_pawns(&self, ts: &Tables, single: Option<Coord>, col: Color) -> Vec<Move> {
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
        let ps = ps & !(if col == White { BitBoard::mask_rank(6) } else { BitBoard::mask_rank(1) });

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

        if let Some(ep) = self.state.en_passant {
            let attacks = ts.get_pawn(ep).get_capture(!col);
            let attacks = *attacks & ps;

            attacks.iter_bitscan(|sq| {
                let capture = if col == White { S.shift_coord(ep) } else { N.shift_coord(ep) };
                let capture = capture
                    .expect(&format!("en passant bug? ep: {:?}, capture: {:?}", ep, capture));
                out.push(Move::EnPassant { from: sq.into(), to: ep, capture });
            });

        }

        // pushes.serialize()
        // unimplemented!()
        // vec![]
        out
    }

    pub fn _search_promotions(&self, ts: &Tables, single: Option<Coord>, col: Color) -> Vec<Move> {
        let mut out = vec![];
        let oc = self.all_occupied();

        let (dir,dw,de) = match col {
            White => (N,NW,NE),
            Black => (S,SW,SE),
        };

        let ps = match single {
            Some(c0) => BitBoard::single(c0),
            None     => self.get(Pawn, col),
        };
        let ps = ps & if col == White { BitBoard::mask_rank(6) } else { BitBoard::mask_rank(1) };

        let pushes = ps.shift(dir);
        let pushes = pushes & !(oc);

        pushes.iter_bitscan(|t| {
            let t = t.into();
            if let Some(f) = (!dir).shift_coord(t) {
                // out.push(Move::Quiet { from: f, to: t });
                out.push(Move::Promotion { from: f, to: t, new_piece: Queen });
                out.push(Move::Promotion { from: f, to: t, new_piece: Knight });
                out.push(Move::Promotion { from: f, to: t, new_piece: Rook });
                out.push(Move::Promotion { from: f, to: t, new_piece: Bishop });
            }
        });

        ps.iter_bitscan(|p0| {
            let f  = BitBoard::index_bit(p0);
            let bb = BitBoard::empty().flip(f);
            let mut cs = (bb.shift(dw) & self.get_color(!col))
                | (bb.shift(de) & self.get_color(!col));
            while cs.0 != 0 {
                let t = cs.bitscan_reset_mut().into();
                out.push(Move::PromotionCapture { from: f, to: t, new_piece: Queen });
                out.push(Move::PromotionCapture { from: f, to: t, new_piece: Knight });
                out.push(Move::PromotionCapture { from: f, to: t, new_piece: Rook });
                out.push(Move::PromotionCapture { from: f, to: t, new_piece: Bishop });
            }
        });

        out
    }

}

// /// previous sliding
// impl Game {

//     pub fn search_sliding(&self, pc: Piece, ts: &Tables, col: Color) -> Vec<Move> {
//         // self._search_sliding(None, None, pc, &ts, col)
//         self._search_sliding(None, pc, &ts, col)
//     }

//     pub fn _search_sliding(&self,
//                        single: Option<Coord>,
//                        // blocks: Option<BitBoard>,
//                        pc: Piece,
//                        ts: &Tables,
//                        col: Color
//     ) -> Vec<Move> {
//         // let (quiets_pos, quiets_neg, captures_pos, captures_neg) =
//         // let moves = self._search_sliding_2(single, blocks, pc, ts, col);
//         let moves = self._search_sliding_2(single, None, pc, ts, col);
//         let mut out = vec![];

//         for (p0,(quiets_pos, quiets_neg, captures_pos, captures_neg)) in moves {

//             let qs = quiets_pos | quiets_neg;
//             let cs = captures_pos | captures_neg;

//             cs.iter_bitscan(|sq| {
//                 out.push(Move::Capture { from: p0, to: sq.into() });
//             });
//             qs.iter_bitscan(|sq| {
//                 out.push(Move::Quiet { from: p0, to: sq.into() });
//             });

//         }


//         out
//     }

//     pub fn _search_sliding_2(&self,
//                          single: Option<Coord>,
//                          blocks: Option<BitBoard>,
//                          pc: Piece,
//                          ts: &Tables,
//                          col: Color,
//     ) -> Vec<(Coord, (BitBoard,BitBoard,BitBoard,BitBoard))> {
//         let mut out = vec![];
//         // let occ = self.all_occupied();
//         let occ = match blocks {
//             Some(oc) => oc,
//             None     => self.all_occupied(),
//         };

//         let mut pieces = match single {
//             None     => self.get(pc, col),
//             Some(c0) => BitBoard::single(c0),
//         };

//         pieces.iter_bitscan(|p0| {
//             let (out_quiets_pos,out_quiets_neg,out_captures_pos,out_captures_neg) =
//                 self._search_sliding_single(p0.into(), pc, occ, &ts, col);
//             out.push((p0.into(),(out_quiets_pos,out_quiets_neg,out_captures_pos,out_captures_neg)))
//         });

//         out
//     }

//     pub fn _search_sliding_single(&self,
//                                   p0:       Coord,
//                                   pc:       Piece,
//                                   blocks:   BitBoard,
//                                   ts:       &Tables,
//                                   col:      Color,
//     ) -> (BitBoard,BitBoard,BitBoard,BitBoard) {

//         let mut out_quiets_pos   = BitBoard::empty();
//         let mut out_quiets_neg   = BitBoard::empty();
//         let mut out_captures_pos = BitBoard::empty();
//         let mut out_captures_neg = BitBoard::empty();

//         let rooks = ts.get_rook(p0).to_vec();
//         let bishops = ts.get_bishop(p0).to_vec();

//         let both = [rooks[0],rooks[1],rooks[2],rooks[3],
//                     bishops[0],bishops[1],bishops[2],bishops[3]];

//         let ms = match pc {
//             Rook   => rooks.iter(),
//             Bishop => bishops.iter(),
//             Queen  => {
//                 // let mut m0 = ts.get_bishop(p0).to_vec().iter();
//                 // let m1 = ts.get_rook(p0).to_vec().iter();
//                 // m0.chain(m1).into()
//                 // unimplemented!()
//                 // ts.get_bishop(p0).to_vec_with_rook(ts.get_rook(p0).to_vec()).iter()

//                 both.iter()
//                 // unimplemented!()

//                 // let mut m: Vec<(D, BitBoard)> = ts.get_bishop(p0).to_vec();
//                 // m.append(&mut ts.get_rook(p0).to_vec());
//                 // m
//                 // unimplemented!()
//             },
//             _      => panic!("search_sliding: wrong piece: {:?}", pc),
//         };

//         for (dir,moves) in ms {
//             let (dir,moves) = (*dir,*moves);
//             match dir {

//                 // Rook Positive
//                 N | E => {
//                     let blocks = moves & blocks;
//                     if blocks.0 != 0 {
//                         let square = blocks.bitscan_isolate();
//                         let sq: Coord = square.bitscan().into();
//                         let nots = ts.get_rook(sq).get_dir(dir);
//                         let mm = moves ^ *nots;
//                         let mm = mm & !square;
//                         if (square & self.get_color(!col)).0 != 0 {
//                             // capture
//                             // out.push(Move::Capture { from: p0.into(), to: sq });
//                             let ss: Coord = sq.into();
//                             // eprintln!("ss = {:?}", ss);
//                             out_captures_pos.set_one_mut(sq.into());
//                         };
//                         // mm.iter_bitscan(|t| {
//                         //     // out.push(Move::Quiet { from: p0.into(), to: t.into() });
//                         //     out_quiets_pos.set_one(t.into());
//                         // });
//                         out_quiets_pos |= mm;
//                     } else {
//                         // moves.iter_bitscan(|t| {
//                         //     // out.push(Move::Quiet { from: p0.into(), to: t.into() });
//                         //     out_quiets_pos.set_one(t.into());
//                         // });
//                         out_quiets_pos |= moves;
//                     }
//                 },

//                 // Rook Negative
//                 S | W => {
//                     let blocks = moves & blocks;
//                     if blocks.0 != 0 {
//                         let square = blocks.bitscan_rev_isolate();
//                         let sq: Coord = square.bitscan_rev().into();
//                         let nots = ts.get_rook(sq).get_dir(dir);
//                         let mm = moves ^ *nots;
//                         let mm = mm & !square;
//                         if (square & self.get_color(!col)).0 != 0 {
//                             // capture
//                             // out.push(Move::Capture { from: p0.into(), to: sq });
//                             out_captures_neg.set_one_mut(sq.into());
//                         }
//                         // mm.iter_bitscan_rev(|t| {
//                         //     // out.push(Move::Quiet { from: p0.into(), to: t.into() });
//                         //     out_quiets_neg.set_one(t.into());
//                         // });
//                         out_quiets_neg |= mm;
//                     } else {
//                         // moves.iter_bitscan_rev(|t| {
//                         //     // out.push(Move::Quiet { from: p0.into(), to: t.into() });
//                         //     out_quiets_neg.set_one(t.into());
//                         // });
//                         out_quiets_neg |= moves;
//                     }
//                 },

//                 // Bishop Positive
//                 NE | NW => {
//                     let blocks = moves & blocks;
//                     if blocks.0 != 0 {
//                         let square = blocks.bitscan_isolate();
//                         let sq: Coord = square.bitscan().into();
//                         let nots = ts.get_bishop(sq).get_dir(dir);
//                         let mm = moves ^ *nots;
//                         let mm = mm & !square;
//                         if (square & self.get_color(!col)).0 != 0 {
//                             // capture
//                             // out.push(Move::Capture { from: p0.into(), to: sq });
//                             out_captures_pos.set_one_mut(sq.into());
//                         }
//                         // mm.iter_bitscan(|t| {
//                         //     // out.push(Move::Quiet { from: p0.into(), to: t.into() });
//                         //     out_quiets_pos.set_one(t.into());
//                         // });
//                         out_quiets_pos |= mm;
//                     } else {
//                         // moves.iter_bitscan(|t| {
//                         //     // out.push(Move::Quiet { from: p0.into(), to: t.into() });
//                         //     out_quiets_pos.set_one(t.into());
//                         // });
//                         out_quiets_pos |= moves;
//                     }
//                 },

//                 // Bishop Negative
//                 SE | SW => {
//                     let blocks = moves & blocks;
//                     if blocks.0 != 0 {
//                         let square = blocks.bitscan_rev_isolate();
//                         let sq: Coord = square.bitscan_rev().into();
//                         let nots = ts.get_bishop(sq).get_dir(dir);
//                         let mm = moves ^ *nots;
//                         let mm = mm & !square;
//                         if (square & self.get_color(!col)).0 != 0 {
//                             // capture
//                             // out.push(Move::Capture { from: p0.into(), to: sq });
//                             out_captures_neg.set_one_mut(sq.into());
//                         }
//                         // mm.iter_bitscan_rev(|t| {
//                         //     // out.push(Move::Quiet { from: p0.into(), to: t.into() });
//                         //     out_quiets_neg.set_one(t.into());
//                         // });
//                         out_quiets_neg |= mm;
//                     } else {
//                         // moves.iter_bitscan_rev(|t| {
//                         //     // out.push(Move::Quiet { from: p0.into(), to: t.into() });
//                         //     out_quiets_neg.set_one(t.into());
//                         // });
//                         out_quiets_neg |= moves;
//                     }
//                 },
//             }
//         }

//         // out.push((p0.into(),(out_quiets_pos,out_quiets_neg,out_captures_pos,out_captures_neg)))
//         (out_quiets_pos, out_quiets_neg, out_captures_pos, out_captures_neg)
//     }
// }

