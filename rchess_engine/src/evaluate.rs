
use crate::explore::*;
use crate::material_table::MatEval;
use crate::types::*;
use crate::tables::*;
use crate::pawn_hash_table::*;
use crate::endgame::*;

pub use self::tapered::TaperedScore;

use serde::{Serialize,Deserialize};

// pub type Score = i32;
// pub type Score = i16;

// pub static CHECKMATE_VALUE: Score = 100_000_000;
// pub static STALEMATE_VALUE: Score = 20_000_000;
// pub static DRAW_VALUE: Score = 20_000_000;
// // pub static CHECKMATE_VALUE: Score = 32000;
// // pub static STALEMATE_VALUE: Score = 31000;

mod tapered {
    use crate::types::*;
    use crate::tables::*;
    use crate::evaluate::*;

    #[derive(Serialize,Deserialize,Debug,Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
    pub struct TaperedScore {
        mid: Score,
        end: Score,
    }

    impl TaperedScore {
        pub const fn new(mid: Score, end: Score) -> Self {
            Self {
                mid,
                end,
            }
        }

        // pub fn taper(&self, phase: Phase) -> Score {
        //     ((self.mid * (256 - phase as Score)) + (self.end * phase as Score)) / 256
        // }

    }

    impl std::ops::Add for TaperedScore {
        type Output = Self;
        fn add(self, other: Self) -> Self {
            Self {
                mid: self.mid + other.mid,
                end: self.end + other.end,
            }
        }
    }

    impl std::ops::Mul<Score> for TaperedScore {
        type Output = Self;
        fn mul(self, x: Score) -> Self {
            Self {
                mid: self.mid * x,
                end: self.end * x,
            }
        }
    }

}

// #[derive(Default,Eq,PartialEq,PartialOrd,Clone,Copy)]
// pub struct Eval {
//     pub phase:            Phase,
//     pub material:         [[Score; 6]; 2],
//     pub piece_positions:  [[Score; 6]; 2],
// }

// impl Eval {

//     fn sum(&self) -> Score {

//         let white = self.sum_color(White);
//         let black = self.sum_color(Black);

//         // if col == White {
//         //     white - black
//         // } else {
//         //     black - white
//         // }

//         // white - black
//         white + black

//         // unimplemented!("Eval::diff()")
//     }

//     fn sum_material(&self) -> [Score; 2] {
//         let w = self.material[White].iter().sum();
//         let b = self.material[Black].iter().sum();
//         [w,b]
//     }

//     fn sum_positions(&self) -> [Score; 2] {
//         let w = self.piece_positions[White].iter().sum();
//         let b = self.piece_positions[Black].iter().sum();
//         [w,b]
//     }

//     fn sum_color(&self, col: Color) -> Score {
//         let mut score = 0;
//         match col {
//             White => {
//                 for m in self.material[White].iter() {
//                     score += m;
//                 }
//                 for m in self.piece_positions[White].iter() {
//                     score += m;
//                 }
//             },
//             Black => {
//                 for m in self.material[Black].iter() {
//                     score -= m;
//                 }
//                 for m in self.piece_positions[Black].iter() {
//                     score -= m;
//                 }
//             },
//         }
//         score
//     }

//     pub fn get_piece_pos(&self, pc: Piece, col: Color) -> Score {
//         self.piece_positions[col][pc.index()]
//     }
//     pub fn set_piece_pos_mut(&mut self, pc: Piece, col: Color, s: Score) {
//         self.piece_positions[col][pc.index()] = s
//     }
//     pub fn get_piece_mat(&self, pc: Piece, col: Color) -> Score {
//         self.material[col][pc.index()]
//     }
//     pub fn set_piece_mat_mut(&mut self, pc: Piece, col: Color, s: Score) {
//         self.material[col][pc.index()] = s
//     }

// }

/// Main Eval
/// Chooses between NNUE and HCE
impl ExHelper {

    /// NNUE eval is ~18x slower than classic (only material and psqt)
    pub fn evaluate(&mut self, ts: &Tables, g: &Game, quiesce: bool) -> Score {

        let mut use_classical = false;

        // if cfg!(feature = "NNUE")
        //     || self.nnue.is_none()
        //     || (g.psqt_score_end[White] + g.psqt_score_end[White])
        // unimplemented!()

        let stand_pat = self.evaluate_classical(ts, g);
        let score = if g.state.side_to_move == Black { -stand_pat } else { stand_pat };

        score
    }

