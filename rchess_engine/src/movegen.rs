
pub use self::pieces::*;
use crate::explore::ABStack;
use crate::move_ordering::OrdMove;
use crate::types::*;
use crate::tables::*;
use crate::move_ordering::*;

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use arrayvec::ArrayVec;
use itertools::Itertools;

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
    // CounterMove,

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
    QChecksInit,
    QChecks,

    // QSearchRecaps,

    // GenAllInit,
    // GenAll,

    RootMoves,

    Finished,
}

impl MoveGenStage {
    pub fn next(self) -> Option<Self> {
        use MoveGenStage::*;
        match self {
            Hash         => Some(CapturesInit),
            // CounterMove  => Some(CapturesInit),

            CapturesInit => Some(Captures),
            Captures     => Some(QuietsInit),
            QuietsInit   => Some(Quiets),
            Quiets       => Some(Finished),

            EvasionHash  => Some(EvasionInit),
            EvasionInit  => Some(Evasion),
            Evasion      => Some(Finished),

            QSearchHash  => Some(QSearchInit),
            QSearchInit  => Some(QSearch),
            QSearch      => Some(QChecksInit),
            QChecksInit  => Some(QChecks),
            QChecks      => Some(Finished),

            // GenAllInit   => Some(GenAll),
            // GenAll       => None,

            RootMoves    => Some(Finished),

            Finished     => None,
        }
    }
}

#[cfg(feature = "nope")]
mod mg_key {
    #[derive(Debug,Eq,Clone,Copy)]
    pub struct MGKey(usize, OrdMove);

    impl Ord for MGKey {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.1.cmp(&other.1)
        }
    }

    impl PartialOrd for MGKey {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            self.1.partial_cmp(&other.1)
        }
    }

    impl PartialEq for MGKey {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
                && self.1 == other.1
        }
    }

}

#[derive(Debug,Clone)]
pub struct MoveGen<'a> {
    // ts:                  &'static Tables,
    ts:                  &'a Tables,
    game:                &'a Game,

    // root_moves:          Option<Vec<Move>>,

    pub see_map:         HashMap<Move,Score>,

    in_check:            bool,
    side:                Color,

    pub skip_quiets:     bool,

    stage:               MoveGenStage,

    buf:                 ArrayVec<Move,256>,
    // pub buf:                 ArrayVec<Move,256>,

    // buf_scored:          ArrayVec<(Move,OrdMove),256>,
    buf_scored:          ArrayVec<(Move,Score),256>,
    // pub buf_scored:          ArrayVec<(Move,Score),256>,

    // buf_set:             BinaryHeap<MGKey>,

    cur:                 usize,

    // pub move_history:    Vec<(Zobrist, Move)>,

    pub hashmove:        Option<Move>,
    pub counter_move:    Option<Move>,
    // pub killer_moves:    ArrayVec<Move,2>,
    pub killer_moves:    (Option<Move>,Option<Move>),

    depth:               Depth,
    ply:                 Depth,
}

/// Sort and pick, but incremental
#[cfg(feature = "nope")]
impl<'a> MoveGen<'a> {

    pub fn sort(&mut self, st: &ABStack, gen_type: MoveGenType) {
        let mut see_map = &mut self.see_map;

        // #[cfg(feature = "killer_moves")]
        // if gen_type != MoveGenType::Captures {
        //     let (k1,k2) = (self.killer_moves.0,self.killer_moves.1);
        //     self.buf.retain(|mv| Some(*mv) != k1 && Some(*mv) != k2);
        // }

        for mv in self.buf.drain(..) {
            let score = score_move_for_sort4(
                self.ts,
                self.game,
                gen_type,
                see_map,
                st,
                self.ply,
                mv,
                self.killer_moves,
                self.counter_move);

            self.buf_scored.push((mv, score));
        }

    }

    pub fn _pick(&mut self, st: &ABStack, best: bool) -> Option<Move> {

        if self.buf_scored.len() == 0 { return None; }

        if !best { return Some(self.buf_scored.pop()?.0); }

        let mut best_score_idx = self.cur;
        for k in self.cur+1 .. self.buf_scored.len() {
            if self.buf_scored[k].1 > self.buf_scored[best_score_idx].1 {
                best_score_idx = k;
            }
        }

        if best_score_idx != self.cur {
            self.buf_scored.swap(self.cur, best_score_idx);
        }

        self.cur += 1;

        Some(self.buf_scored.get(self.cur - 1)?.0)
    }

}

/// Score, Sort, Pick best
// #[cfg(feature = "nope")]
impl<'a> MoveGen<'a> {

