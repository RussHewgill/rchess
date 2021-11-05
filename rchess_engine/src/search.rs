
use std::str::FromStr;
use std::collections::HashMap;

use crate::types::*;
use crate::tables::*;

use rayon::prelude::*;

const PERFTLENTH: usize = 20;

/// Search All
impl Game {

    pub fn search_all_single_to(&self, ts: &Tables, c0: Coord, col: Option<Color>, captures: bool) -> Outcome {
        let ms = if captures {
            self.search_only_captures(&ts)
        } else {
            self.search_all(&ts)
        };
        match ms {
            Outcome::Checkmate(win) => Outcome::Checkmate(win),
            Outcome::Stalemate      => Outcome::Stalemate,
            Outcome::Moves(ms)      => {
                let out: Vec<Move> = ms.into_iter()
                    .filter(|m| m.sq_to() == c0)
                    .collect();
                Outcome::Moves(out)
            },
        }
    }

    // XXX: slower than could be
    pub fn search_all_single_from(&self, ts: &Tables, c0: Coord, col: Option<Color>) -> Outcome {
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

        // let ms = self.search_all(&ts, col);
        let ms = self.search_all(&ts);
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

    pub fn search_only_captures(&self, ts: &Tables) -> Outcome {
        let col = self.state.side_to_move;
        let moves = if self.state.checkers.is_not_empty() {
            self._search_in_check(&ts, col, true)
        } else {
            self._search_all(&ts, col, true)
        };
        // if moves.is_end() {
        //     self.search_all(&ts)
        // } else {
        //     moves
        // }
        moves
    }

    // pub fn search_all(&self, ts: &Tables, col: Color) -> Option<Vec<Move>> {
    // pub fn search_all(&self, ts: &Tables, col: Option<Color>) -> Outcome {
    pub fn search_all(&self, ts: &Tables) -> Outcome {
        // let col = if let Some(c) = col { c } else { self.state.side_to_move };
        let col = self.state.side_to_move;

        if self.state.checkers.is_not_empty() {
            self._search_in_check(&ts, col, false)
        } else {
            self._search_all(&ts, col, false)
        }

        // match self.state.checkers {
        //     Some(cs) if !cs.is_empty() => {
        //         // if let Some(win) = self.is_checkmate(&ts) {
        //         //     return Outcome::Checkmate(win);
        //         // }
        //         // println!("wat check");
        //         self._search_in_check(&ts, col)
        //     },
        //     _                          => {
        //         self._search_all(&ts, col)
        //         // let moves = self._search_all_iters(&ts, col);
        //         // let moves = moves.collect::<Vec<_>>();
        //         // if moves.len() == 0 {
        //         //     Outcome::Stalemate
        //         // } else {
        //         //     Outcome::Moves(moves)
        //         // }
        //     },
        // }

    }

    // pub fn _search_all_iters<'a>(&'a self, ts: &'a Tables, col: Color)
    //                              -> impl Iterator<Item = Move> + 'a {
    //     let k = self.search_king_iter(&ts, col);
    //     let b = self.search_sliding_iter(&ts, Bishop, col);
    //     let r = self.search_sliding_iter(&ts, Rook, col);
    //     let q = self.search_sliding_iter(&ts, Queen, col);
    //     let n = self.search_knights(&ts, col);
    //     let p = self.search_pawns(&ts, col);
    //     let pp = self._search_promotions(&ts, None, col);
    //     let cs = self._search_castles(&ts);
    //     let mut out = k
    //         .chain(b)
    //         .chain(r)
    //         .chain(q)
    //         // .chain(n.into_iter())
    //         // .chain(p.into_iter())
    //         // .chain(pp.into_iter())
    //         // .chain(cs.into_iter())
    //         .filter(move |m| self.move_is_legal(&ts, *m))
    //         ;
    //     out
    //     // vec![].into_iter()
    // }

    pub fn _search_all(&self, ts: &Tables, col: Color, only_caps: bool) -> Outcome {

        let k = self.search_king(&ts, col, only_caps);
        let b = self.search_sliding(&ts, Bishop, col, only_caps);
        let r = self.search_sliding(&ts, Rook, col, only_caps);
        let q = self.search_sliding(&ts, Queen, col, only_caps);
        // let b = self.search_sliding_iter(&ts, Bishop, col).collect();
        // let r = self.search_sliding_iter(&ts, Rook, col).collect();
        // let q = self.search_sliding_iter(&ts, Queen, col).collect();

        // let mut k = self.search_king(&ts, col);
        // // k.extend(self.search_sliding_iter(&ts, Bishop, col));
        // // k.extend(self.search_sliding_iter(&ts, Rook, col));
        // // k.extend(self.search_sliding_iter(&ts, Queen, col));
        // let b = self.search_sliding_iter(&ts, Bishop, col);
        // let r = self.search_sliding_iter(&ts, Rook, col);
        // let q = self.search_sliding_iter(&ts, Queen, col);
        // k.extend(b.chain(r).chain(q));

        let n = self.search_knights(&ts, col, only_caps);
        let p = self.search_pawns(&ts, col, only_caps);
        let pp = self._search_promotions(&ts, None, col, only_caps);
        let cs = if !only_caps {
            self._search_castles(&ts)
        } else { Vec::with_capacity(0) };

        let out = vec![k,b,r,q,n,p,pp,cs].concat();
        // let out = vec![k,n,p,pp,cs].concat();

        // // XXX: par == way slower ?
        // let out: Vec<Move> = out.into_par_iter().filter(|m| {

        // let out: Vec<Move> = out.into_iter().filter(|m| {
        //     self.move_is_legal(&ts, *m)
        // }).collect();

        if out.is_empty() {
            Outcome::Stalemate
        } else {
            Outcome::Moves(out)
        }
    }

    pub fn _search_in_check(&self, ts: &Tables, col: Color, only_caps: bool) -> Outcome {
        let mut out = vec![];

        out.extend(&self.search_king(&ts, col, only_caps));

        // let mut x = 0;
        // self.state.checkers.iter_bitscan(|sq| x += 1);
        let x = self.state.checkers.into_iter().count();
        if x == 1 {
            out.extend(&self.search_sliding(&ts, Bishop, col, only_caps));
            out.extend(&self.search_sliding(&ts, Rook, col, only_caps));
            out.extend(&self.search_sliding(&ts, Queen, col, only_caps));
            out.extend(&self.search_knights(&ts, col, only_caps));
            out.extend(&self.search_pawns(&ts, col, only_caps));
            out.extend(&self._search_promotions(&ts, None, col, only_caps));
        }

        // let out: Vec<Move> = out.into_iter().filter(|m| {
        //     self.move_is_legal(&ts, *m)
        // }).collect();

        if out.is_empty() {
            Outcome::Checkmate(!self.state.side_to_move)
        } else {
            Outcome::Moves(out)
        }
    }

}

/// Perft
impl Game {

