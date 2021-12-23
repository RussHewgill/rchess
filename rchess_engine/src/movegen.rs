
use std::collections::HashMap;

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

    QSearchHash,
    QSearchInit,
    QSearch,

    // QSearchRecaps,

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

            QSearchHash  => Some(QSearchInit),
            QSearchInit  => Some(QSearch),
            QSearch      => Some(Finished),

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

    pub see_hashes:      HashMap<Move,Score>,

    in_check:            bool,
    side:                Color,

    stage:               MoveGenStage,
    buf:                 ArrayVec<Move,256>,
    // buf:                 Vec<Move>,

    pub hashmove:        Option<Move>,
    pub counter_move:    Option<Move>,

    depth:               Depth,
    ply:                 Depth,
}

/// Sort
impl<'a> MoveGen<'a> {

    #[cfg(feature = "nope")]
    pub fn sort(&mut self, st: &ABStack) {}

    // #[cfg(feature = "nope")]
    pub fn sort(&mut self, st: &ABStack) {
        let mut buf = &mut self.buf;
        let ts      = self.ts;
        let g       = self.game;
        let ply     = self.ply;
        let stage   = self.stage;
        let mut see = &mut self.see_hashes;

        #[cfg(feature = "killer_moves")]
        let killers = match st.stacks.get(ply as usize) {
            Some(ks) => (ks.killers[0],ks.killers[1]),
            _        => (None,None)
        };
        // let killers = st.killers.get(g.state.side_to_move,ply);
        #[cfg(not(feature = "killer_moves"))]
        let killers = (None,None);

        buf.sort_by_cached_key(|&mv| {
            // std::cmp::Reverse(crate::move_ordering::score_move_for_sort(ts, g, st, ply, mv, killers))
            score_move_for_sort(ts, g, see, stage, st, ply, mv, killers)
        });
        buf.reverse();

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
            see_hashes:  HashMap::default(),
            in_check,
            side,
            stage:     if in_check { MoveGenStage::EvasionHash } else { MoveGenStage::Hash },
            buf:       ArrayVec::new(),
            // buf:       Vec::with_capacity(128),

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

    pub fn new_qsearch(
        ts:             &'static Tables,
        game:           &'a Game,
        hashmove:       Option<Move>,
        ply:            Depth,
    ) -> Self {
        let in_check = game.state.checkers.is_not_empty();
        let side = game.state.side_to_move;

        let mut out = Self {
            ts,
            game,
            see_hashes:  HashMap::default(),
            in_check,
            side,
            stage:     if in_check { MoveGenStage::EvasionHash } else { MoveGenStage::QSearchHash },
            // stage:     MoveGenStage::Finished,
            buf:       ArrayVec::new(),
            // buf:       Vec::with_capacity(128),

            hashmove,
            counter_move: None,

            depth: 0,
            ply,
        };

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
        use MoveGenStage::*;
        match self.stage {

            Hash     => {
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
            CounterMove => {
                if let Some(mv) = self.counter_move {
                    if self.move_is_legal(mv) {
                        self.stage = self.stage.next()?;
                        return Some(mv);
                    }
                }
                self.stage = self.stage.next()?;
                self.next(stack)
            },

            CapturesInit => {
                self.generate(MoveGenType::Captures);
                // TODO: sort here
                self.sort(stack);

                self.stage = self.stage.next()?;
                self.next(stack)
            },
            Captures => {
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
            QuietsInit => {
                self.generate(MoveGenType::Quiets);
                // TODO: sort here?
                self.sort(stack);

                self.stage = self.stage.next()?;
                self.next(stack)
            },
            Quiets => {
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

            EvasionHash => {
                if let Some(mv) = self.hashmove {
                    if self.move_is_legal(mv) {
                        self.stage = self.stage.next()?;
                        return Some(mv);
                    }
                }

                self.stage = self.stage.next()?;
                self.next(stack)
            },
            EvasionInit => {
                self.generate(MoveGenType::Evasions);
                self.sort(stack);

                self.stage = self.stage.next()?;
                self.next(stack)
            },
            Evasion => {
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

            QSearchHash => {
                if let Some(mv) = self.hashmove {
                    if self.move_is_legal(mv) {
                        self.stage = self.stage.next()?;
                        return Some(mv);
                    }
                }
                self.stage = self.stage.next()?;
                self.next(stack)
            },
            QSearchInit => {

                if self.ply > QS_RECAPS_ONLY && !self.in_check {
                    if let Some(prev) = self.game.state.last_capture {
                        self.generate(MoveGenType::Captures);
                        self.buf.retain(|mv| mv.sq_to() == prev);

                        // assert!(self.buf.len() <= 1);
                        self.sort(stack);
                        self.stage = self.stage.next()?;
                        return self.next(stack);
                    } else {
                        assert_eq!(self.buf.len(), 0);
                        self.stage = self.stage.next()?;
                        return self.next(stack);
                    }
                }

                self.generate(MoveGenType::Captures);
                self.sort(stack);

                self.stage = self.stage.next()?;
                self.next(stack)
            },
            QSearch => {
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

            GenAll     => self.next_all(stack),
            GenAllInit => self.next_all(stack),

            Finished => {
            // _ => {
                None
            },
        }
    }

}

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
        self.gen_pawns(gen, None);
        self.gen_knights(gen, None);
        self.gen_sliding(gen, Bishop, None);
        self.gen_sliding(gen, Rook, None);
        self.gen_sliding(gen, Queen, None);
        self.gen_king(gen, None);
        if gen == MoveGenType::Quiets {
            self.gen_castles();
        }
    }

    #[allow(unused_doc_comments)]
    fn _gen_all_in_check(&mut self, gen: MoveGenType) {

        let num_checkers = self.game.state.checkers.popcount();
        if num_checkers == 0 {
            panic!();
        } else if num_checkers == 1 {
            /// in check, must block or capture attacker

            let target = match gen {
                MoveGenType::Captures                       => Some(self.game.state.checkers),
                MoveGenType::Quiets                         => {
                    let ksq     = self.game.get(King, self.side).bitscan();
                    let between = self.ts.between(ksq, self.game.state.checkers.bitscan());
                    Some(between & self.game.all_empty())
                },
                MoveGenType::Evasions                       => {
                    let ksq     = self.game.get(King, self.side).bitscan();
                    let between = self.ts.between(ksq, self.game.state.checkers.bitscan());
                    Some(between | self.game.state.checkers)
                },
                // MoveGenType::QuietChecks                    => unimplemented!(),
                _                                        => None,
            };

            // let target = None;

            // let target = match gen {
            //     MoveGenType::Captures    => target & self.game.get_color(!self.side),
            //     MoveGenType::Quiets      => target & self.game.all_occupied(),
            // }

            self.gen_king(gen, target);
            self.gen_pawns(gen, target);
            self.gen_knights(gen, target);
            self.gen_sliding(gen, Bishop, target);
            self.gen_sliding(gen, Rook, target);
            self.gen_sliding(gen, Queen, target);

        } else {
            // double check, only generate king moves

            self.gen_king(gen, None);

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

/// SEE
impl<'a> MoveGen<'a> {

    // pub fn _static_exchange(
    //     ts: &'static Tables,
    //     g: &Game,
    //     mut map: &mut HashMap<Move,Score>,
    //     mv: Move
    // ) -> Option<Score> {
    //     if !mv.filter_all_captures() { return None; }
    //     if let Some(score) = map.get(&mv) {
    //         Some(*score)
    //     } else {
    //         if let Some(score) = g.static_exchange(ts, mv) {
    //             map.insert(mv, score);
    //             Some(score)
    //         } else {
    //             None
    //         }
    //     }
    // }

    pub fn static_exchange(&mut self, mv: Move) -> Option<Score> {
        // Self::_static_exchange(self.ts, self.game, &mut self.see_hashes, mv)
        self.game.static_exchange(self.ts, mv)
    }
}

/// Misc helpers
impl<'a> MoveGen<'a> {

    pub fn gives_check(ts: &Tables, g: &Game, mv: Move) -> bool {

        let ksq_enemy = g.get(King, !g.state.side_to_move).bitscan();

        /// Castle
        if let Move::Castle { rook_to, .. } = mv {
            return g.state.check_squares[Rook].is_one_at(rook_to);
        }

        /// Promotion
        if let Some(new_pc) = mv.new_piece() {
            return g.state.check_squares[new_pc].is_one_at(mv.sq_to());
        }

        if let Some(pc) = mv.piece() {
            if g.state.check_squares[pc].is_one_at(mv.sq_to()) {
                return true;
            }
        }

        /// revealed check
        if g.get_pins(!g.state.side_to_move).is_one_at(mv.sq_from()) {
            if !ts.aligned(mv.sq_from(), mv.sq_to(), ksq_enemy) {
                return true;
            }
        }

        /// En Passant
        if let Move::EnPassant { from, to, capture } = mv {
            // let b = g.all_occupied() ^ BitBoard::single(from) ^ BitBoard::single(capture);
            // let b = b | BitBoard::single(to);
            return g.get_pins(!g.state.side_to_move).is_one_at(capture);
        }

        false
    }

}

/// Check Pseudo
impl<'a> MoveGen<'a> {
    pub fn move_is_pseudo_legal(&self, mv: Move) -> bool {

        if let Some((side,pc)) = self.game.get_at(mv.sq_from()) {
            if self.side != side || mv.piece() != Some(pc) {
                return false;
            }
        }

        unimplemented!()
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
                    || self.ts.aligned(
                        mv.sq_from(), mv.sq_to(), self.game.get(King, self.side).bitscan().into());

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
        pub fn gen_pawns(&mut self, gen: MoveGenType, target: Option<BitBoard>) {

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

                    let enemies = self.game.get_color(!self.side);

                    let mut bw = ps.shift_dir(dw) & enemies;
                    let mut be = ps.shift_dir(de) & enemies;

                    if let Some(tgt) = target {
                        bw &= tgt;
                        be &= tgt;
                    }

                    for to in bw.into_iter() {
                        let (_,victim) = self.game.get_at(to).unwrap();
                        if let Some(from) = (!dw).shift_coord(to) {
                            let mv = Move::Capture { from, to, pc: Pawn, victim };
                            self.buf.push(mv);
                        }
                    }

                    for to in be.into_iter() {
                        let (_,victim) = self.game.get_at(to).unwrap();
                        if let Some(from) = (!de).shift_coord(to) {
                            let mv = Move::Capture { from, to, pc: Pawn, victim };
                            self.buf.push(mv);
                        }
                    }

                    // for from in ps.into_iter() {
                    //     let bb = BitBoard::single(from);
                    //     let mut cs = (bb.shift_dir(dw) & self.game.get_color(!self.side))
                    //         | (bb.shift_dir(de) & self.game.get_color(!self.side));
                    //     if let Some(tgt) = target { cs &= tgt; }
                    //     for to in cs.into_iter() {
                    //         let (_,victim) = self.game.get_at(to).unwrap();
                    //         let mv = Move::Capture { from, to, pc: Pawn, victim };
                    //         self.buf.push(mv);
                    //     }
                    // }

                    if let Some(ep) = self.game.state.en_passant {
                        let attacks = self.ts.get_pawn(ep).get_capture(!self.side);
                        let attacks = attacks & ps;
                        attacks.into_iter().for_each(|sq| {
                            let capture =
                                if self.side == White { S.shift_coord(ep) } else { N.shift_coord(ep) };
                            let capture = capture
                                .unwrap_or_else(
                                    || panic!("en passant bug? ep: {:?}, capture: {:?}", ep, capture));
                            let mv = Move::EnPassant { from: sq.into(), to: ep, capture };
                            self.buf.push(mv);
                        });
                    }

                    self.gen_promotions(gen, target);
                },
                MoveGenType::Quiets => {
                    let pushes = ps.shift_dir(dir);
                    let mut pushes = pushes & !(occ);
                    if let Some(tgt) = target { pushes &= tgt; }

                    let doubles = ps & BitBoard::mask_rank(if self.side == White { 1 } else { 6 });
                    let doubles = doubles.shift_mult(dir, 2);
                    let mut doubles = doubles & !(occ) & (!(occ)).shift_dir(dir);
                    if let Some(tgt) = target { doubles &= tgt; }

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

                    self.gen_promotions(gen, target);
                },
                MoveGenType::Evasions    => {
                    self.gen_pawns(MoveGenType::Captures, target);
                    self.gen_pawns(MoveGenType::Quiets, target);
                },
                MoveGenType::QuietChecks => unimplemented!(),
            }
        }

        pub fn gen_promotions(&mut self, gen: MoveGenType, target: Option<BitBoard>) {

            let rank7 = if self.side == White { BitBoard::mask_rank(6) } else { BitBoard::mask_rank(1) };

            let ps = self.game.get(Pawn, self.side);
            let ps = ps & rank7;

            let occ = self.game.all_occupied();
            let (dir,dw,de) = match self.side {
                White => (N,NW,NE),
                Black => (S,SW,SE),
            };

            let pushes = ps.shift_dir(dir);
            let mut pushes = pushes & !(occ);
            if let Some(tgt) = target { pushes &= tgt; }

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
                if let Some(tgt) = target { cs &= tgt; }

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

        pub fn gen_knights(&mut self, gen: MoveGenType, target: Option<BitBoard>) {
            let occ = self.game.all_occupied();
            let ks = self.game.get(Knight, self.side);

            match gen {
                MoveGenType::Captures => {
                    ks.into_iter().for_each(|from| {
                        let ms = self.ts.get_knight(from);
                        let mut captures = ms & self.game.get_color(!self.side);
                        if let Some(tgt) = target { captures &= tgt; }
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
                        let mut quiets = ms & !occ;
                        if let Some(tgt) = target { quiets &= tgt; }
                        quiets.into_iter().for_each(|to| {
                            let mv = Move::Quiet { from, to, pc: Knight };
                            self.buf.push(mv);
                        });
                    });
                },
                MoveGenType::Evasions    => {
                    self.gen_knights(MoveGenType::Captures, target);
                    self.gen_knights(MoveGenType::Quiets, target);
                },
                MoveGenType::QuietChecks => unimplemented!(),
            }

        }

    }

    /// Sliding
    impl<'a> MoveGen<'a> {

        pub fn gen_sliding(&mut self, gen: MoveGenType, pc: Piece, target: Option<BitBoard>) {
            let pieces = self.game.get(pc, self.side);

            match gen {
                MoveGenType::Captures => {
                    for sq in pieces.into_iter() {
                        let moves   = self._gen_sliding_single(pc, sq.into(), None);
                        let mut captures = moves & self.game.get_color(!self.side);
                        if let Some(tgt) = target { captures &= tgt; }
                        captures.into_iter().for_each(|to| {
                            let (_,victim) = self.game.get_at(to).unwrap();
                            let mv = Move::Capture { from: sq, to, pc, victim };
                            self.buf.push(mv);
                        });
                    }
                },
                MoveGenType::Quiets => {
                    for sq in pieces.into_iter() {
                        let moves   = self._gen_sliding_single(pc, sq.into(), None);
                        let mut quiets  = moves & self.game.all_empty();
                        if let Some(tgt) = target { quiets &= tgt; }
                        quiets.into_iter().for_each(|sq2| {
                            let mv = Move::Quiet { from: sq, to: sq2, pc };
                            self.buf.push(mv);
                        });
                    }
                },
                MoveGenType::Evasions    => {
                    self.gen_sliding(MoveGenType::Captures, pc, target);
                    self.gen_sliding(MoveGenType::Quiets, pc, target);
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
            if self.in_check {
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

        pub fn gen_king(&mut self, gen: MoveGenType, target: Option<BitBoard>) {
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

                    self.gen_king(MoveGenType::Captures, target);
                    self.gen_king(MoveGenType::Quiets, target);

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