    // #[cfg(feature = "nope")]
    pub fn sort(&mut self, st: &ABStack, gen_type: MoveGenType) {
        let mut see_map = &mut self.see_map;

        #[cfg(feature = "killer_moves")]
        if gen_type != MoveGenType::Captures {
            let (k1,k2) = (self.killer_moves.0,self.killer_moves.1);
            self.buf.retain(|mv| Some(*mv) != k1 && Some(*mv) != k2);
        }

        // let score = score_move_for_sort(
        //     self.ts,
        //     self.game,
        //     see_map,
        //     self.stage,
        //     gen_type,
        //     st,
        //     self.ply,
        //     mv,
        //     killers,
        //     self.counter_move);

        // for mv in self.buf.iter() {
        //     if let Some(victim) = mv.victim() {
        //         if victim == King {
        //             eprintln!("MoveGen: generated move that captures king?\n{:?}\n{:?}\nmv = {:?}",
        //                    self.game.to_fen(),
        //                    self.game,
        //                    mv,
        //             );
        //             for (n,(_,mv)) in st.move_history.iter().enumerate() {
        //                 eprintln!("mv {:>2} = {:?}", n, mv);
        //             }
        //             panic!();
        //         }
        //     }
        // }

        for mv in self.buf.drain(..) {

            // let score = score_move_for_sort3(
            //     self.ts,
            //     self.game,
            //     see_map,
            //     st,
            //     self.ply,
            //     mv,
            //     self.killer_moves,
            //     self.counter_move);

            let score = score_move_for_sort4(
                self.ts,
                self.game,
                gen_type,
                see_map,
                st,
                self.ply,
                mv,
                self.killer_moves,
                self.counter_move);

            self.buf_scored.push((mv, score));
        }

        // crate::move_ordering::selection_sort(&mut self.buf_scored);
        self.buf_scored.sort_unstable_by_key(|x| x.1);
        // self.buf_scored.sort_by_key(|x| x.1);

        // self.buf_scored.reverse();

    }

    // #[cfg(feature = "nope")]
    pub fn _pick(&mut self, st: &ABStack, best: bool) -> Option<Move> {
        let (mv,_) = self.buf_scored.pop()?;
        Some(mv)
    }

}

/// Pick best or first
// #[cfg(feature = "nope")]
impl<'a> MoveGen<'a> {

    pub fn pick_next(&mut self, st: &ABStack) -> Option<Move> {
        self._pick(st, false)
    }

    pub fn pick_best(&mut self, st: &ABStack) -> Option<Move> {
        self._pick(st, true)
    }

    #[cfg(feature = "nope")]
    pub fn _pick(&mut self, st: &ABStack, best: bool) -> Option<Move> {

        let len = self.buf_scored.len();

        // let mut cur = 0;
        self.cur = 0;
        while self.cur < len {

            if best {
                self.buf_scored.swap(self.cur, len - 1);
            }

            let (mv,score) = self.buf_scored[self.cur];
            // if Some(mv) != self.hashmove && filter 
            if Some(mv) != self.hashmove {
                self.buf_scored.remove(self.cur);
                return Some(mv);
            }

            self.cur += 1;
        }

        None
    }

    #[cfg(feature = "nope")]
    pub fn _pick(&mut self, st: &ABStack, best: bool) -> Option<Move> {
        if !best { return Some(self.buf_scored.pop()?.0); }

        if self.cur == self.buf_scored.len() { return None; }

        let mut kmin = self.cur;
        for k in self.cur+1 .. self.buf_scored.len() {
            if self.buf_scored[k].1 > self.buf_scored[kmin].1 {
                kmin = k;
            }
        }

        if kmin != self.cur {
            self.buf_scored.swap(self.cur, kmin);
        }

        self.cur += 1;

        Some(self.buf_scored.get(self.cur - 1)?.0)
    }

    #[cfg(feature = "nope")]
    pub fn _pick(&mut self, st: &ABStack, best: bool) -> Option<Move> {

        // eprintln!("best = {:?}", best);
        // eprintln!("self.buf_scored.len() = {:?}", self.buf_scored.len());

        // if !best {
        //     return Some(self.buf_scored.pop()?.0);
        //     // let mv = self.buf_scored.get(self.cur)?;
        //     // self.cur += 1;
        //     // return Some(mv.0);
        // }

        if self.buf_scored.is_empty() { return None; }

        let mut kmax = 0;
        let mut best_score = self.buf_scored[0].1;
        for (k,(mv,score)) in self.buf_scored.iter().enumerate() {

            if *score < best_score {
                kmax = k;
                best_score = *score;
            }

        }

        if kmax != 0 {
            self.buf_scored.swap(0, kmax);
        }

        let mv = self.buf_scored.pop()?.0;

        #[cfg(feature = "nope")]
        while self.cur < self.buf_scored.len() {

            // let max = self.buf_scored[self.cur..].iter().position_max_by_key(|x| x.1)?;

            let mut kmax = self.cur;

            for k in self.cur..self.buf_scored.len() {
                if self.buf_scored[k] < self.buf_scored[kmax] {
                    kmax = k;
                }
            }

            // self.buf_scored.swap(self.cur, self.cur + max);
            self.buf_scored.swap(self.cur, kmax);

            let mv = self.buf_scored.get(self.cur)?;

            self.cur += 1;

            return Some(mv.0);
        }

        Some(mv)

        // None
    }
}

