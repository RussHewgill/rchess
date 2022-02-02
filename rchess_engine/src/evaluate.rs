
use crate::explore::*;
// use crate::material_table::{MatEval,PawnEval,MaterialTable,PawnTable};
use crate::material::{MatEval,PawnEval,MaterialTable,PawnTable};
use crate::types::*;
use crate::tables::*;
use crate::endgame::*;

pub use self::tapered::TaperedScore;

mod tapered {
    use crate::types::*;

    use serde::{Serialize,Deserialize};
    use derive_more::*;

    // #[derive(Serialize,Deserialize,Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]

    #[derive(Default,Eq,Ord,PartialEq,PartialOrd,Hash,Clone,Copy,
             Serialize,Deserialize,
             Add,Sub,Mul,Div,Sum,Neg,
             AddAssign,SubAssign,MulAssign,
    )]
    pub struct TaperedScore {
        pub mid: Score,
        pub end: Score,
    }

    impl std::fmt::Debug for TaperedScore {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&format!("TS( {}, {} )", self.mid, self.end))?;
            Ok(())
        }
    }

    impl TaperedScore {
        pub const fn new(mid: Score, end: Score) -> Self {
            Self {
                mid,
                end,
            }
        }
    }

    /// convert TaperedScore to Score
    impl TaperedScore {
        pub fn taper(&self, g: &Game) -> Score {
            let p = g.state.phase as Score;
            ((self.mid * (256 - p)) + (self.end * p)) / 256
        }
    }

}

/// Evaluate
impl ExHelper {

    fn use_nnue_imbalance(g: &Game) -> bool {
        let imbalance = g.state.npm[White]
            + Pawn.score_tapered() * g.state.material.get(Pawn, White) as Score
            - g.state.npm[Black]
            - Pawn.score_tapered() * g.state.material.get(Pawn, Black) as Score;
        imbalance.taper(g) < 3 * Pawn.score()
    }

    /// NNUE eval is ~18x slower than classic (only material and psqt)
    /// so fallback to classic for large material imbalance
    pub fn evaluate(
        &mut self,
        ts:       &Tables,
        stats:    &mut SearchStats,
        g:        &Game,
        ply:      Depth,
        quiesce:  bool,
    ) -> Score {

        // /// evaluate is only called from quiet positions
        // assert!(!g.state.in_check);

        let use_nnue = cfg!(feature = "nnue")
            && self.nnue.is_some()
            && Self::use_nnue_imbalance(g);

        if use_nnue {
            stats.eval_nnue += 1;
            if let Some(nnue) = self.nnue.as_mut() {
                // let score = nnue.evaluate(&g, true, ply);
                let score = nnue.evaluate(&g, true);
                score
            } else { unreachable!() }
        } else {
            let stand_pat = self.evaluate_classical(ts, g);
            let score = if g.state.side_to_move == Black { -stand_pat } else { stand_pat };
            stats.eval_classical += 1;
            score
        }
    }

}

/// evaluate_classical
impl ExHelper {

    pub fn evaluate_classical(&mut self, ts: &Tables, g: &Game) -> Score {
        let score = self._evaluate_classical::<false>(ts, g);
        score.taper(g)
    }

    pub fn _evaluate_classical<const TR: bool>(&mut self, ts: &Tables, g: &Game) -> TaperedScore {

        let me = self.material_table.get_or_insert(ts, g);

        /// TODO: endgames
        if let Some(eg) = me.eg_val {
            // return eg.evaluate(ts, g);
            unimplemented!()
        }

        let mut score = g.psqt_score[White] - g.psqt_score[Black];
        if TR { eprintln!("psqt = {:?}", (g.psqt_score[White],g.psqt_score[Black])); }

        score += me.material_score;
        if TR { eprintln!("material = {:?}", me.material_score); }

        let pawns = self.pawn_table.get_or_insert(ts, g);

        score += pawns.scores[White] - pawns.scores[Black];
        if TR { eprintln!("pawns = {:?}", (pawns.scores[White], pawns.scores[Black])); }

        score
    }
}

