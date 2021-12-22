
pub use self::pieces::*;
use crate::explore::ABStack;
use crate::types::*;
use crate::tables::*;
use crate::move_ordering::score_move_for_sort;

use arrayvec::ArrayVec;

// use strum::{IntoEnumIterator,EnumIter};

#[derive(Debug,Eq,PartialEq,Ord,PartialOrd,Clone,Copy)]
pub enum MoveGenType {
    Captures, // also queen promotions
    Quiets, // also under-promotions
    Evasions,
    QuietChecks,
    // Pseudo,
    // AllLegal,
}

// #[derive(Debug,Eq,PartialEq,Ord,PartialOrd,Clone,Copy,EnumIter)]
#[derive(Debug,Eq,PartialEq,Ord,PartialOrd,Clone,Copy)]
pub enum MoveGenStage {
    // Init = 0,
    Hash,
    CounterMove,

    CapturesInit,
    Captures,
    QuietsInit,
    Quiets,

    EvasionHash,
    EvasionInit,
    Evasion,

    GenAllInit,
    GenAll,

    Finished,
}

impl MoveGenStage {
    pub fn next(self) -> Option<Self> {
        use MoveGenStage::*;
        match self {
            Hash         => Some(CounterMove),
            CounterMove  => Some(CapturesInit),

            CapturesInit => Some(Captures),
            Captures     => Some(QuietsInit),
            QuietsInit   => Some(Quiets),
            Quiets       => Some(Finished),

            EvasionHash  => Some(EvasionInit),
            EvasionInit  => Some(Evasion),
            Evasion      => Some(Finished),

            GenAllInit   => Some(GenAll),
            GenAll       => None,

            Finished     => None,
        }
    }
}

#[derive(Debug,Clone)]
pub struct MoveGen<'a> {
    ts:                  &'static Tables,
    game:                &'a Game,

    in_check:            bool,
    side:                Color,

    stage:               MoveGenStage,
    buf:                 ArrayVec<Move,256>,

    pub hashmove:        Option<Move>,
    pub counter_move:    Option<Move>,

    depth:               Depth,
    ply:                 Depth,
}

/// Sort
impl<'a> MoveGen<'a> {
    pub fn sort(&mut self, st: &ABStack) {
        let mut buf = &mut self.buf;
        let ts      = self.ts;
        let g       = self.game;
        let ply     = self.ply;
        let stage   = self.stage;

        #[cfg(feature = "killer_moves")]
        let killers = match st.stacks.get(ply as usize) {
            Some(ks) => (ks.killers.get(0).copied(),ks.killers.get(1).copied()),
            _        => (None,None)
        };
        // let killers = st.killers.get(g.state.side_to_move,ply);
        #[cfg(not(feature = "killer_moves"))]
        let killers = (None,None);

        buf.sort_by_cached_key(|&mv| {
            // std::cmp::Reverse(crate::move_ordering::score_move_for_sort(ts, g, st, ply, mv, killers))
            score_move_for_sort(ts, g, stage, st, ply, mv, killers)
        });

    }
}

/// New, getters
impl<'a> MoveGen<'a> {
    pub fn new(
        ts:             &'static Tables,
        game:           &'a Game,
        hashmove:       Option<Move>,
        // counter_move:   Option<Move>,
        depth:          Depth,
        ply:            Depth,
    ) -> Self {
        let in_check = game.state.checkers.is_not_empty();
        let side = game.state.side_to_move;

        let counter_move = None;

        let mut out = Self {
            ts,
            game,
            in_check,
            side,
            stage:     if in_check { MoveGenStage::EvasionHash } else { MoveGenStage::Hash },
            buf:       ArrayVec::new(),

            hashmove,
            counter_move,

            depth,
            ply,
        };
        // XXX: check for legal move incase of corrupted TTEntry
        let hashmove = if let Some (mv) = hashmove {
            if out.move_is_legal(mv) { Some(mv) } else { None }
        } else { None };
        out.hashmove = hashmove;
        out
    }