/// New
impl<'a> MoveGen<'a> {

    pub fn new(
        // ts:             &'static Tables,
        ts:             &'a Tables,
        game:           &'a Game,
        hashmove:       Option<Move>,
        stack:          &ABStack,
        depth:          Depth,
        ply:            Depth,
        // move_history:   Vec<(Zobrist,Move)>,
    ) -> Self {
        let in_check = game.state.checkers.is_not_empty();
        let side = game.state.side_to_move;

        // let counter_move = None;
        let counter_move = if let Some(prev_mv) = game.last_move {
            stack.counter_moves.get_counter_move(prev_mv, game.state.side_to_move)
            // stack.counter_moves.get_counter_move(prev_mv)
        } else { None };

        let killer_moves = stack.killers_get(ply);
        // let killer_moves = (None,None);

        // let killer_moves = {
        //     let (k1,k2) = stack.killers_get(ply);
        //     let k1 = k1.map(|mv| )
        // };

        let mut out = Self {
            ts,
            game,
            // root_moves: None,
            see_map:  HashMap::default(),
            in_check,
            side,
            skip_quiets: false,
            stage:     if in_check { MoveGenStage::EvasionHash } else { MoveGenStage::Hash },
            buf:       ArrayVec::new(),

            buf_scored: ArrayVec::new(),

            cur:   0,
            // move_history,

            hashmove,
            counter_move,
            // killer_moves:   ArrayVec::new(),
            killer_moves,

            depth,
            ply,
        };
        // XXX: check for legal move incase of corrupted TTEntry
        // let hashmove = if let Some (mv) = hashmove {
        //     if out.move_is_legal(mv) { Some(mv) } else { None }
        // } else { None };
        // out.hashmove = hashmove;
        out
    }

    pub fn new_qsearch(
        // ts:             &'static Tables,
        ts:             &'a Tables,
        game:           &'a Game,
        hashmove:       Option<Move>,
        stack:          &ABStack,
        ply:            Depth,
        // move_history:   Vec<(Zobrist,Move)>,
    ) -> Self {
        let in_check = game.state.checkers.is_not_empty();
        let side = game.state.side_to_move;

        let counter_move = if let Some(prev_mv) = game.last_move {
            stack.counter_moves.get_counter_move(prev_mv, game.state.side_to_move)
            // stack.counter_moves.get_counter_move(prev_mv)
        } else { None };

        let mut out = Self {
            ts,
            game,
            // root_moves: None,
            see_map:  HashMap::default(),
            in_check,
            side,
            skip_quiets: false,
            stage:     if in_check { MoveGenStage::EvasionHash } else { MoveGenStage::QSearchHash },
            // stage:     MoveGenStage::Finished,
            buf:       ArrayVec::new(),

            buf_scored: ArrayVec::new(),

            cur:   0,
            // move_history,

            hashmove,
            counter_move,
            // killer_moves:   ArrayVec::new(),
            killer_moves:   (None,None),

            depth: 0,
            ply,
        };

        out
    }

}

/// Gen All
impl<'a> MoveGen<'a> {

    pub fn new_root(
        // ts:             &'static Tables,
        ts:             &'a Tables,
        game:           &'a Game,
        root_moves:     &[Move],
    ) -> Self {

        let in_check = game.state.checkers.is_not_empty();
        let side = game.state.side_to_move;

        let mut buf = ArrayVec::new();
        // buf.copy_from_slice(root_moves);
        buf.try_extend_from_slice(root_moves).unwrap();

        let mut out = Self {
            ts,
            game,
            // root_moves: Some(root_moves),
            see_map:  HashMap::default(),
            in_check,
            side,
            skip_quiets: false,

            stage:      MoveGenStage::RootMoves,
            // stage:     if in_check { MoveGenStage::EvasionHash } else { MoveGenStage::Hash },
            buf,
            buf_scored: ArrayVec::new(),

            cur:   0,

            hashmove:     None,
            counter_move: None,
            // killer_moves:   ArrayVec::new(),
            killer_moves:   (None,None),

            depth: 0,
            ply: 0,

        };
        out
    }

    // pub fn gen_all(ts: &'static Tables, g: &'a Game) -> Vec<Move> {
    pub fn gen_all(ts: &'a Tables, g: &'a Game) -> Vec<Move> {
        let st = ABStack::new();
        let mut movegen = Self::new(ts, g, None, &st, 0, 0);
        let mut mvs = vec![];
        while let Some(mv) = movegen.next(&st) {
            mvs.push(mv);
        }
        mvs
    }

}

/// Move queries
impl<'a> MoveGen<'a> {

