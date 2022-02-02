
use std::collections::HashMap;

use crate::endgame::*;
use crate::endgame::helpers::is_kx_vs_k;
use crate::evaluate::TaperedScore;
use crate::types::*;
use crate::tables::*;

use crate::material::vec_table::VecTable;

/// 8192 * 12 / 1024 = 96 kB
const MAT_TABLE_SIZE: usize = 8192 * std::mem::size_of::<MatEval>() / 1024;

// pub type MaterialTable = VecTable<MatEval, MAT_TABLE_SIZE>;

#[derive(Debug,Default,Clone)]
pub struct MaterialTable(HashMap<Zobrist, MatEval>);

#[derive(Debug,Clone,Copy)]
/// Score is only the material balance
pub struct MatEval {

    pub material_score: TaperedScore,
    pub phase:          Phase,

    // pub scaling_func:   

    pub eg_val:         Option<EndGameType>,

}

impl MaterialTable {
    pub fn get_or_insert(&mut self, ts: &Tables, g: &Game) -> MatEval {
        if let Some(me) = self.0.get(&g.zobrist) {
            return *me;
        }

        let me = MatEval::new(ts, g);

        self.0.insert(g.zobrist, me);

        me
    }
}

impl MatEval {

    /// TODO: sf quadratics?
    #[cfg(feature = "nope")]
    pub fn imbalance(mat: &Material, side: Color) -> Score {
        // let mut score = 0;
        // for pc in Piece::iter_nonking_pieces() {
        //     let n = mat.get(pc, side);
        //     if n == 0 { continue; }
        //     score += n as Score * pc.score();
        // }
        // score
        unimplemented!()
    }

    pub fn imbalance(g: &Game) -> TaperedScore {
        g.state.npm[White]
            + Pawn.score_tapered() * g.state.material.get(Pawn, White) as Score
            - g.state.npm[Black]
            - Pawn.score_tapered() * g.state.material.get(Pawn, Black) as Score
    }

    pub fn new(ts: &Tables, g: &Game) -> Self {

        // let score = g.sum_evaluate(ts, &ts.eval_params_mid, &ts.eval_params_mid, None);
        let mut material_score = Self::imbalance(g);

        // if is_kx_vs_k(g, g.state.side_to_move) {
        //     unimplemented!()
        // }

        // if g.state.npm[White] + g.state.npm[Black] == 0 && g.state.material.get_both(Pawn) > 0 {
        //     unimplemented!()
        // }

        let eg_val = None;

        Self {
            material_score,
            phase:     g.state.phase,
            eg_val,
        }

    }

    // pub fn new(g: &Game, score: Score) -> Self {
    //     Self {
    //         score,
    //         phase:     g.state.phase,
    //         // factor:    [ScaleFactor::Normal; 2],
    //         eg_val:    None,
    //         // eg_scale:  None,
    //     }
    // }

}