    pub fn new_all(
        ts:             &'static Tables,
        game:           &'a Game,
        hashmove:       Option<Move>,
        depth:          Depth,
        ply:            Depth,
    ) -> Self {
        let mut out = Self::new(ts, game, hashmove, depth, ply);
        out.stage = MoveGenStage::GenAllInit;
        out
    }

    pub fn buf(&self) -> &[Move] {
        &self.buf
    }

    pub fn buf_legal(&self) -> impl Iterator<Item = &Move> {
        self.buf.iter().filter(move |mv| self.move_is_legal(**mv))
    }

}

/// Not quite Iter
impl<'a> MoveGen<'a> {

    pub fn next_all(&mut self, stack: &ABStack) -> Option<Move> {
        match self.stage {
            MoveGenStage::GenAll => {
                if let Some(mv) = self.buf.pop() {
                    if self.move_is_legal(mv) {
                        return Some(mv);
                    } else {
                        return self.next_all(stack);
                    }
                }
                self.stage = MoveGenStage::Finished;
                None
            },
            MoveGenStage::GenAllInit => {
                if self.in_check {
                    self.generate(MoveGenType::Evasions);
                } else {
                    self.generate(MoveGenType::Captures);
                    self.generate(MoveGenType::Quiets);
                }
                self.sort(stack);

                self.stage = self.stage.next()?;
                self.next_all(stack)
            }
            _ => unimplemented!(),
            // _ => None,
        }
    }

    pub fn next(&mut self, stack: &ABStack) -> Option<Move> {
        match self.stage {

            MoveGenStage::Hash     => {
                if let Some(mv) = self.hashmove {
                    if self.move_is_legal(mv) {
                        // println!("returning hashmove: {:?}: {:?}", self.game.to_fen(), mv);
                        self.stage = self.stage.next()?;
                        return Some(mv);
                    }
                }
                self.stage = self.stage.next()?;
                self.next(stack)
            },
            MoveGenStage::CounterMove => {
                if let Some(mv) = self.counter_move {
                    if self.move_is_legal(mv) {
                        self.stage = self.stage.next()?;
                        return Some(mv);
                    }
                }
                self.stage = self.stage.next()?;
                self.next(stack)
            },

            MoveGenStage::CapturesInit => {
                self.generate(MoveGenType::Captures);
                // TODO: sort here
                self.sort(stack);

                self.stage = self.stage.next()?;
                self.next(stack)
            },
            MoveGenStage::Captures => {
                if let Some(mv) = self.buf.pop() {
                    if Some(mv) != self.hashmove && self.move_is_legal(mv) {
                        return Some(mv);
                    } else {
                        return self.next(stack);
                    }
                }

                self.stage = self.stage.next()?;
                self.next(stack)
            },
            MoveGenStage::QuietsInit => {
                self.generate(MoveGenType::Quiets);
                // TODO: sort here?
                self.sort(stack);

                self.stage = self.stage.next()?;
                self.next(stack)
            },
            MoveGenStage::Quiets => {
                if let Some(mv) = self.buf.pop() {
                    if Some(mv) != self.hashmove && self.move_is_legal(mv) {
                        return Some(mv);
                    } else {
                        return self.next(stack);
                    }
                }
                self.stage = self.stage.next()?;
                None
            },

            MoveGenStage::EvasionHash => {
                if let Some(mv) = self.hashmove {
                    if self.move_is_legal(mv) {
                        self.stage = self.stage.next()?;
                        return Some(mv);
                    }
                }

                self.stage = self.stage.next()?;
                self.next(stack)
            },

            MoveGenStage::EvasionInit => {
                self.generate(MoveGenType::Evasions);
                // TODO: sort here?
                self.sort(stack);

                self.stage = self.stage.next()?;
                self.next(stack)
            },
            MoveGenStage::Evasion => {
                if let Some(mv) = self.buf.pop() {
                    if Some(mv) != self.hashmove && self.move_is_legal(mv) {
                        return Some(mv);
                    } else {
                        return self.next(stack);
                    }
                }
                self.stage = self.stage.next()?;
                None
            },

            MoveGenStage::GenAll     => self.next_all(stack),
            MoveGenStage::GenAllInit => self.next_all(stack),

            MoveGenStage::Finished => {
            // _ => {
                None
            },
        }
    }

}