    pub fn is_killer(&self, stack: &ABStack, mv: Move) -> bool {
        if let Some(ks) = stack.get_with(self.ply, |st| st.killers) {
            Some(mv) == ks[0] || Some(mv) == ks[1]
        } else {
            false
        }
    }

    pub fn is_counter(&self, stack: &ABStack, mv: Move) -> bool {
        Some(mv) == self.counter_move
    }

}

/// Not quite Iter
impl<'a> MoveGen<'a> {

    pub fn next(&mut self, stack: &ABStack) -> Option<Move> {
        use MoveGenStage::*;
        match self.stage {

            Hash     => {
                if let Some(mv) = self.hashmove {
                    if self.move_is_legal(mv) && self.move_is_pseudo_legal(mv) {
                        // println!("returning hashmove: {:?}: {:?}", self.game.to_fen(), mv);
                        self.stage = self.stage.next()?;
                        return Some(mv);
                    }
                }
                self.stage = self.stage.next()?;
                self.next(stack)
            },

            CapturesInit => {
                self.generate(MoveGenType::Captures);

                /// Killer moves
                // self.buf.retain(|mv| !ks.contains(mv)); // killer moves are only quiets
                for &mv in [self.killer_moves.0,self.killer_moves.1].iter().flatten() {
                    if self.move_is_pseudo_legal(mv) && self.move_is_legal(mv) {
                        self.buf.push(mv);
                    } else {
                        // debug!("MoveGen: non legal killer move: {:?}", mv);
                    }
                }

                /// Sort
                self.sort(stack, MoveGenType::Captures);

                self.stage = self.stage.next()?;
                self.next(stack)
            },
            Captures => {
                // if let Some(mv) = self.buf.pop() {
                if let Some(mv) = self.pick_best(stack) {
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
                if !self.skip_quiets {
                    self.generate(MoveGenType::Quiets);

                    self.sort(stack, MoveGenType::Quiets);
                }

                self.stage = self.stage.next()?;
                self.next(stack)
            },
            Quiets => {
                if self.skip_quiets {
                    self.stage = self.stage.next()?;
                    return None;
                }
                if let Some(mv) = self.pick_next(stack) {
                // if let Some(mv) = self.buf.pop() {
                    // #[cfg(feature = "killer_moves")]
                    // if Some(mv) == self.killer_moves.0 || Some(mv) == self.killer_moves.1 {
                    //     return self.next(stack);
                    // } else
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
                    if self.move_is_legal(mv) && self.move_is_pseudo_legal(mv) {
                        self.stage = self.stage.next()?;
                        return Some(mv);
                    }
                }

                self.stage = self.stage.next()?;
                self.next(stack)
            },
            EvasionInit => {
                self.generate(MoveGenType::Evasions);
                self.sort(stack, MoveGenType::Evasions);

                self.stage = self.stage.next()?;
                self.next(stack)
            },
            Evasion => {
                if let Some(mv) = self.pick_best(stack) {
                // if let Some(mv) = self.buf.pop() {
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
                        self.sort(stack, MoveGenType::Captures);
                        self.stage = self.stage.next()?;
                        return self.next(stack);
                    } else {
                        assert_eq!(self.buf.len(), 0);
                        self.stage = self.stage.next()?;
                        return self.next(stack);
                    }
                }

                self.generate(MoveGenType::Captures);
                self.sort(stack, MoveGenType::Captures);

                self.stage = self.stage.next()?;
                self.next(stack)
            },
            QSearch => {
                if let Some(mv) = self.pick_best(stack) {
                // if let Some(mv) = self.buf.pop() {
                    if Some(mv) != self.hashmove && self.move_is_legal(mv) {
                        return Some(mv);
                    } else {
                        return self.next(stack);
                    }
                }
                self.stage = self.stage.next()?;
                self.next(stack)
            },

            QChecksInit => {

                if !self.skip_quiets {
                    self.generate(MoveGenType::QuietChecks);
                    self.sort(stack, MoveGenType::QuietChecks);
                }

                self.stage = self.stage.next()?;
                self.next(stack)
            },
            QChecks => {
                if !self.skip_quiets {
                    if let Some(mv) = self.pick_best(stack) {
                        if self.move_is_legal(mv) {
                            return Some(mv);
                        } else {
                            return self.next(stack);
                        }
                    }
                }
                self.stage = self.stage.next()?;
                self.next(stack)
            },

            // GenAll     => self.next_all(stack),
            // GenAllInit => self.next_all(stack),

            RootMoves => {
                if let Some(mv) = self.buf.pop() {
                    return Some(mv);
                }
                self.stage = self.stage.next()?;
                // None
                unimplemented!()
            },

            Finished => {
            // _ => {
                None
            },
        }
    }

    pub fn next_debug(&mut self, stack: &ABStack) -> Option<(MoveGenStage,Move)> {
        let stage = self.stage;
        let mv = self.next(stack)?;
        Some((stage,mv))
    }

}

/// Generate
impl<'a> MoveGen<'a> {
    // pub fn gen_to_vec(&mut self, gen: MoveGenType)