/// Piece Scores
impl Piece {

    const SC_PAWN_MG: Score = 100;
    const SC_PAWN_EG: Score = 200;

    const SC_KNIGHT_MG: Score = 320;
    const SC_KNIGHT_EG: Score = 320;

    const SC_BISHOP_MG: Score = 330;
    const SC_BISHOP_EG: Score = 330;

    const SC_ROOK_MG: Score = 500;
    const SC_ROOK_EG: Score = 500;

    const SC_QUEEN_MG: Score = 900;
    const SC_QUEEN_EG: Score = 900;

    const SC_KING_MG: Score = 32001;
    const SC_KING_EG: Score = 32001;


    pub const fn score_tapered(&self) -> TaperedScore {
        match self {
            Pawn   => TaperedScore::new(100,200),
            Knight => TaperedScore::new(320,320),
            Bishop => TaperedScore::new(330,330),
            Rook   => TaperedScore::new(500,500),
            Queen  => TaperedScore::new(900,900),
            King   => TaperedScore::new(32001,32001),
        }
    }

    pub const fn score(&self) -> Score {
        match self {
            Pawn   => 100,
            Knight => 320,
            Bishop => 330,
            Rook   => 500,
            Queen  => 900,
            King   => 32001,
        }
    }
    /// Same, but pawns are worth twice as much
    pub const fn score_endgame(&self) -> Score {
        match self {
            Pawn   => 200,
            Knight => 320,
            Bishop => 330,
            Rook   => 500,
            Queen  => 900,
            King   => 32001,
        }
    }
}

const PAWN_PH: i16   = 0;
const KNIGHT_PH: i16 = 1;
const BISHOP_PH: i16 = 1;
const ROOK_PH: i16   = 2;
const QUEEN_PH: i16  = 4;
const PHASES: [i16; 5] = [PAWN_PH,KNIGHT_PH,BISHOP_PH,ROOK_PH,QUEEN_PH];
const PH_TOTAL: i16 = PAWN_PH * 16 + KNIGHT_PH * 4 + BISHOP_PH * 4 + ROOK_PH * 4 + QUEEN_PH * 2;

/// Phase
impl Game {

    pub fn count_npm(&self, side: Color) -> TaperedScore {
        let mut npm = TaperedScore::default();
        for pc in Piece::iter_nonking_nonpawn_pieces() {
            let n = self.state.material.get(pc, side);
            // npm += pc.score_st_phase() * n as Score;
            npm += pc.score_tapered() * n as Score;
        }
        npm
    }

    pub fn increment_phase_mut(&mut self, mv: Move) {

        if mv.filter_promotion() {
            let new_pc = mv.new_piece().unwrap();

            self.state.phase_unscaled += PHASES[Pawn];
            self.state.phase_unscaled -= PHASES[new_pc];
        }

        if let Some(victim) = mv.victim() {
            self.state.phase_unscaled += PHASES[victim];
        }

        let phase = (self.state.phase_unscaled * 256 + (PH_TOTAL / 2)) / PH_TOTAL;
        let phase = phase.clamp(0,255) as u8;
        self.state.phase = phase;

    }

    // #[cfg(feature = "nope")]
    pub fn game_phase(&self) -> (Phase,i16) {
        let mut phase_unscaled = PH_TOTAL;

        phase_unscaled -= PAWN_PH *   self.state.material.count_piece(Pawn) as i16;
        phase_unscaled -= KNIGHT_PH * self.state.material.count_piece(Knight) as i16;
        phase_unscaled -= BISHOP_PH * self.state.material.count_piece(Bishop) as i16;
        phase_unscaled -= ROOK_PH *   self.state.material.count_piece(Rook) as i16;
        phase_unscaled -= QUEEN_PH *  self.state.material.count_piece(Queen) as i16;

        let phase = (phase_unscaled * 256 + (PH_TOTAL / 2)) / PH_TOTAL;
        let phase = phase.clamp(0,255) as u8;

        (phase,phase_unscaled)
    }


}