// impl<'a> Iterator for MoveGen<'a> {
//     type Item = Move;
//     fn next(&mut self) -> Option<Self::Item> {
//         match self.stage {
//             MoveGenStage::Hash     => {
//                 if let Some(mv) = self.hashmove {
//                     if self.move_is_legal(mv) {
//                         // println!("returning hashmove: {:?}: {:?}", self.game.to_fen(), mv);
//                         self.stage = self.stage.next()?;
//                         return Some(mv);
//                     }
//                 }
//                 self.stage = self.stage.next()?;
//                 self.next()
//             },
//             MoveGenStage::Captures(true) => {
//                 self.generate(MoveGenType::Captures);
//                 // TODO: sort here?
//                 self.stage = self.stage.next()?;
//                 self.next()
//             },
//             MoveGenStage::Captures(false) => {
//                 if let Some(mv) = self.buf.pop() {
//                     if Some(mv) != self.hashmove && self.move_is_legal(mv) {
//                         return Some(mv);
//                     } else {
//                         return self.next();
//                     }
//                 }
//                 self.stage = self.stage.next()?;
//                 self.next()
//             },
//             MoveGenStage::Quiets(true) => {
//                 self.generate(MoveGenType::Quiets);
//                 // TODO: sort here?
//                 self.stage = self.stage.next()?;
//                 self.next()
//             },
//             MoveGenStage::Quiets(false) => {
//                 if let Some(mv) = self.buf.pop() {
//                     if Some(mv) != self.hashmove && self.move_is_legal(mv) {
//                         return Some(mv);
//                     } else {
//                         return self.next();
//                     }
//                 }
//                 self.stage = self.stage.next()?;
//                 None
//             },

//             MoveGenStage::EvasionHash => {
//                 if let Some(mv) = self.hashmove {
//                     if self.move_is_legal(mv) {
//                         self.stage = self.stage.next()?;
//                         return Some(mv);
//                     }
//                 }
//                 self.stage = self.stage.next()?;
//                 self.next()
//             },
//             MoveGenStage::Evasion(true) => {
//                 self.generate(MoveGenType::Evasions);
//                 // TODO: sort here?
//                 self.stage = self.stage.next()?;
//                 self.next()
//             },
//             MoveGenStage::Evasion(false) => {
//                 if let Some(mv) = self.buf.pop() {
//                     if Some(mv) != self.hashmove && self.move_is_legal(mv) {
//                         return Some(mv);
//                     } else {
//                         return self.next();
//                     }
//                 }
//                 self.stage = self.stage.next()?;
//                 None
//             },

//             MoveGenStage::Finished => {
//                 None
//             },
//         }
//     }
// }

/// Generate
impl<'a> MoveGen<'a> {
    // pub fn gen_to_vec(&mut self, gen: MoveGenType)

    fn generate(&mut self, gen: MoveGenType) {
        if self.in_check {
            self._gen_all_in_check(gen);
        } else {
            self._gen_all(gen);
        }
    }

    fn _gen_all(&mut self, gen: MoveGenType) {
        self.gen_pawns(gen);
        self.gen_knights(gen);
        self.gen_sliding(gen, Bishop);
        self.gen_sliding(gen, Rook);
        self.gen_sliding(gen, Queen);
        self.gen_king(gen);
        if gen == MoveGenType::Quiets {
            self.gen_castles();
        }
    }

    fn _gen_all_in_check(&mut self, gen: MoveGenType) {
        self.gen_king(gen);

        let num_checkers = self.game.state.checkers.into_iter().count();
        if num_checkers == 0 {
            panic!();
        } else if num_checkers == 1 {

            self.gen_pawns(gen);
            self.gen_knights(gen);
            self.gen_sliding(gen, Bishop);
            self.gen_sliding(gen, Rook);
            self.gen_sliding(gen, Queen);

        } else {
            // double check
            return;
        }

    }

}

