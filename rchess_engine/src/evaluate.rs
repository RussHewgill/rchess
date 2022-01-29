
use crate::explore::*;
use crate::material_table::{MatEval,PawnEval,MaterialTable,PawnTable};
use crate::types::*;
use crate::tables::*;
use crate::endgame::*;

pub use self::tapered::TaperedScore;

mod tapered {
    use crate::types::*;

    use derive_more::*;

    // #[derive(Serialize,Deserialize,Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]

    #[derive(Debug,Default,Eq,Ord,PartialEq,PartialOrd,Hash,Clone,Copy,
             Add,Sub,Mul,Div,Sum,Neg,
             AddAssign,MulAssign,
    )]
    pub struct TaperedScore {
        mid: Score,
        end: Score,
    }

    impl TaperedScore {
        pub fn new(mid: Score, end: Score) -> Self {
            Self {
                mid,
                end,
            }
        }
    }

}


impl ExHelper {

    /// NNUE eval is ~18x slower than classic (only material and psqt)
    /// so fallback to classic for large material imbalance
    pub fn evaluate(&mut self, ts: &Tables, g: &Game, quiesce: bool) -> Score {
        /// evaluate is only called from quiet positions
        assert!(!g.state.in_check);

        unimplemented!()
    }

    pub fn evaluate_classical(&mut self, ts: &Tables, g: &Game) -> Score {




        unimplemented!()
    }

}


impl Piece {
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

    pub fn count_npm(&self, side: Color) -> Score {
        let mut npm = 0;
        for pc in Piece::iter_nonking_nonpawn_pieces() {
            let n = self.state.material.get(pc, side);
            // npm += pc.score_st_phase() * n as Score;
            npm += pc.score() * n as Score;
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