    pub fn evaluate2(&mut self, ts: &Tables, g: &Game, quiesce: bool) -> Score {

        let use_nnue = cfg!(feature = "NNUE")
            && self.nnue.is_some()
            // && 
            ;

        // if !use_nnue || ()

        if use_nnue {
            if let Some(nnue) = self.nnue.as_mut() {
                let score = nnue.evaluate(&g, true);
                score
            } else { unreachable!() }
        } else {
            let stand_pat = self.evaluate_classical(ts, g);
            let score = if g.state.side_to_move == Black { -stand_pat } else { stand_pat };
            score
        }

        // unimplemented!()
        // self.eval_nn_or_hce(ts, g)
    }

    #[cfg(feature = "nope")]
    pub fn evaluate_classical(&mut self, ts: &Tables, g: &Game) -> Score {
        let score = g.sum_evaluate(ts, &ts.eval_params_mid, &ts.eval_params_mid, None);
        score
    }

    // #[cfg(feature = "nope")]
    pub fn evaluate_classical(&mut self, ts: &Tables, g: &Game) -> Score {
        assert!(!g.state.in_check);

        // let entry = if let Some(entry) = self.material_table.get(g.zobrist) {
        //     if let Some(eg) = entry.eg_val {
        //         return eg.evaluate(ts, g);
        //     } else {
        //         entry
        //     }
        // } else {
        //     let entry = MatEval::new(ts, g);
        //     self.material_table.insert(g.zobrist, entry);
        //     entry
        // };

        if let Some(entry) = self.material_table.get(g.zobrist) {
            if let Some(eg) = entry.eg_val { return eg.evaluate(ts, g); }

            let mut score = entry.score;

            unimplemented!()
        } else {
            unimplemented!()
        }
    }

    // pub fn eval_nn_or_hce(
    //     &self,
    //     // ts:           &'static Tables,
    //     ts:           &Tables,
    //     g:            &Game,
    // ) -> Score {
    //     // TODO: endgame
    //     // TODO: material tables?
    //     if let Some(nnue) = &self.nnue {
    //         /// NNUE Eval, cheap-ish
    //         /// TODO: bench vs evaluate
    //         let mut nn = nnue.borrow_mut();
    //         let score = nn.evaluate(&g, true);
    //         score
    //     } else {
    //         let stand_pat = self.evaluate_classical(ts, g, &self.ph_rw);
    //         let score = if g.state.side_to_move == Black { -stand_pat } else { stand_pat };
    //         score
    //     }
    // }

}

impl ExConfig {

    // pub fn evaluate_classical(&self, ts: &Tables, g: &Game, ph_rw: &PHTable) -> Score {
    //     if let Some(entry) = self.mat
    //     // g.sum_evaluate(ts, &self.eval_params_mid, &self.eval_params_mid, Some(ph_rw))
    //     g.sum_evaluate(ts, &self.eval_params_mid, &self.eval_params_mid, None)
    //     // g.sum_evaluate2(ts)
    // }

}

/// Main Eval 2
impl Game {

    #[cfg(feature = "only_material_eval")]
    pub fn sum_evaluate(
        &self,
        ts:         &Tables,
        ev_mid:     &EvalParams,
        ev_end:     &EvalParams,
        ph_rw:      Option<&PHTable>,
    ) -> Score {
        self.score_material2(White) - self.score_material2(Black)
    }

    #[cfg(not(feature = "only_material_eval"))]
    pub fn sum_evaluate(
        &self,
        ts:         &Tables,
        ev_mid:     &EvalParams,
        ev_end:     &EvalParams,
    ) -> Score {
        // const SIDES: [Color; 2] = [White,Black];
        // let side = self.state.side_to_move;

        let mg = self.sum_evaluate_mg(ts, ev_mid, ev_end);
        let eg = self.sum_evaluate_eg(ts, ev_mid, ev_end);

        // let mut mg = self.score_material(White, true) - self.score_material(Black, true);
        // let mut eg = self.score_material(White, false) - self.score_material(Black, false);

        // mg += self.psqt_score_mid[White] - self.psqt_score_mid[Black];
        // eg += self.psqt_score_end[White] - self.psqt_score_end[Black];

        // self.taper_score2(mg, eg)
        self.taper_score(mg, eg)
    }

