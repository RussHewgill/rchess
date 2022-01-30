
use crate::types::*;
use crate::tables::*;
use crate::evaluate::TaperedScore;

use crate::material::VecTable;

pub type PawnTable = VecTable<PawnEval, 8>;

#[derive(Debug,Clone,Copy)]
pub struct PawnEval {
    pub scores:          [TaperedScore; 2],
    pub passed:          BitBoard,
    pub attacks:         BitBoard,
    pub attacks_span:    BitBoard,
}

impl PawnTable { /// get_or_insert
    pub fn get_or_insert(&mut self, ts: &Tables, g: &Game) -> PawnEval {
        if let Some(ev) = self.get(g.zobrist) {
            return *ev;
        }

        let ev = PawnEval::new(ts, g);

        self.insert(g.zobrist, ev);

        ev
    }
}

impl PawnEval { /// build, evaluate
    pub fn new(ts: &Tables, g: &Game) -> Self {

        let mut passed       = BitBoard::empty();
        let mut attacks      = BitBoard::empty();
        let mut attacks_span = BitBoard::empty();

        let mut out = Self {
            scores: [TaperedScore::default(); 2],
            passed,
            attacks,
            attacks_span,
        };

        out.evaluate(ts, g, White);
        out.evaluate(ts, g, Black);

        out
    }

    fn evaluate(&mut self, ts: &Tables, g: &Game, side: Color) -> TaperedScore {

        let score = TaperedScore::default();

        let pawns_us   = g.get(Pawn, side);
        let pawns_them = g.get(Pawn, !side);

        for sq in pawns_us.into_iter() {

            let r = BitBoard::relative_rank(side, sq);

            let opposed = pawns_them & forward_file_bb(side, sq);

            // unimplemented!()
        }

        score
    }

}

/// Pawn Spans
impl Game {

    pub fn pawn_attacks_span(&self, side: Color) -> BitBoard {
        let (d,dw,de) = if side == White { (N,NW,NE) } else { (S,SW,SE) };
        let pawns = self.get(Pawn, side);
        pawns.shift_dir(dw) | pawns.shift_dir(de)
    }

    pub fn _pawn_attacks_span(bb: BitBoard, side: Color) -> BitBoard {
        let (d,dw,de) = if side == White { (N,NW,NE) } else { (S,SW,SE) };
        bb.shift_dir(dw) | bb.shift_dir(de)
    }

}