    // pub fn generate_list(ts: &'static Tables, g: &'a Game, gen: Option<MoveGenType>) -> ArrayVec<Move,256> {
    pub fn generate_list(ts: &'a Tables, g: &'a Game, gen: Option<MoveGenType>) -> ArrayVec<Move,256> {

        let st = ABStack::new();
        let mut movegen = Self::new(ts, g, None, &st, 0, 0);

        movegen._generate_list(gen)
    }

    pub fn _generate_list(&mut self, gen: Option<MoveGenType>) -> ArrayVec<Move,256> {
        if let Some(gen) = gen {
            if self.in_check {
                self._gen_all_in_check(gen);
            } else {
                self._gen_all(gen);
            }
        } else {
            if self.in_check {
                self._gen_all_in_check(MoveGenType::Captures);
                self._gen_all_in_check(MoveGenType::Quiets);
            } else {
                self._gen_all(MoveGenType::Captures);
                self._gen_all(MoveGenType::Quiets);
            }
        }

        let mut moves = self.buf.clone();
        moves.retain(|mv| self.move_is_legal(*mv));

        moves
    }

    pub fn generate(&mut self, gen: MoveGenType) {
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

    // pub fn perft(ts: &'static Tables, g: &'a Game, depth: Depth) -> (u64,Vec<(Move,u64)>) {
    pub fn perft(ts: &'a Tables, g: &'a Game, depth: Depth) -> (u64,Vec<(Move,u64)>) {
        let depth = depth.max(1);
        let mut out = vec![];
        let mut sum = 0;
        let stack = ABStack::new();
        let mut gen = Self::new(ts, &g, None, &stack, depth, 0);

        let moves = Self::generate_list(ts, &g, None);

        for mv in moves {
            if let Ok(g2) = g.make_move_unchecked(&ts, mv) {
                let sum2 = Self::_perft(ts, &stack, g2, depth - 1);
                out.push((mv,sum2));
                sum += sum2;
            }
        }

        // while let Some(mv) = gen.next(&stack) {
        //     if let Ok(g2) = g.make_move_unchecked(&ts, mv) {
        //         let sum2 = Self::_perft(ts, &stack, g2, depth - 1);
        //         out.push((mv,sum2));
        //         sum += sum2;
        //     }
        // }

        // Self::_perft(ts, g, depth)
        (sum,out)
    }

    // pub fn _perft(ts: &'static Tables, st: &ABStack, g: Game, depth: Depth) -> u64 {
    pub fn _perft(ts: &'a Tables, st: &ABStack, g: Game, depth: Depth) -> u64 {
        if depth == 0 { return 1; }

        let mut gen = MoveGen::new(ts, &g, None, st, depth, 0);

        let mut sum = 0;

        let moves = gen._generate_list(None);

        for mv in moves {
            if let Ok(g2) = g.make_move_unchecked(&ts, mv) {
                let sum2 = Self::_perft(ts, st, g2, depth - 1);
                sum += sum2;
            }
        }

        // while let Some(mv) = gen.next(st) {
        //     if let Ok(g2) = g.make_move_unchecked(&ts, mv) {
        //         let sum2 = Self::_perft(ts, st, g2, depth - 1);
        //         sum += sum2;
        //     }
        // }

        sum
    }

}

/// SEE
impl<'a> MoveGen<'a> {

    // // #[cfg(feature = "nope")]
    // pub fn _static_exchange(
    //     ts:          &'static Tables,
    //     g:           &Game,
    //     mut map:     &mut HashMap<Move,Score>,
    //     mv:          Move,
    //     threshold:   Score,
    // ) -> Option<Score> {
    //     g.static_exchange(ts, mv)
    // }

    // #[cfg(feature = "nope")]
    pub fn _static_exchange(
        // ts:          &'static Tables,
        ts:          &'a Tables,
        g:           &Game,
        mut map:     &mut HashMap<Move,Score>,
        mv:          Move,
    ) -> Option<Score> {
        if !mv.filter_all_captures() { return None; }
        if let Some(score) = map.get(&mv) {
            Some(*score)
        } else {
            if let Some(score) = g.static_exchange(ts, mv) {
                map.insert(mv, score);
                Some(score)
            } else {
                None
            }
        }
    }

    // pub fn static_exchange(&mut self, mv: Move, threshold: Score) -> Option<Score> {
    pub fn static_exchange_ge(&mut self, mv: Move, threshold: Score) -> bool {
        // Self::_static_exchange(self.ts, self.game, &mut self.see_map, mv, threshold)

        // if let Some(see) = self.game.static_exchange(self.ts, mv) {
        if let Some(see) = Self::_static_exchange(self.ts, self.game, &mut self.see_map, mv) {
            see >= threshold
        } else {
            false
        }

        // self.game.static_exchange_ge(self.ts, mv, threshold)
    }

}

/// Misc helpers
// #[cfg(feature = "nope")]
impl<'a> MoveGen<'a> {