    // fn taper_score2(&self, mid: Score, end: Score) -> Score {
    //     let p = self.state.phase as Score;
    //     ((mid * p + ((end * (128 - p)) << 0)) / 128) << 0
    // }

    pub fn sum_evaluate_mg(
        &self,
        ts:          &Tables,
        ev_mid:      &EvalParams,
        ev_end:      &EvalParams,
    ) -> Score {
        let mut score = 0;
        let ev = ev_mid;

        score += self.score_material(White, true) - self.score_material(Black, true);

        if cfg!(feature = "positional_scoring") {
            // score += self.score_psqt(ts, ev, White) - self.score_psqt(ts, ev, Black);
            score += self.psqt_score_mid[White] - self.psqt_score_mid[Black];
        }

        if cfg!(feature = "mobility_scoring") {
            score += self.score_mobility(ts, White) - self.score_mobility(ts, Black);
        }

        score += self.score_pieces_mg(ts, ev, White) - self.score_pieces_mg(ts, ev, Black);
        // score += self.score_pawns(ts, ev, ph_rw, White) - self.score_pawns(ts, ev, ph_rw, Black);

        let pawns: [Score; 2] = self.score_pawns(ts, ev_mid, ev_end, ph_rw, true);
        score += pawns[0] - pawns[1];

        score
    }

    fn sum_evaluate_eg(
        &self,
        ts:          &Tables,
        ev_mid:      &EvalParams,
        ev_end:      &EvalParams,
        ph_rw:       Option<&PHTable>,
    ) -> Score {
        let mut score = 0;
        let ev = ev_end;

        score += self.score_material(White, false) - self.score_material(Black, false);

        if cfg!(feature = "positional_scoring") {
            // score += self.score_psqt(ts, ev, White) - self.score_psqt(ts, ev, Black);
            score += self.psqt_score_end[White] - self.psqt_score_end[Black];
        }

        if cfg!(feature = "mobility_scoring") {
            score += self.score_mobility(ts, White) - self.score_mobility(ts, Black);
        }

        score += self.score_pieces_eg(ts, ev, White) - self.score_pieces_eg(ts, ev, Black);

        let pawns: [Score; 2] = self.score_pawns(ts, ev_mid, ev_end, ph_rw, false);
        score += pawns[0] - pawns[1];

        score
    }

    pub fn score_material(&self, side: Color, mid: bool) -> Score {
        const PCS: [Piece; 6] = [Pawn,Knight,Bishop,Rook,Queen,King];
        if mid {
            PCS.iter().map(|&pc| self.state.material.get(pc, side) as Score * pc.score()).sum()
        } else {
            PCS.iter().map(|&pc| self.state.material.get(pc, side) as Score * pc.score_endgame()).sum()
        }
    }

    // pub fn score_psqt(&self, ts: &Tables, ev: &EvalParams, side: Color) -> Score {
    //     const PCS: [Piece; 6] = [Pawn,Knight,Bishop,Rook,Queen,King];
    //     PCS.iter().map(|&pc| {
    //         self._score_psqt(ev, pc, side)
    //     }).sum()
    // }

    // pub fn _score_psqt(&self, ev: &EvalParams, pc: Piece, side: Color) -> Score {
    //     let pieces = self.get(pc, side);
    //     pieces.into_iter().map(|sq| {
    //         ev.psqt.get(pc, side, sq)
    //         // ev.get_psqt(pc, side, sq)
    //     }).sum()
    // }

    pub fn score_pieces_mg(&self, ts: &Tables, ev: &EvalParams, side: Color) -> Score {
        let mut score = 0;

        score += self.rook_on_open_file(ev, side);
        score += self.outpost_total(ts, ev, side);

        score
    }