    /// Far slower ??
    pub fn perft2(&self, ts: &Tables, depth: Depth) -> (u64, [u64; PERFTLENTH]) {
        let mut vs = [0; PERFTLENTH];
        self._perft2(ts, depth, 0, &mut vs);
        let tot = vs.iter().sum();
        (tot, vs)
    }

    pub fn _perft2(
        &self,
        ts:            &Tables,
        depth:         Depth,
        ply:           usize,
        mut vs:        &mut [u64; PERFTLENTH],
    ) {
        if depth == 0 { return; }

        let moves = self.search_all(ts);
        if moves.is_end() { return; }
        let moves = moves.get_moves_unsafe();

        let gs = moves.into_iter().flat_map(|mv| {
            if let Ok(g2) = self.make_move_unchecked(ts, mv) {
                Some((mv,g2))
            } else { None }
        });

        for (mv, g2) in gs {
            g2._perft2(ts, depth - 1, ply + 1, &mut vs);
            vs[ply] += 1;
        }
    }

    pub fn perft(&self, ts: &Tables, depth: u64) -> (u64,Vec<(Move,u64)>) {
        // let mut nodes = 0;
        // let mut captures = 0;

        if depth == 0 { return (1,vec![]); }

        let moves = self.search_all(&ts);
        if moves.is_end() { return (0,vec![]); }
        let moves = moves.get_moves_unsafe();

        // eprintln!("moves.len() = {:?}", moves.len());
        // let mut k = 0;
        // let mut out = vec![];

        // let out = moves.into_iter().flat_map(|m| {
        let out = moves.into_par_iter().flat_map(|m| {
            if let Ok(g2) = self.make_move_unchecked(&ts, m) {
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

        let moves = self.search_all(&ts);
        if moves.is_end() { return (0,0); }

        // eprintln!("moves.len() = {:?}", moves.len());
        let mut k = 0;
        for m in moves.into_iter() {
            if let Ok(g2) = self.make_move_unchecked(&ts, m) {
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

    /// returns (leaves, collisions)
    pub fn perft_hash_collisions(&self,
                                 ts: &Tables,
                                 mut hs: &mut HashMap<Zobrist, GameState>,
                                 mut cols: &mut Vec<(Move,Zobrist,(GameState,GameState))>,
                                 depth: u64) -> (u64,u64) {
        let mut nodes = 0;
        let mut collisions = 0;

        if depth == 0 { return (1,0); }

        let moves = self.search_all(&ts);
        if moves.is_end() { return (0,0); }

        let moves = moves.into_iter().flat_map(|m| if let Ok(g2) = self.make_move_unchecked(&ts, m) {
            Some((m,g2)) } else { None });

        for (m,g2) in moves.into_iter() {
            if let Some(st0) = hs.insert(g2.zobrist, g2.state) {
                if !st0.game_equal(g2.state) {
                    collisions += 1;
                    cols.push((m,g2.zobrist, (st0,g2.state)));
                }
            }

            let (ns,cs) = g2.perft_hash_collisions(&ts, &mut hs, &mut cols, depth - 1);

            collisions += cs;
            nodes += ns;
        }

        (nodes,collisions)
    }

}

/// move_is_legal
impl Game {

    pub fn move_is_legal(&self, ts: &Tables, m: Move) -> bool {

        if m.filter_en_passant() {
            if self.state.en_passant.is_none() {
                return false;
            } else if let Some(g2) = self.clone()._make_move_unchecked(&ts, &m) {
                let checks = g2.find_checkers(&ts, self.state.side_to_move);
                return checks.is_empty();
            } else {
                return false;
            }
        }

        let col = if self.get_color(White).is_one_at(m.sq_from()) { White } else { Black };

        match m.piece() {
            Some(King) => {
                !self.find_attacks_by_side(&ts, m.sq_to(), !col, true)
            },
            _ => {
                let pins = self.get_pins(col);

                // Not pinned
                // OR moving along pin ray
                let x = (BitBoard::single(m.sq_from()) & pins).is_empty()
                    || (ts.aligned(m.sq_from(), m.sq_to(), self.get(King, col).bitscan().into()).0 != 0);

                // not in check
                let x0 = x & self.state.checkers.is_empty();

                x0 & self.state.checkers.is_empty()
                    || (x && m.sq_to() == self.state.checkers.bitscan().into())
                    || (x && (BitBoard::single(m.sq_to())
                              & self.state.check_block_mask).is_not_empty())
            },
        }

    }

    pub fn move_is_legal2(&self, ts: &Tables, m: Move) -> bool {

        // TODO: En Passant Captures
        // TODO: Castling

        if m.filter_en_passant() {
            if self.state.en_passant.is_none() {
                return false;
            } else if let Some(g2) = self.clone()._make_move_unchecked(&ts, &m) {
                let checks = g2.find_checkers(&ts, self.state.side_to_move);
                return checks.is_empty();
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

                // Not pinned
                // OR moving along pin ray
                let x = (BitBoard::single(m.sq_from()) & pins).is_empty()
                    || (ts.aligned(m.sq_from(), m.sq_to(), self.get(King, col).bitscan().into()).0 != 0);

                // not in check
                let x0 = x & self.state.checkers.is_empty();

                x0 & self.state.checkers.is_empty()
                    || (x && m.sq_to() == self.state.checkers.bitscan().into())
                    || (x && (BitBoard::single(m.sq_to())
                              & self.state.check_block_mask).is_not_empty())


            },
            None => panic!(),
        }
    }

}

/// Helpers
impl Game {

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

        // snipers.iter_bitscan(|sq| {
        snipers.into_iter().for_each(|sq| {
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

        let moves_n = ts.get_knight(c0);
        if (*moves_n & self.get(Knight, col)).is_not_empty() { return true; }

        let occ = if king {
            self.all_occupied() & !self.get(King, !col)
        } else {
            self.all_occupied()
        };

        let moves_r = self._search_sliding_single(&ts, Rook, c0, !col, Some(occ));
        if ((moves_r & self.get(Rook, col)).is_not_empty())
            | ((moves_r & self.get(Queen, col)).is_not_empty()) { return true; }

        let moves_b = self._search_sliding_single(&ts, Bishop, c0, !col, Some(occ));
        if ((moves_b & self.get(Bishop, col)).is_not_empty())
            | ((moves_b & self.get(Queen, col)).is_not_empty()) { return true; }

        false
    }

    pub fn find_attackers_to(&self, ts: &Tables, c0: Coord, col: Color) -> BitBoard {

        let pawns = ts.get_pawn(c0);
        // let pawns = *pawns.get_capture(White) | *pawns.get_capture(Black);
        let pawns = *pawns.get_capture(col);
        let pawns = pawns & self.get_piece(Pawn);

        let knights = *ts.get_knight(c0) & self.get_piece(Knight);

        let moves_r = self._search_sliding_single(&ts, Rook, c0, col, None);
            // | self._search_sliding_single(&ts, Rook, c0, !col, Some(occ));
        let rooks = moves_r & (self.get_piece(Rook) | self.get_piece(Queen));

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

    pub fn search_sliding(&self, ts: &Tables, pc: Piece, col: Color, only_caps: bool) -> Vec<Move> {
        let mut out = vec![];
        let pieces = self.get(pc, col);

        for sq in pieces.into_iter() {
            let moves   = self._search_sliding_single(&ts, pc, sq.into(), col, None);
            let attacks = moves & self.get_color(!col);
            let quiets  = moves & self.all_empty();
            attacks.into_iter().for_each(|sq2| {
                let to = sq2.into();
                let (_,victim) = self.get_at(to).unwrap();

                // let (_,victim) = match self.get_at(to) {
                //     Some(x) => x,
                //     None    => {
                //         let f: Coord = sq.into();
                //         panic!("get_at: from: {:?} to: {:?}, g = {:?}",
                //                f, to, &self);
                //     },
                // };

                // out.push(Move::Capture { from: sq.into(), to, pc, victim });
                let m = Move::Capture { from: sq.into(), to, pc, victim };
                if self.move_is_legal(&ts, m) { out.push(m); }
                // out.push(m);
            });
            if !only_caps {
                quiets.into_iter().for_each(|sq2| {
                    // out.push(Move::Quiet { from: sq.into(), to: sq2.into() });
                    let m = Move::Quiet { from: sq.into(), to: sq2.into(), pc };
                    if self.move_is_legal(&ts, m) { out.push(m); }
                    // out.push(m);
                });
            }
        }

        out
    }

    pub fn _search_sliding_single(
        &self,
        ts:     &Tables,
        pc:     Piece,
        c0:     Coord,
        col:    Color,
        occ:    Option<BitBoard>,
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

/// King + Castles
impl Game {

    pub fn _search_castles(&self, ts: &Tables) -> Vec<Move> {
        let mut out = vec![];
        let col = self.state.side_to_move;
        let (kingside,queenside) = self.state.castling.get_color(col);

        if self.state.checkers.is_not_empty() { return out; }

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

    pub fn search_king(&self, ts: &Tables, col: Color, only_caps: bool) -> Vec<Move> {
        self._search_king(&ts, col, true, only_caps)
    }


    pub fn _search_king(
        &self,
        ts:               &Tables,
        col:              Color,
        forbid_check:     bool,
        only_caps:        bool,
    ) -> Vec<Move> {

        let p0 = self.get(King, col).bitscan();
        if p0 == 64 { return vec![]; }
        let moves = *ts.get_king(p0);

        let oc = self.all_occupied();
        let quiets   = moves & !oc;
        let captures = moves & self.get_color(!col);

        let mut out = vec![];

        if !only_caps {
            quiets.into_iter().for_each(|sq| {
                let go = if forbid_check {
                    // let mut threats = self.find_attacks_to(&ts, sq.into(), !col);
                    // threats.next().is_none()
                    !self.find_attacks_by_side(&ts, sq.into(), !col, true)
                } else {
                    true
                };
                if go {
                    let m = Move::Quiet { from: p0.into(), to: sq.into(), pc: King };
                    // if self.move_is_legal(&ts, m) { out.push(m); }
                    out.push(m);
                }
            });
        }

        // captures.iter_bitscan(|sq| {
        captures.into_iter().for_each(|sq| {
            let to = sq.into();
            let go = if forbid_check {
                // let mut threats = self.find_attacks_to(&ts, sq.into(), !col);
                // threats.next().is_none()
                !self.find_attacks_by_side(&ts, to, !col, true)
            } else {
                true
            };
            if go {
                let (_,victim) = self.get_at(to).unwrap();
                let m = Move::Capture { from: p0.into(), to, pc: King, victim };
                // if self.move_is_legal(&ts, m) { out.push(m); }
                out.push(m);
                // out.push(Move::Capture { from: p0.into(), to, pc: King, victim });
                // out.push(Move::Capture { from: p0.into(), to});
            }
        });

        // b3
        out
    }

    // pub fn search_king_iter<'a>(&'a self, ts: &'a Tables, col: Color)
    //                             -> impl Iterator<Item = Move> + 'a {
    //     self._search_king_iter(&ts, col, true)
    // }

    // pub fn _search_king_iter<'a>(&'a self, ts: &'a Tables, col: Color, forbid_check: bool)
    //                         -> impl Iterator<Item = Move> + 'a {
    //     let p0 = self.get(King, col).bitscan();
    //     // if p0 == 64 { return vec![].into_iter() }
    //     let moves = *ts.get_king(p0);

    //     let moves = moves & !self.get_color(col);

    //     let attacks = self.get_color(!col);

    //     let p1: Coord = p0.into();
    //     moves.into_iter().flat_map(move |to| {
    //         let to: Coord = to.into();
    //         if attacks.is_one_at(to) {
    //             if forbid_check && !self.find_attacks_by_side(&ts, to, !col, false) {
    //                 let (_,victim) = self.get_at(to).unwrap();
    //                 Some(Move::Capture { from: p1, to: to, pc: King, victim })
    //                 // Some(Move::Capture { from: p1, to: to})
    //             } else {
    //                 None
    //             }
    //         } else {
    //             if forbid_check && !self.find_attacks_by_side(&ts, to, !col, false) {
    //                 Some(Move::Quiet { from: p1, to: to, pc: King })
    //             } else {
    //                 None
    //             }
    //         }
    //     })

    // }

    // pub fn _search_king_single(&self, c0: Coord, ts: &Tables, col: Color, forbid_check: bool)
    //                            -> Vec<Move> {
    //     // let mut out = vec![];
    //     let occ = self.all_occupied();

    //     let moves = ts.get_king(c0);
    //     // let quiets   = b3 & !oc;
    //     // let captures = b3 & oc;

    //     eprintln!("moves = {:?}", moves);

    //     unimplemented!()
    //     // out
    // }

    // pub fn search_knight(&self, )

}

/// Knights
impl Game {

    pub fn search_knights(&self, ts: &Tables, col: Color, only_caps: bool) -> Vec<Move> {
        self._search_knights(None, ts, col, only_caps)
    }

    pub fn _search_knights(
        &self,
        single:       Option<Coord>,
        ts:           &Tables,
        col:          Color,
        only_caps:    bool,
    ) -> Vec<Move> {
        let mut out = vec![];
        let oc = self.all_occupied();

        let ks = match single {
            Some(c0) => BitBoard::single(c0),
            None     => self.get(Knight, col),
        };

        ks.into_iter().for_each(|sq| {
            let ms = ts.get_knight(sq);

            let quiets   = *ms & !oc;
            let captures = *ms & self.get_color(!col);

            if !only_caps {
                quiets.into_iter().for_each(|t| {
                    // out.push(Move::Quiet { from: sq.into(), to: t.into()});
                    let m = Move::Quiet { from: sq.into(), to: t.into(), pc: Knight };
                    if self.move_is_legal(&ts, m) { out.push(m); }
                });
            }

            captures.into_iter().for_each(|t| {
                let (_,victim) = self.get_at(t.into()).unwrap();
                // out.push(Move::Capture { from: sq.into(), to: t.into(), pc: Knight, victim});
                // out.push(Move::Capture { from: sq.into(), to: t.into()});
                let m = Move::Capture { from: sq.into(), to: t.into(), pc: Knight, victim};
                if self.move_is_legal(&ts, m) { out.push(m); }
            });

        });

        out
    }

}

/// Pawns + Promotions
impl Game {

    pub fn search_pawns(&self, ts: &Tables, col: Color, only_caps: bool) -> Vec<Move> {
        self._search_pawns(&ts, None, col, only_caps)
    }

    pub fn _search_pawns(&self, ts: &Tables, single: Option<Coord>, col: Color, only_caps: bool)
                         -> Vec<Move> {
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

        // let b = doubles;
        // eprintln!("{:?}", b);

        if !only_caps {
            let pushes = ps.shift_dir(dir);
            let pushes = pushes & !(oc);

            let doubles = ps & BitBoard::mask_rank(if col == White { 1 } else { 6 });
            let doubles = doubles.shift_mult(dir, 2);
            let doubles = doubles & !(oc) & (!(oc)).shift_dir(dir);

            pushes.into_iter().for_each(|t| {
                let t = t.into();
                if let Some(f) = (!dir).shift_coord(t) {
                    // out.push(Move::Quiet { from: f, to: t });
                    let m = Move::Quiet { from: f, to: t, pc: Pawn };
                    if self.move_is_legal(&ts, m) { out.push(m); }
                }
            });

            doubles.into_iter().for_each(|t| {
                let f = BitBoard::single(t.into()).shift_mult(!dir, 2);
                // out.push(Move::PawnDouble { from: f.bitscan().into(), to: t.into() })
                let m = Move::PawnDouble { from: f.bitscan().into(), to: t.into() };
                if self.move_is_legal(&ts, m) { out.push(m); }
            });
        }

        ps.into_iter().for_each(|p0| {
            let f  = BitBoard::index_bit(p0);
            let bb = BitBoard::empty().flip(f);
            let mut cs = (bb.shift_dir(dw) & self.get_color(!col))
                | (bb.shift_dir(de) & self.get_color(!col));
            while cs.0 != 0 {
                let t = cs.bitscan_reset_mut();
                let (_,victim) = self.get_at(t.into()).unwrap();
                // out.push(Move::Capture { from: f, to: t.into(), pc: Pawn, victim });
                // out.push(Move::Capture { from: f, to: t.into()});
                let m = Move::Capture { from: f, to: t.into(), pc: Pawn, victim };
                if self.move_is_legal(&ts, m) { out.push(m); }
            }
        });

        if let Some(ep) = self.state.en_passant {
            let attacks = ts.get_pawn(ep).get_capture(!col);
            let attacks = *attacks & ps;
            attacks.into_iter().for_each(|sq| {
                let capture = if col == White { S.shift_coord(ep) } else { N.shift_coord(ep) };
                let capture = capture
                    // .expect(&format!("en passant bug? ep: {:?}, capture: {:?}", ep, capture));
                    .unwrap_or_else(|| panic!("en passant bug? ep: {:?}, capture: {:?}", ep, capture));
                // let (_,victim) = self.get_at(capture).unwrap();
                // out.push(Move::EnPassant { from: sq.into(), to: ep, capture, victim });
                let m = Move::EnPassant { from: sq.into(), to: ep, capture };
                if self.move_is_legal(&ts, m) { out.push(m); }
            });
        }

        out
    }

    pub fn _search_promotions(
        &self,
        ts:         &Tables,
        single:     Option<Coord>,
        col:        Color,
        only_caps:  bool,
    ) -> Vec<Move> {
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

        let pushes = ps.shift_dir(dir);
        let pushes = pushes & !(oc);

        if !only_caps {
            pushes.into_iter().for_each(|t| {
                let t = t.into();
                if let Some(f) = (!dir).shift_coord(t) {
                    // out.push(Move::Quiet { from: f, to: t });
                    // out.push(Move::Promotion { from: f, to: t, new_piece: Queen });
                    // out.push(Move::Promotion { from: f, to: t, new_piece: Knight });
                    // out.push(Move::Promotion { from: f, to: t, new_piece: Rook });
                    // out.push(Move::Promotion { from: f, to: t, new_piece: Bishop });

                    // let m = Move::Promotion { from: f, to: t, new_piece: Queen };
                    // if self.move_is_legal(&ts, m) { out.push(m); }
                    // let m = Move::Promotion { from: f, to: t, new_piece: Knight };
                    // if self.move_is_legal(&ts, m) { out.push(m); }
                    // let m = Move::Promotion { from: f, to: t, new_piece: Rook };
                    // if self.move_is_legal(&ts, m) { out.push(m); }
                    // let m = Move::Promotion { from: f, to: t, new_piece: Bishop };
                    // if self.move_is_legal(&ts, m) { out.push(m); }

                    // XXX: would this ever be different?
                    let m = Move::Promotion { from: f, to: t, new_piece: Queen };
                    let legal = self.move_is_legal(&ts, m);
                    if legal { out.push(m); }
                    let m = Move::Promotion { from: f, to: t, new_piece: Knight };
                    if legal { out.push(m); }
                    let m = Move::Promotion { from: f, to: t, new_piece: Rook };
                    if legal { out.push(m); }
                    let m = Move::Promotion { from: f, to: t, new_piece: Bishop };
                    if legal { out.push(m); }

                }
            });
        }

        ps.into_iter().for_each(|p0| {
            let f  = BitBoard::index_bit(p0);
            let bb = BitBoard::empty().flip(f);
            let mut cs = (bb.shift_dir(dw) & self.get_color(!col))
                | (bb.shift_dir(de) & self.get_color(!col));
            while cs.0 != 0 {
                let t = cs.bitscan_reset_mut().into();
                let (_,victim) = self.get_at(t).unwrap();
                // out.push(Move::PromotionCapture { from: f, to: t, new_piece: Queen, victim });
                // out.push(Move::PromotionCapture { from: f, to: t, new_piece: Knight, victim });
                // out.push(Move::PromotionCapture { from: f, to: t, new_piece: Rook, victim });
                // out.push(Move::PromotionCapture { from: f, to: t, new_piece: Bishop, victim });

                // let m = Move::PromotionCapture { from: f, to: t, new_piece: Queen, victim };
                // if self.move_is_legal(&ts, m) { out.push(m); }
                // let m = Move::PromotionCapture { from: f, to: t, new_piece: Knight, victim };
                // if self.move_is_legal(&ts, m) { out.push(m); }
                // let m = Move::PromotionCapture { from: f, to: t, new_piece: Rook, victim };
                // if self.move_is_legal(&ts, m) { out.push(m); }
                // let m = Move::PromotionCapture { from: f, to: t, new_piece: Bishop, victim };
                // if self.move_is_legal(&ts, m) { out.push(m); }

                let m = Move::PromotionCapture { from: f, to: t, new_piece: Queen, victim };
                let legal = self.move_is_legal(&ts, m);
                if legal { out.push(m); }
                let m = Move::PromotionCapture { from: f, to: t, new_piece: Knight, victim };
                if legal { out.push(m); }
                let m = Move::PromotionCapture { from: f, to: t, new_piece: Rook, victim };
                if legal { out.push(m); }
                let m = Move::PromotionCapture { from: f, to: t, new_piece: Bishop, victim };
                if legal { out.push(m); }


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