/// Perft
impl<'a> MoveGen<'a> {

    pub fn perft(ts: &'static Tables, g: &'a Game, depth: Depth) -> (u64,Vec<(Move,u64)>) {
        let depth = depth.max(1);
        let mut out = vec![];
        let mut sum = 0;
        let mut gen = Self::new(ts, &g, None, depth, 0);

        let stack = ABStack::new();

        while let Some(mv) = gen.next(&stack) {
            if let Ok(g2) = g.make_move_unchecked(&ts, mv) {
                let sum2 = Self::_perft(ts, &stack, g2, depth - 1);
                out.push((mv,sum2));
                sum += sum2;
            }
        }
        // Self::_perft(ts, g, depth)
        (sum,out)
    }

    pub fn _perft(ts: &'static Tables, st: &ABStack, g: Game, depth: Depth) -> u64 {
        if depth == 0 { return 1; }

        let mut gen = MoveGen::new(ts, &g, None, depth, 0);

        let mut sum = 0;

        while let Some(mv) = gen.next(st) {
            if let Ok(g2) = g.make_move_unchecked(&ts, mv) {
                let sum2 = Self::_perft(ts, st, g2, depth - 1);
                sum += sum2;
            }
        }

        sum
    }

}

/// Check legal
impl<'a> MoveGen<'a> {
    pub fn move_is_legal(&self, mv: Move) -> bool {

        if self.game.state.side_to_move != self.side {
            // debug!("non legal move, wrong color? {:?}", mv);
            // return false;
            panic!("non legal move, wrong color? {:?}", mv);
        }

        if mv.filter_en_passant() {
            if self.game.state.en_passant.is_none() {
                return false;
            } else if let Some(g2) = self.game.clone()._apply_move_unchecked(self.ts, mv, false) {
                let checks = g2.find_checkers(self.ts, self.game.state.side_to_move);
                return checks.is_empty();
            } else {
                return false;
            }
        }

        match mv.piece() {
            Some(King) => {
                !self.game.find_attacks_by_side(self.ts, mv.sq_to(), !self.side, true)
            },
            _ => {
                let pins = self.game.get_pins(self.side);

                // Not pinned
                // OR moving along pin ray
                let x = (BitBoard::single(mv.sq_from()) & pins).is_empty()
                    || (self.ts.aligned(mv.sq_from(), mv.sq_to(),
                                        self.game.get(King, self.side).bitscan().into()).0 != 0);

                // not in check
                let x0 = x & self.game.state.checkers.is_empty();

                x0 && self.game.state.checkers.is_empty()
                    || (x && mv.sq_to() == self.game.state.checkers.bitscan().into())
                    || (x && (BitBoard::single(mv.sq_to())
                              & self.game.state.check_block_mask).is_not_empty())
            },
        }
    }
}

mod pieces {
    use super::*;