    pub fn score_pieces_eg(&self, ts: &Tables, ev: &EvalParams, side: Color) -> Score {
        let mut score = 0;

        score += self.rook_on_open_file(ev, side);
        score += self.outpost_total(ts, ev, side);

        score
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

    #[cfg(feature = "nope")]
    pub fn game_phase_sf(&self) -> (Phase,i16) {

        const MG_LIMIT: Score = 15258;
        const EG_LIMIT: Score = 3915;

        const PHASE_MG: Score = 128;

        // let _ = Pawn.score();

        let npm_w = self.state.npm[White];
        let npm_b = self.state.npm[Black];

        let npm = (npm_w + npm_b).clamp(EG_LIMIT, MG_LIMIT);

        let phase = ((npm - EG_LIMIT) * PHASE_MG) / (MG_LIMIT - EG_LIMIT);

        (0,phase as i16)
    }

    pub fn count_npm(&self, side: Color) -> Score {
        let mut npm = 0;
        for pc in Piece::iter_nonking_nonpawn_pieces() {
            let n = self.state.material.get(pc, side);
            // npm += pc.score_st_phase() * n as Score;
            npm += pc.score() * n as Score;
        }
        npm
    }

    // #[cfg(feature = "nope")]
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

    #[cfg(feature = "nope")]
    pub fn game_phase(&self) -> u8 {
        const PAWN_PH: i16   = 0;
        const KNIGHT_PH: i16 = 1;
        const BISHOP_PH: i16 = 1;
        const ROOK_PH: i16   = 2;
        const QUEEN_PH: i16  = 4;

        const PCS: [Piece; 5] = [Pawn,Knight,Bishop,Rook,Queen];
        const PHASES: [i16; 5] = [PAWN_PH,KNIGHT_PH,BISHOP_PH,ROOK_PH,QUEEN_PH];

        let ph_total = PAWN_PH * 16 + KNIGHT_PH * 4 + BISHOP_PH * 4 + ROOK_PH * 4 + QUEEN_PH * 2;

        let mut phase = ph_total;

        phase -= PAWN_PH *   self.state.material.count_piece(Pawn) as i16;
        phase -= KNIGHT_PH * self.state.material.count_piece(Knight) as i16;
        phase -= BISHOP_PH * self.state.material.count_piece(Bishop) as i16;
        phase -= ROOK_PH *   self.state.material.count_piece(Rook) as i16;
        phase -= QUEEN_PH *  self.state.material.count_piece(Queen) as i16;

        let phase = (phase * 256 + (ph_total / 2)) / ph_total;
        phase.clamp(0,255) as u8
    }

    pub fn taper_score(&self, mid: Score, end: Score) -> Score {
        let p = self.state.phase as Score;
        ((mid * (256 - p)) + (end * p)) / 256
    }

    #[cfg(feature = "nope")]
    fn _game_phase(&self) -> u8 {
        const MIDGAME_LIMIT: Score = 15258;
        const ENDGAME_LIMIT: Score = 3915;

        const NON_PAWN: [Piece; 4] = [Knight,Bishop,Rook,Queen];

        let side = self.state.side_to_move;

        let npm: Score = NON_PAWN.iter().map(|&pc| {
            self.state.material.get(pc, White) as Score * pc.score()
                + self.state.material.get(pc, Black) as Score * pc.score()
        }).sum();

        let npm = Score::max(ENDGAME_LIMIT, Score::min(npm, MIDGAME_LIMIT));

        let out = (((npm - ENDGAME_LIMIT) * 128) / (MIDGAME_LIMIT - ENDGAME_LIMIT)) << 0;

        let out = convert_from_score(out) as i16 + 127;

        out as u8
    }

    fn game_phase2(&self) -> u8 {
        const PAWN_PH: u16   = 0;
        const KNIGHT_PH: u16 = 1;
        const BISHOP_PH: u16 = 1;
        const ROOK_PH: u16   = 2;
        const QUEEN_PH: u16  = 4;

        const PCS: [Piece; 5] = [Pawn,Knight,Bishop,Rook,Queen];
        const PHASES: [u16; 5] = [PAWN_PH,KNIGHT_PH,BISHOP_PH,ROOK_PH,QUEEN_PH];

        let ph_total = PAWN_PH * 16 + KNIGHT_PH * 4 + BISHOP_PH * 4 + ROOK_PH * 4 + QUEEN_PH * 2;
        let mut ph = ph_total;

        for &col in [White,Black].iter() {
            for pc in PCS {
                let x = self.state.material.buf[col][pc.index()] as u16 * PHASES[pc.index()];
                if ph < x {
                    ph = 0;
                    break;
                }
                ph -= x;
            }
        }

        // for &col in [White,Black].iter() {
        //     for pc in PCS {
        //         let ps = self.get(pc, col);
        //         let pn = ps.popcount() as u16;
        //         let x = pn * PHASES[pc.index()];
        //         if ph < x {
        //             ph = 0;
        //             break;
        //         }
        //         ph -= x;
        //     }
        // }

        // eprintln!("ph_total = {:?}", ph_total);
        // eprintln!("ph = {:?}", ph);

        let phase = (ph * 256 + (ph_total / 2)) / ph_total;

        phase.min(255) as u8
    }

}

impl Piece {

