
use crate::types::*;
use crate::tables::*;

use arrayvec::ArrayVec;

// use strum::{IntoEnumIterator,EnumIter};

#[derive(Debug,Eq,PartialEq,Ord,PartialOrd,Clone,Copy)]
pub enum MoveGenType {
    Captures,
    Quiets,
    // QuietChecks,
    // Evasions,
    // Pseudo,
    // AllLegal,
}

// #[derive(Debug,Eq,PartialEq,Ord,PartialOrd,Clone,Copy,EnumIter)]
#[derive(Debug,Eq,PartialEq,Ord,PartialOrd,Clone,Copy)]
pub enum MoveGenStage {
    // Init = 0,
    Hash = 0,
    Captures,
    Quiets,
}

impl MoveGenStage {
    pub fn next(self) -> Option<Self> {
        use MoveGenStage::*;
        match self {
            Hash     => Some(Captures),
            Captures => Some(Quiets),
            Quiets   => None,
        }
    }
}

#[derive(Debug,Clone)]
pub struct MoveGen<'a> {
    ts:         &'a Tables,
    game:       Game,

    in_check:   bool,
    side:       Color,

    stage:      MoveGenStage,
    buf:        ArrayVec<Move,256>,

    hashmove:   Option<Move>,
    depth:      Depth,
    ply:        Depth,
}

impl<'a> MoveGen<'a> {
    pub fn new(
        ts:             &'a Tables,
        game:           Game,
        hashmove:       Option<Move>,
        depth:          Depth,
        ply:            Depth,
    ) -> Self {
        let in_check = game.state.checkers.is_not_empty();
        let side = game.state.side_to_move;
        Self {
            ts,
            game,
            in_check,
            side,
            stage:     MoveGenStage::Hash,
            buf:       ArrayVec::new(),
            hashmove,
            depth,
            ply,
        }
    }
}

impl<'a> Iterator for MoveGen<'a> {
    type Item = Move;
    fn next(&mut self) -> Option<Self::Item> {
        match self.stage {
            MoveGenStage::Hash     => {
                if let Some(mv) = self.hashmove {
                    if self.move_is_legal(mv) {
                        return Some(mv);
                    }
                }

                self.stage = self.stage.next()?;

                self.gen_all(MoveGenType::Captures);
                // TODO: sort here?

                self.next()
            },
            MoveGenStage::Captures => {
                if let Some(mv) = self.buf.pop() {
                    Some(mv)
                } else {
                    unimplemented!()
                }
            },
            MoveGenStage::Quiets   => {
                unimplemented!()
            },
        }
    }
}

/// Generate
impl<'a> MoveGen<'a> {
    pub fn gen_all(&mut self, gen: MoveGenType) {
        if self.in_check {
            self._gen_all_in_check(gen);
        } else {
            self._gen_all(gen);
        }
    }

    fn _gen_all(&mut self, gen: MoveGenType) {
        unimplemented!()
    }

    fn _gen_all_in_check(&mut self, gen: MoveGenType) {
        unimplemented!()
    }

}

/// Check legal
impl<'a> MoveGen<'a> {
    pub fn move_is_legal(&self, mv: Move) -> bool {
        unimplemented!()
    }
}

/// Pawns
impl<'a> MoveGen<'a> {
    pub fn gen_pawns(&mut self, gen: MoveGenType) {
        unimplemented!()
    }
}

/// Knights
impl<'a> MoveGen<'a> {
    // pub fn gen_knights(&mut self, gen: MoveGenType) -> impl Iterator<Item = (Piece,Coord)> {
    pub fn gen_knights(&'a self, gen: MoveGenType) -> impl Iterator<Item = Move> + 'a {
        let occ = self.game.all_occupied();
        let ks = self.game.get(Knight, self.side);

        // ks.into_iter().map(|sq| (Knight,sq))
        // ks.into_iter().map(|sq| {
        //     let ms = ts.get_knight(sq);
        // })

        ks.into_iter().flat_map(move |from| {
            let ms = self.ts.get_knight(from);

            match gen {
                MoveGenType::Captures => {
                    let captures = *ms & self.game.get_color(!self.side);
                    captures.into_iter().map(move |to| {
                        let (_,victim) = self.game.get_at(to).unwrap();
                        Move::Capture { from, to, pc: Knight, victim }
                    })
                },
                MoveGenType::Quiets => {
                    let quiets   = *ms & !occ;
                    quiets.into_iter().map(move |to| {
                        Move::Quiet { from, to, pc: Knight }
                    })
                },
            }

            // unimplemented!()
            // (from, )
        })
    }
}