    /// Pawns
    impl<'a> MoveGen<'a> {
        pub fn gen_pawns(&mut self, gen: MoveGenType) {

            let occ = self.game.all_occupied();
            let rank7 = if self.side == White { BitBoard::mask_rank(6) } else { BitBoard::mask_rank(1) };
            let rank3 = if self.side == White { BitBoard::mask_rank(2) } else { BitBoard::mask_rank(5) };

            let (dir,dw,de) = match self.side {
                White => (N,NW,NE),
                Black => (S,SW,SE),
            };

            let ps = self.game.get(Pawn, self.side);
            let ps = ps & !rank7;

            match gen {
                MoveGenType::Captures => {

                    // let enemies = self.game.get_color(!self.side);

                    // let b1 = ps.shift_dir(dw) & enemies;
                    // let b2 = ps.shift_dir(de) & enemies;

                    // for to in b1.into_iter() {
                    //     let (_,victim) = self.game.get_at(to).unwrap();
                    //     if let Some(from) = (!dw).shift_coord(to) {
                    //         let mv = Move::Capture { from, to, pc: Pawn, victim };
                    //         self.buf.push(mv);
                    //     }
                    // }

                    for f in ps.into_iter() {
                        let bb = BitBoard::single(f);
                        let mut cs = (bb.shift_dir(dw) & self.game.get_color(!self.side))
                            | (bb.shift_dir(de) & self.game.get_color(!self.side));
                        while cs.0 != 0 {
                            let t = cs.bitscan_reset_mut();
                            let (_,victim) = self.game.get_at(t.into()).unwrap();
                            let mv = Move::Capture { from: f, to: t.into(), pc: Pawn, victim };
                            self.buf.push(mv);
                        }
                    }

                    if let Some(ep) = self.game.state.en_passant {
                        let attacks = self.ts.get_pawn(ep).get_capture(!self.side);
                        let attacks = *attacks & ps;
                        attacks.into_iter().for_each(|sq| {
                            let capture = if self.side == White { S.shift_coord(ep) } else { N.shift_coord(ep) };
                            let capture = capture
                                .unwrap_or_else(
                                    || panic!("en passant bug? ep: {:?}, capture: {:?}", ep, capture));
                            let mv = Move::EnPassant { from: sq.into(), to: ep, capture };
                            self.buf.push(mv);
                        });
                    }

                    self.gen_promotions(gen);
                },
                MoveGenType::Quiets => {
                    let pushes = ps.shift_dir(dir);
                    let pushes = pushes & !(occ);

                    let doubles = ps & BitBoard::mask_rank(if self.side == White { 1 } else { 6 });
                    let doubles = doubles.shift_mult(dir, 2);
                    let doubles = doubles & !(occ) & (!(occ)).shift_dir(dir);

                    for to in pushes.into_iter() {
                        if let Some(f) = (!dir).shift_coord(to) {
                            let mv = Move::Quiet { from: f, to, pc: Pawn };
                            self.buf.push(mv);
                        }
                    };

                    for to in doubles.into_iter() {
                        let f = BitBoard::single(to.into()).shift_mult(!dir, 2);
                        let mv = Move::PawnDouble { from: f.bitscan().into(), to: to.into() };
                        self.buf.push(mv);
                    }

                    self.gen_promotions(gen);
                },
                MoveGenType::Evasions    => {
                    self.gen_pawns(MoveGenType::Captures);
                    self.gen_pawns(MoveGenType::Quiets);
                },
                MoveGenType::QuietChecks => unimplemented!(),
            }
        }

        pub fn gen_promotions(&mut self, gen: MoveGenType) {

            let rank7 = if self.side == White { BitBoard::mask_rank(6) } else { BitBoard::mask_rank(1) };

            let ps = self.game.get(Pawn, self.side);
            let ps = ps & rank7;

            let occ = self.game.all_occupied();
            let (dir,dw,de) = match self.side {
                White => (N,NW,NE),
                Black => (S,SW,SE),
            };

            let pushes = ps.shift_dir(dir);
            let pushes = pushes & !(occ);

            for to in pushes.into_iter() {
                if let Some(from) = (!dir).shift_coord(to) {
                    if gen == MoveGenType::Captures {
                        let mv = Move::Promotion { from, to, new_piece: Queen };
                        self.buf.push(mv);
                    } else if gen == MoveGenType::Quiets {
                        let mv = Move::Promotion { from, to, new_piece: Knight };
                        self.buf.push(mv);
                        let mv = Move::Promotion { from, to, new_piece: Bishop };
                        self.buf.push(mv);
                        let mv = Move::Promotion { from, to, new_piece: Rook };
                        self.buf.push(mv);
                    }
                }
            }

            for from in ps.into_iter() {
                let bb = BitBoard::single(from);
                let mut cs = (bb.shift_dir(dw) & self.game.get_color(!self.side))
                    | (bb.shift_dir(de) & self.game.get_color(!self.side));

                for to in cs.into_iter() {
                    let (_,victim) = self.game.get_at(to).unwrap();
                    if gen == MoveGenType::Captures {
                        let mv = Move::PromotionCapture { from, to, new_piece: Queen, victim };
                        self.buf.push(mv);
                    } else if gen == MoveGenType::Quiets {
                        let mv = Move::PromotionCapture { from, to, new_piece: Knight, victim };
                        self.buf.push(mv);
                        let mv = Move::PromotionCapture { from, to, new_piece: Bishop, victim };
                        self.buf.push(mv);
                        let mv = Move::PromotionCapture { from, to, new_piece: Rook, victim };
                        self.buf.push(mv);
                    }
                }

            }

        }

    }