    // const fn score_st_phase(self) -> Score {
    //     match self {
    //         Pawn   => 126,
    //         Knight => 781,
    //         Bishop => 825,
    //         Rook   => 1276,
    //         Queen  => 2538,
    //         King   => 32001,
    //     }
    // }

    // pub const fn score_basic(&self) -> Score {
    //     match self {
    //         Pawn   => 100,
    //         Knight => 300,
    //         Bishop => 300,
    //         Rook   => 500,
    //         Queen  => 900,
    //         King   => 32001,
    //     }
    // }

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

/// Material Scoring old
#[cfg(feature = "nope")]
impl Game {

    pub fn score_material(&self, pc: Piece, side: Color) -> Score {
        match pc {
            King   => {
                let s = King.score();
                s
            }
            Rook   => {
                let rs = self.get(Rook, side);

                // let n = rs.popcount() as i32;
                let n = self.state.material.buf[side][Rook.index()] as Score;
                let s = Rook.score() * n;
                s
            },
            // Knight => {},
            Bishop => {
                // let n = self.get(Bishop, side).popcount() as i32;
                let n = self.state.material.buf[side][Bishop.index()] as Score;
                let s = if n > 1 {
                    // 2 bishops = 0.5 pawn
                    Bishop.score() * n + 50
                } else {
                    Bishop.score() * n
                };
                s
            },
            _      => {
                // let s = pc.score() * self.get(pc, side).popcount() as i32;
                let n = self.state.material.buf[side][pc.index()] as Score;
                let s = pc.score() * n;
                s
            },
        }
    }

    // pub fn score_material(&self, pc: Piece, side: Color) -> TaperedScore {
    //     match pc {
    //         King   => {
    //             let s = King.score();
    //             TaperedScore::new(s, s)
    //         }
    //         Rook   => {
    //             let rs = self.get(Rook, side);
    //             // let n = rs.popcount() as i32;
    //             let n = self.state.material[side][Rook.index()] as i32;
    //             let s = Rook.score() * n;
    //             TaperedScore::new(s,s)
    //         },
    //         // Knight => {},
    //         Bishop => {
    //             // let n = self.get(Bishop, side).popcount() as i32;
    //             let n = self.state.material[side][Bishop.index()] as i32;
    //             let s = if n > 1 {
    //                 // 2 bishops = 0.5 pawn
    //                 Bishop.score() * n + 50
    //             } else {
    //                 Bishop.score() * n
    //             };
    //             TaperedScore::new(s,s)
    //         },
    //         _      => {
    //             // let s = pc.score() * self.get(pc, side).popcount() as i32;
    //             let n = self.state.material[side][pc.index()] as i32;
    //             let s = pc.score() * n;
    //             TaperedScore::new(s,s)
    //         },
    //     }
    // }

}

/// Outposts
impl Game {

    pub fn outpost_total(&self, ts: &Tables, ev: &EvalParams, side: Color) -> Score {
        let outposts  = self.outpost_squares(ev, side);
        let reachable = self.reachable_outposts(ts, ev, side);
        outposts + reachable * ev.pieces.outpost.reachable_knight
        // outposts + reachable * ev[EPIndex::OutpostReachableKnight]
    }

    pub fn reachable_outposts(&self, ts: &Tables, ev: &EvalParams, side: Color) -> Score {
        self.get(Knight, side).into_iter()
            .map(|sq| {
                let moves = ts.get_knight(sq);
                let moves = moves & self.all_empty();
                moves.into_iter().filter(|&sq2| {
                    self.outpost_square(ev, sq2.into(), side)
                }).count() as Score
            }).sum()
    }

    pub fn outpost_squares(&self, ev: &EvalParams, side: Color) -> Score {
        let n = self.get(Knight, side).into_iter()
            .filter(|&sq| self.outpost_square(ev, sq.into(), side)).count() as Score;
        let b = self.get(Bishop, side).into_iter()
            .filter(|&sq| self.outpost_square(ev, sq.into(), side)).count() as Score;
        n * ev.pieces.outpost.outpost_knight + b * ev.pieces.outpost.outpost_bishop
        // n * ev[EPIndex::OutpostKnight] + b * ev[EPIndex::OutpostBishop]
    }