    pub fn gives_check(&self, mv: Move) -> bool {
        Self::_gives_check(&self.ts, &self.game, mv)
    }

    fn _gives_check(ts: &Tables, g: &Game, mv: Move) -> bool {

        let ksq_enemy = g.get(King, !g.state.side_to_move).bitscan();

        /// Castle
        // if let Move::Castle { rook_to, .. } = mv {
        if let Move::Castle { .. } = mv {
            let (rook_from, rook_to) = mv.castle_rook_mv();
            return g.state.check_squares[Rook].is_one_at(rook_to);
        }

        /// Promotion
        if let Some(new_pc) = mv.new_piece() {
            return g.state.check_squares[new_pc].is_one_at(mv.sq_to());
        }

        if let Some(pc) = mv.piece() {
            if pc != King && g.state.check_squares[pc].is_one_at(mv.sq_to()) {
                return true;
            }
        }

        /// revealed check
        if g.get_pins(!g.state.side_to_move).is_one_at(mv.sq_from()) {
            if !ts.aligned(mv.sq_from(), mv.sq_to(), ksq_enemy) {
                return true;
            }
        }

        // XXX: maybe not complete?
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

    #[cfg(feature = "nope")]
    pub fn move_is_pseudo_legal(&mut self, mv: Move) -> bool {
        true
    }

    // #[cfg(feature = "nope")]
    pub fn move_is_pseudo_legal(&mut self, mv: Move) -> bool {

        if mv == Move::NullMove {
            return true;
        } else if mv.filter_promotion() {
            let mut vs = vec![];
            self._gen_promotions(MoveGenType::Captures, None, Some(&mut vs));
            self._gen_promotions(MoveGenType::Quiets, None, Some(&mut vs));
            return vs.contains(&mv);
        } else if mv.filter_en_passant() {
            let mut vs = vec![];
            // unimplemented!("en passant")
            self.gen_en_passant(None, Some(&mut vs));
            return vs.contains(&mv);
        } else if mv.filter_castle() {
            let mut vs = vec![];
            self._gen_castles(Some(&mut vs));
            return vs.contains(&mv);
        }

        /// From must be occupied by correct side and piece
        match self.game.get_side_at(mv.sq_from()) {
            Some(side) => {
                if self.game.state.side_to_move != side {
                    // trace!("move_is_pseudo_legal: wrong side move: {:?}\n{:?}\n{:?}",
                    //        mv, self.game.to_fen(), self.game);
                    return false;
                } else {
                    if let Some((_,pc)) = self.game.get_at(mv.sq_from()) {
                        if Some(pc) != mv.piece() {
                            return false;
                        }
                    }
                }
            },
            None => {
                // trace!("move_is_pseudo_legal: non legal move, no piece?: {:?}\n{:?}\n{:?}",
                //        mv, self.game.to_fen(), self.game);
                return false;
            }
        }

        /// To must not be occupied by friendly
        /// Must be occupied by enemy if capture
        /// Must be not occupied if quiet
        match self.game.get_side_at(mv.sq_to()) {
            Some(side) => {
                if side == self.game.state.side_to_move {
                    // trace!("move_is_pseudo_legal: self side capture: {:?}\n{:?}\n{:?}",
                    //        mv, self.game.to_fen(), self.game);
                    return false;
                } else if !mv.filter_all_captures() {
                    // trace!("move_is_pseudo_legal: non capture to occupied: {:?}\n{:?}\n{:?}",
                    //        mv, self.game.to_fen(), self.game);
                    return false;
                }
            },
            None => {
                if mv.filter_all_captures() {
                    return false;
                }
            },
        }

        let pc = mv.piece().unwrap();
        if pc == Bishop || pc == Rook || pc == Queen || mv.filter_pawndouble() {
            let between = self.ts.between_exclusive(mv.sq_from(), mv.sq_to());
            if (between & self.game.all_occupied()).is_not_empty() {
                return false;
            }
        }

        true
        // debug!("unhandled move_is_pseudo_legal: \n{:?}\n{:?}\n{:?}",
        //        self.game.to_fen(), self.game, mv);
        // panic!("unhandled move_is_pseudo_legal");
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

        if let Some(King) = mv.victim() {
            return false;
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
                MoveGenType::Captures    => {

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
                            // let mv = Move::Capture { from, to, pc: Pawn, victim };
                            let mv = Move::new_capture(from, to, Pawn, victim);
                            self.buf.push(mv);
                        }
                    }

                    for to in be.into_iter() {
                        let (_,victim) = self.game.get_at(to).unwrap();
                        if let Some(from) = (!de).shift_coord(to) {
                            // let mv = Move::Capture { from, to, pc: Pawn, victim };
                            let mv = Move::new_capture(from, to, Pawn, victim);
                            self.buf.push(mv);
                        }
                    }

                    self.gen_en_passant(target, None);

                    self.gen_promotions(gen, target);
                },
                MoveGenType::Quiets      => {
                    let pushes = ps.shift_dir(dir);
                    let mut pushes = pushes & !(occ);
                    if let Some(tgt) = target { pushes &= tgt; }

                    let doubles = ps & BitBoard::mask_rank(if self.side == White { 1 } else { 6 });
                    let doubles = doubles.shift_mult(dir, 2);
                    let mut doubles = doubles & !(occ) & (!(occ)).shift_dir(dir);
                    if let Some(tgt) = target { doubles &= tgt; }

                    for to in pushes.into_iter() {
                        if let Some(from) = (!dir).shift_coord(to) {
                            let mv = Move::new_quiet(from, to, Pawn);
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
                MoveGenType::QuietChecks => {

                    let ps  = self.game.get(Pawn, self.side);
                    let ksq = self.game.get(King, !self.side).bitscan();

                    let mut ms = ps.shift_dir(dir);

                    ms &= self.game.state.check_squares[Pawn];

                    let blockers = ps & self.game.get_pins(!self.side);
                    let blockers = blockers.shift_dir(dir);

                    ms |= blockers;

                    ms &= !(occ);
                    if let Some(tgt) = target { ms &= tgt; }

                    for to in ms.into_iter() {
                        if let Some(from) = (!dir).shift_coord(to) {
                            let mv = Move::new_quiet(from, to, Pawn);
                            self.buf.push(mv);
                        }
                    };

                },
            }
        }

        pub fn gen_en_passant(&mut self, target: Option<BitBoard>, mut buf: Option<&mut Vec<Move>>) {

            // let rank5 = if self.side == White { BitBoard::mask_rank(4) } else { BitBoard::mask_rank(3) };

            let ps = self.game.get(Pawn, self.side);
            // let ps = ps & rank5;

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
                    if let Some(mut buf) = buf.as_mut() {buf.push(mv);} else {self.buf.push(mv);}
                });
            }

        }