    /// Knights
    impl<'a> MoveGen<'a> {

        pub fn gen_knights(&mut self, gen: MoveGenType) {
            let occ = self.game.all_occupied();
            let ks = self.game.get(Knight, self.side);

            match gen {
                MoveGenType::Captures => {
                    ks.into_iter().for_each(|from| {
                        let ms = self.ts.get_knight(from);
                        let captures = ms & self.game.get_color(!self.side);
                        captures.into_iter().for_each(|to| {
                            let (_,victim) = self.game.get_at(to).unwrap();
                            let mv = Move::Capture { from, to, pc: Knight, victim };
                            self.buf.push(mv);
                        });
                    });
                },
                MoveGenType::Quiets => {
                    ks.into_iter().for_each(|from| {
                        let ms = self.ts.get_knight(from);
                        let quiets = ms & !occ;
                        quiets.into_iter().for_each(|to| {
                            let mv = Move::Quiet { from, to, pc: Knight };
                            self.buf.push(mv);
                        });
                    });
                },
                MoveGenType::Evasions    => {
                    self.gen_knights(MoveGenType::Captures);
                    self.gen_knights(MoveGenType::Quiets);
                },
                MoveGenType::QuietChecks => unimplemented!(),
            }

        }

        // pub fn gen_knights(&'a self, gen: MoveGenType) -> impl Iterator<Item = Move> + 'a {
        //     let occ = self.game.all_occupied();
        //     let ks = self.game.get(Knight, self.side);
        //     ks.into_iter().flat_map(move |from| {
        //         let ms = self.ts.get_knight(from);
        //         match gen {
        //             MoveGenType::Captures => {
        //                 let captures = *ms & self.game.get_color(!self.side);
        //                 captures.into_iter().map(move |to| {
        //                     let (_,victim) = self.game.get_at(to).unwrap();
        //                     Move::Capture { from, to, pc: Knight, victim }
        //                 })
        //             },
        //             MoveGenType::Quiets => {
        //                 let quiets   = *ms & !occ;
        //                 quiets.into_iter().map(move |to| {
        //                     Move::Quiet { from, to, pc: Knight }
        //                 })
        //             },
        //         }
        //     })
        // }

    }

    /// Sliding
    impl<'a> MoveGen<'a> {

        pub fn gen_sliding(&mut self, gen: MoveGenType, pc: Piece) {
            let pieces = self.game.get(pc, self.side);

            match gen {
                MoveGenType::Captures => {
                    for sq in pieces.into_iter() {
                        let moves   = self._gen_sliding_single(pc, sq.into(), None);
                        let attacks = moves & self.game.get_color(!self.side);
                        attacks.into_iter().for_each(|to| {
                            let (_,victim) = self.game.get_at(to).unwrap();
                            let mv = Move::Capture { from: sq, to, pc, victim };
                            self.buf.push(mv);
                        });
                    }
                },
                MoveGenType::Quiets => {
                    for sq in pieces.into_iter() {
                        let moves   = self._gen_sliding_single(pc, sq.into(), None);
                        let quiets  = moves & self.game.all_empty();
                        quiets.into_iter().for_each(|sq2| {
                            let mv = Move::Quiet { from: sq, to: sq2, pc };
                            self.buf.push(mv);
                        });
                    }
                },
                MoveGenType::Evasions    => {
                    self.gen_sliding(MoveGenType::Captures, pc);
                    self.gen_sliding(MoveGenType::Quiets, pc);
                },
                MoveGenType::QuietChecks => unimplemented!(),
            }


        }