    pub fn outpost_square(&self, ev: &EvalParams, c0: Coord, side: Color) -> bool {
        // if c0.rank() < 3 || c0.rank() > 7 { return false; }
        let rank = BitBoard::relative_rank(side, c0);
        if rank < 4 || rank > 6 { return false; }

        let (dw,de) = if side == White { (NW,NE) } else { (SW,SE) };

        let b0 = BitBoard::single(c0);
        if (self.get(Pawn, side) & (b0.shift_dir(!dw) | b0.shift_dir(!de))).is_empty() {
            false
        } else {
            let b1 = b0.shift_dir(dw) | b0.shift_dir(de);
            let b2 = if side == White { b1.fill_north() } else { b1.fill_south() };

            (b2 & self.get(Pawn, !side)).is_empty()
        }
    }

}

/// Misc Positional
impl Game {

    /// Open      = no pawns = 2
    /// Half open = only enemy pawns = 1
    pub fn rook_on_open_file(&self, ev: &EvalParams, side: Color) -> Score {
        let pawns_own   = self.get(Pawn, side);
        let pawns_other = self.get(Pawn, !side);

        self.get(Rook, side).into_iter().map(|sq| {
            let c0 = Coord::from(sq);
            let file = c0.file();
            if (BitBoard::mask_file(file) & pawns_own).is_not_empty() {
                0
            } else if (BitBoard::mask_file(file) & pawns_other).is_not_empty() {
                // ev[EPIndex::PPRookHalfOpenFile]
                ev.pieces.rook_open_file[0]
            } else {
                // ev[EPIndex::PPRookOpenFile]
                ev.pieces.rook_open_file[1]
            }
        }).sum()
    }

}

/// Mobility
impl Game {

    pub fn score_mobility(&self, ts: &Tables, side: Color) -> Score {
        const PCS: [Piece; 4] = [Knight,Bishop,Rook,Queen];
        let mob = self.mobility_area(ts, side);

        PCS.iter().map(|&pc| {
            self.get(pc, side).into_iter().map(|sq| {
                let c0: Coord = sq.into();
                self._score_mobility(ts, mob, pc, c0, side)
            }).sum::<Score>()
        }).sum::<Score>()
    }

    pub fn _score_mobility(&self, ts: &Tables, mob: BitBoard, pc: Piece, c0: Coord, side: Color) -> Score {
        match pc {
            Knight => {
                let mvs = ts.get_knight(c0);
                (mvs & mob).popcount() as Score
            },
            Bishop => {
                let occ = self.all_occupied() & !self.get(Queen, side);
                let mvs = ts.attacks_bishop(c0, occ);
                (mvs & mob).popcount() as Score
            },
            Rook   => {
                let occ = self.all_occupied() & !self.get(Queen, side);
                let mvs = ts.attacks_rook(c0, occ);
                (mvs & mob).popcount() as Score
            },
            Queen  => {
                let mvs0 = ts.attacks_bishop(c0, self.all_occupied());
                let mvs1 = ts.attacks_rook(c0, self.all_occupied());
                ((mvs0 | mvs1) & mob).popcount() as Score
            },

            Pawn   => unimplemented!(),
            King   => unimplemented!(),
        }

    }