        pub fn gen_promotions(&mut self, gen: MoveGenType, target: Option<BitBoard>) {
            self._gen_promotions(gen, target, None);
        }

        pub fn _gen_promotions(
            &mut self,
            gen: MoveGenType,
            target: Option<BitBoard>,
            mut buf: Option<&mut Vec<Move>>
        ) {

            let rank7 = if self.side == White { BitBoard::mask_rank(6) } else { BitBoard::mask_rank(1) };

            // let mut buf = if let Some(mut b) = buf.as_mut() { b } else { &mut self.buf }

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
                        if let Some(mut buf) = buf.as_mut() {buf.push(mv);} else {self.buf.push(mv);}
                    } else if gen == MoveGenType::Quiets {
                        let mv = Move::Promotion { from, to, new_piece: Knight };
                        if let Some(mut buf) = buf.as_mut() {buf.push(mv);} else {self.buf.push(mv);}
                        let mv = Move::Promotion { from, to, new_piece: Bishop };
                        if let Some(mut buf) = buf.as_mut() {buf.push(mv);} else {self.buf.push(mv);}
                        let mv = Move::Promotion { from, to, new_piece: Rook };
                        if let Some(mut buf) = buf.as_mut() {buf.push(mv);} else {self.buf.push(mv);}
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
                        // let mv = Move::PromotionCapture { from, to, new_piece: Queen, victim };
                        let mv = Move::PromotionCapture { from, to, pcs: PackedPieces::new(Queen,victim) };
                        if let Some(mut buf) = buf.as_mut() {buf.push(mv);} else {self.buf.push(mv);}
                    } else if gen == MoveGenType::Quiets {
                        // let mv = Move::PromotionCapture { from, to, new_piece: Knight, victim };
                        let mv = Move::PromotionCapture { from, to, pcs: PackedPieces::new(Knight,victim) };
                        if let Some(mut buf) = buf.as_mut() {buf.push(mv);} else {self.buf.push(mv);}
                        // let mv = Move::PromotionCapture { from, to, new_piece: Bishop, victim };
                        let mv = Move::PromotionCapture { from, to, pcs: PackedPieces::new(Bishop,victim) };
                        if let Some(mut buf) = buf.as_mut() {buf.push(mv);} else {self.buf.push(mv);}
                        // let mv = Move::PromotionCapture { from, to, new_piece: Rook, victim };
                        let mv = Move::PromotionCapture { from, to, pcs: PackedPieces::new(Rook,victim) };
                        if let Some(mut buf) = buf.as_mut() {buf.push(mv);} else {self.buf.push(mv);}
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
                            // let mv = Move::Capture { from, to, pc: Knight, victim };
                            let mv = Move::new_capture(from, to, Knight, victim);
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
                MoveGenType::QuietChecks => {

                    let check_squares = self.game.state.check_squares[Knight];
                    let blockers = self.game.get_pins(!self.side);

                    ks.into_iter().for_each(|from| {
                        let mut ms = self.ts.get_knight(from);
                        ms &= !occ;
                        if blockers.is_zero_at(from) {
                            ms &= check_squares;
                        }
                        if let Some(tgt) = target { ms &= tgt; }
                        ms.into_iter().for_each(|to| {
                            let mv = Move::Quiet { from, to, pc: Knight };
                            self.buf.push(mv);
                        });
                    });

                },
            }

        }

    }

    /// Sliding
    impl<'a> MoveGen<'a> {

        pub fn gen_sliding(&mut self, gen: MoveGenType, pc: Piece, target: Option<BitBoard>) {
            let pieces = self.game.get(pc, self.side);

            match gen {
                MoveGenType::Captures => {
                    for from in pieces.into_iter() {
                        let moves   = self._gen_sliding_single(pc, from, None);
                        let mut captures = moves & self.game.get_color(!self.side);
                        if let Some(tgt) = target { captures &= tgt; }
                        captures.into_iter().for_each(|to| {
                            let (_,victim) = self.game.get_at(to).unwrap();
                            let mv = Move::new_capture(from, to, pc, victim);
                            self.buf.push(mv);
                        });
                    }
                },
                MoveGenType::Quiets => {
                    for from in pieces.into_iter() {
                        let moves   = self._gen_sliding_single(pc, from, None);
                        let mut quiets  = moves & self.game.all_empty();
                        if let Some(tgt) = target { quiets &= tgt; }
                        quiets.into_iter().for_each(|sq2| {
                            let mv = Move::Quiet { from, to: sq2, pc };
                            self.buf.push(mv);
                        });
                    }
                },
                MoveGenType::Evasions    => {
                    self.gen_sliding(MoveGenType::Captures, pc, target);
                    self.gen_sliding(MoveGenType::Quiets, pc, target);
                },
                MoveGenType::QuietChecks => {

                    let check_squares = self.game.state.check_squares[pc];
                    let blockers = self.game.get_pins(!self.side);

                    pieces.into_iter().for_each(|from| {
                        let mut ms = self._gen_sliding_single(pc, from, None);
                        ms &= self.game.all_empty();
                        if blockers.is_zero_at(from) {
                            ms &= check_squares;
                        }
                        if let Some(tgt) = target { ms &= tgt; }
                        ms.into_iter().for_each(|to| {
                            let mv = Move::Quiet { from, to, pc };
                            self.buf.push(mv);
                        });
                    });


                },
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
            self._gen_castles(None);
        }

        pub fn _gen_castles(&mut self, mut buf: Option<&mut Vec<Move>>) {
            if self.in_check {
                return;
            }

            let (kingside,queenside) = self.game.state.castling.get_color(self.side);

            let ksq: Coord = self.game.get(King, self.side).bitscan().into();

            if (self.side == White && ksq != Sq::E1.to()) || (self.side == Black && ksq != Sq::E8.to()) {
                return;
            }

            if kingside {

                let mv = Move::new_castle(self.side, true);
                let ((from, to),(rook_from,rook_to)) = mv.castle_moves();

                // if let mv@Move::Castle { from, to, rook_from, rook_to } = Move::CASTLE_KINGSIDE[self.side] {
                // }

                if self.game.get(Rook,self.side).is_one_at(rook_from) {
                    let between = self.ts.between_exclusive(from, rook_from);
                    if (between & self.game.all_occupied()).is_empty() {
                        if !between.into_iter().any(
                            |sq| self.game.find_attacks_by_side(self.ts, sq, !self.side, true)) {
                            if let Some(mut buf) = buf.as_mut() {
                                buf.push(mv);
                            } else {
                                self.buf.push(mv);
                            }
                        }
                    }
                }

            }

            if queenside {

                let mv = Move::new_castle(self.side, false);
                let ((from, to),(rook_from,rook_to)) = mv.castle_moves();

                // if let mv@Move::Castle { from, to, rook_from, rook_to } = Move::CASTLE_QUEENSIDE[self.side] {
                // }

                if self.game.get(Rook,self.side).is_one_at(rook_from) {
                    let between_blocks  = self.ts.between_exclusive(from, rook_from);
                    let between_attacks = Move::CASTLE_QUEENSIDE_BETWEEN[self.side];
                    if (between_blocks & self.game.all_occupied()).is_empty() {
                        if !between_attacks.into_iter().any(
                            |sq| self.game.find_attacks_by_side(self.ts, sq, !self.side, true)) {
                            if let Some(mut buf) = buf.as_mut() {
                                buf.push(mv);
                            } else {
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
                        // let mv = Move::Capture { from, to, pc: King, victim };
                        let mv = Move::new_capture(from, to, King, victim);
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
                // MoveGenType::QuietChecks => unimplemented!(),
                MoveGenType::QuietChecks => {
                    // XXX: King can't give check
                },
            }
        }
    }

}