        pub fn _gen_sliding_single(
            &self,
            pc:     Piece,
            c0:     Coord,
            occ:    Option<BitBoard>,
        ) -> BitBoard {
            let occ = match occ {
                None    => self.game.all_occupied(),
                Some(b) => b,
            };
            let moves = match pc {
                Rook   => self.ts.attacks_rook(c0, occ),
                Bishop => self.ts.attacks_bishop(c0, occ),
                Queen  => self.ts.attacks_bishop(c0, occ) | self.ts.attacks_rook(c0, occ),
                _      => panic!("search sliding: {:?}", pc),
            };
            moves & !self.game.get_color(self.side)
        }

    }

    /// King, Castling
    impl<'a> MoveGen<'a> {

        pub fn gen_castles(&mut self) {
            if self.game.state.checkers.is_not_empty() {
                return;
            }

            let (kingside,queenside) = self.game.state.castling.get_color(self.side);

            let ksq: Coord = self.game.get(King, self.side).bitscan().into();

            if (self.side == White && ksq != Sq::E1.to()) || (self.side == Black && ksq != Sq::E8.to()) {
                return;
            }

            if kingside {
                if let mv@Move::Castle { from, to, rook_from, rook_to } = Move::CASTLE_KINGSIDE[self.side] {
                    if self.game.get(Rook,self.side).is_one_at(rook_from) {
                        let between = self.ts.between_exclusive(from, rook_from);
                        if (between & self.game.all_occupied()).is_empty() {
                            if !between.into_iter().any(
                                |sq| self.game.find_attacks_by_side(self.ts, sq, !self.side, true)) {
                                self.buf.push(mv);
                            }
                        }
                    }
                }
            }

            if queenside {
                if let mv@Move::Castle { from, to, rook_from, rook_to } = Move::CASTLE_QUEENSIDE[self.side] {
                    if self.game.get(Rook,self.side).is_one_at(rook_from) {
                        let between_blocks  = self.ts.between_exclusive(from, rook_from);
                        let between_attacks = Move::CASTLE_QUEENSIDE_BETWEEN[self.side];
                        if (between_blocks & self.game.all_occupied()).is_empty() {
                            if !between_attacks.into_iter().any(
                                |sq| self.game.find_attacks_by_side(self.ts, sq, !self.side, true)) {
                                self.buf.push(mv);
                            }
                        }
                    }
                }
            }

        }

        pub fn gen_king(&mut self, gen: MoveGenType) {
            let from = self.game.get(King, self.side).bitscan();

            let moves = self.ts.get_king(from);

            let occ = self.game.all_occupied();

            match gen {
                MoveGenType::Captures => {
                    let captures = moves & self.game.get_color(!self.side);
                    captures.into_iter().for_each(|to| {
                        let (_,victim) = self.game.get_at(to).unwrap();
                        let mv = Move::Capture { from, to, pc: King, victim };
                        self.buf.push(mv);
                    });
                },
                MoveGenType::Quiets => {
                    let captures = moves & self.game.get_color(!self.side);
                    let quiets   = moves & !occ;
                    quiets.into_iter().for_each(|to| {
                        let mv = Move::Quiet { from, to, pc: King };
                        self.buf.push(mv);
                    });
                },
                MoveGenType::Evasions    => {

                    self.gen_king(MoveGenType::Captures);
                    self.gen_king(MoveGenType::Quiets);

                    // let captures = moves & self.game.get_color(!self.side);
                    // captures.into_iter().for_each(|to| {
                    //     if !self.game.find_attacks_by_side(self.ts, to, !self.side, true) {
                    //         let (_,victim) = self.game.get_at(to).unwrap();
                    //         let mv = Move::Capture { from, to, pc: King, victim };
                    //         self.buf.push(mv);
                    //     }
                    // });
                    // let quiets   = moves & !occ;
                    // quiets.into_iter().for_each(|to| {
                    //     if !self.game.find_attacks_by_side(self.ts, to, !self.side, true) {
                    //         let mv = Move::Quiet { from, to, pc: King };
                    //         self.buf.push(mv);
                    //     }
                    // });

                },
                MoveGenType::QuietChecks => unimplemented!(),
            }
        }
    }

}