    pub fn mobility_area(&self, ts: &Tables, side: Color) -> BitBoard {
        const WHITE_R23: BitBoard = BitBoard(0x0000000000ffff00);
        const BLACK_R67: BitBoard = BitBoard(0x00ffff0000000000);

        let mut mob = !BitBoard::empty();

        mob &= !self.get(King, side);
        mob &= !self.get(Queen, side);
        mob &= !(self.get(Pawn, side) & (if side == White { WHITE_R23 } else { BLACK_R67 }));
        mob &= !self.get_pins(side);

        let (d,dw,de) = if side == White { (S,SW,SE) } else { (N,NW,NE) };

        let enemy_pawns = self.get(Pawn, !side);
        // Enemy pawn attacks
        mob &= !(enemy_pawns.shift_dir(dw) | enemy_pawns.shift_dir(de));

        // Blocked pawns
        mob &= !(self.get(Pawn, side) & enemy_pawns.shift_dir(d));

        mob
    }

}

/// King Safety
impl Game {
    fn king_safety(&self, ts: &Tables, side: Color) -> Score {

        let king = self.get(King, side).bitscan();
        let sqs = ts.get_king(king);
        let pawns = self.get(Pawn, side) & sqs;
        let pawn_shield = pawns.popcount() as Score;

        unimplemented!()
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

/// Pawn Structure
impl Game {

    /// All values are added, penalties are negative
    pub fn score_pawns(
        &self,
        ts:           &Tables,
        ev_mid:       &EvalParams,
        ev_end:       &EvalParams,
        ph_rw:        Option<&PHTable>,
        mid:          bool,
    ) -> [Score; 2] {

        // ph.get_scores(mid)
        if let Some(ph_rw) = ph_rw {
            let scores = ph_rw.get_scores(ts, &self, ev_mid, ev_end);
            scores.get_scores(mid)
        } else {
            let mut ph = PHEntry::get_or_insert_pawns(ts, &self, ev_mid, ev_end, None);
            // let scores = PHScore::generate(ts, g, &ph, ev_mid, ev_end);
            if mid {
                let mid_w = self._score_pawns_mut(&ph, &ev_mid, White);
                let mid_b = self._score_pawns_mut(&ph, &ev_mid, Black);
                [mid_w, mid_b]
            } else {
                let end_w = self._score_pawns_mut(&ph, &ev_end, White);
                let end_b = self._score_pawns_mut(&ph, &ev_end, Black);
                [end_w, end_b]
            }
        }
    }

    /// Take PHEntry without scores and update score for side, phase
    pub fn _score_pawns_mut(
        &self,
        // ts:           &Tables,
        ph:           &PHEntry,
        ev:           &EvalParams,
        side:         Color,
    ) -> Score {
        let mut score = 0;
        let pawns = self.get(Pawn, side);

        // score += ev.pawns.blocked_r5 * (ph.blocked & BitBoard::mask_rank(r)).popcount() as Score;
        // score += ev.pawns.blocked_r6 * ph.blocked_r6.popcount() as Score;

        // score += ev.pawns.doubled_isolated * (ph.doubled_isolated & pawns).popcount() as Score;

        // score += ev[EPIndex::PawnDoubled] * (ph.doubled & pawns).popcount() as Score;
        // score += ev[EPIndex::PawnIsolated] * (ph.isolated & pawns).popcount() as Score;
        // score += ev[EPIndex::PawnBackward] * (ph.backward & pawns).popcount() as Score;
        score += ev.pawns.doubled * (ph.doubled & pawns).popcount() as Score;
        score += ev.pawns.isolated * (ph.isolated & pawns).popcount() as Score;
        score += ev.pawns.backward * (ph.backward & pawns).popcount() as Score;

        pawns.into_iter().for_each(|sq| {
            score += self._pawn_connected_bonus(ev, &ph, sq.into(), side);
        });

        // ph.update_score_mut(score, ev.mid, side);
        score
    }

    pub fn gen_ph_entry(&self, ts: &Tables, ev_mid: &EvalParams, ev_end: &EvalParams) -> PHEntry {
        let mut out = PHEntry::default();

        for side in [White,Black] {
            let pawns = self.get(Pawn, side);

            pawns.into_iter().for_each(|sq| {
                let c0 = Coord::from(sq);

                match self._pawn_supported(c0, side) {
                    1 => out.supported_1.set_one_mut(c0),
                    2 => out.supported_2.set_one_mut(c0),
                    _ => {},
                }
                if self._pawn_phalanx(c0, side) { out.phalanx.set_one_mut(c0); }
                // TODO: passed
                // TODO: candidate
                if self._pawn_blocked(ts, c0, side).is_some() { out.blocked.set_one_mut(c0); }
                if self._pawn_opposed(c0, side) { out.opposed.set_one_mut(c0); }

                if self._pawn_doubled(ts, c0, side) { out.doubled.set_one_mut(c0); }
                if self._pawn_isolated(ts, c0, side) { out.isolated.set_one_mut(c0); }
                if self._pawn_backward(ts, c0, side) { out.backward.set_one_mut(c0); }

            });

            out.connected = out.phalanx | out.supported_1 | out.supported_2;
        }

        out
    }

    pub fn _pawn_connected_bonus(&self, ev: &EvalParams, ph: &PHEntry, c0: Coord, side: Color) -> Score {

        let rank = BitBoard::relative_rank(side, c0);
        if rank == 0 || rank == 7 { return 0; }

        let op  = if ph.opposed.is_one_at(c0) { 1 } else { 0 };
        let px  = if ph.phalanx.is_one_at(c0) { 1 } else { 0 };

        let su1 = ph.supported_1.is_one_at(c0);
        let su2 = ph.supported_2.is_one_at(c0);
        let su = if su2 { 2 } else if su1 { 1 } else { 0 };

        ev.pawns.connected_ranks[rank as usize] * (2 + px - op) + ev.pawns.supported * su
        // ev[EPIndex::PawnConnectedRank(rank)] * (2 + px - op) + ev[EPIndex::PawnSupported] * su
    }

    pub fn _pawn_opposed(&self, c0: Coord, side: Color) -> bool {
        let b = BitBoard::single(c0);
        let m = if side == White { b.fill_north() } else { b.fill_south() };
        (self.get(Pawn, !side) & (m & !b)).is_not_empty()
    }

    pub fn _pawn_supported(&self, c0: Coord, side: Color) -> Score {
        let b = Self::_pawn_attacks_span(BitBoard::single(c0), !side);
        (self.get(Pawn,side) & b).popcount() as Score
    }

    pub fn _pawn_phalanx(&self, c0: Coord, side: Color) -> bool {
        let pawns = self.get(Pawn, side);
        let b = BitBoard::single(c0);
        let b0 = b.shift_dir(W) | b.shift_dir(E);
        (pawns & b0).is_not_empty()
    }

    pub fn _pawn_connected(&self, c0: Coord, side: Color) -> bool {
        self._pawn_supported(c0, side) > 0 || self._pawn_phalanx(c0, side)
    }

    pub fn _pawn_doubled(&self, ts: &Tables, c0: Coord, side: Color) -> bool {
        let b = BitBoard::single(c0);
        let ps = self.get(Pawn, side) & !b;
        let m = c0.mask_file() & !b;
        if (ps & m).is_empty() { return false; }
        let sups = Self::_pawn_attacks_span(b, !side) & ps;
        sups.is_empty()
    }

    // TODO: bench not finding rank
    pub fn _pawn_blocked(&self, ts: &Tables, c0: Coord, side: Color) -> Option<u8> {

        let b = BitBoard::single(c0);
        let b0 = b.shift_dir(side.fold(N, S));

        if (self.get(Pawn, !side) & b0).is_empty() { return None; }

        Some(BitBoard::relative_rank(side, c0))
    }

    pub fn _pawn_isolated(&self, ts: &Tables, c0: Coord, side: Color) -> bool {
        let ps = self.get(Pawn, side);
        let file = c0.file();

        if file == 0 {
            let mask1 = BitBoard::mask_file(file + 1);
            (ps & mask1).is_empty()
        } else if file == 7 {
            let mask0 = BitBoard::mask_file(file - 1);
            (ps & mask0).is_empty()
        } else {
            let mask0 = BitBoard::mask_file(file - 1);
            let mask1 = BitBoard::mask_file(file + 1);
            (ps & mask0).is_empty() && (ps & mask1).is_empty()
        }
    }

    pub fn _pawn_backward(&self, ts: &Tables, c0: Coord, side: Color) -> bool {

        let b = BitBoard::single(c0);

        let b0 = b.shift_dir(E) | b.shift_dir(W);

        let b0 = if side == White { b0.fill_south() } else { b0.fill_north() };

        // eprintln!("b0 = {:?}", b0);

        if (b0 & self.get(Pawn, side)).is_not_empty() { return false; }

        let b1 = b | Self::_pawn_attacks_span(b, side);
        let b1 = b1.shift_dir(if side == White { N } else { S });

        (b1 & self.get(Pawn, !side)).is_not_empty()
    }

}

// impl std::fmt::Debug for Eval {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let [m_w,m_b] = self.sum_material();
//         let [p_w,p_b] = self.sum_positions();
//         let m_w = m_w - King.score();
//         let m_b = m_b - King.score();
//         f.write_str(&format!("ph: {}, mat: ({},{}), pos: ({},{})",
//                              self.phase, m_w, m_b, p_w, p_b,))?;
//         Ok(())
//     }
// }


