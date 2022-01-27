
#![allow(non_camel_case_types)]

use crate::movegen::MoveGen;
use crate::movegen::MoveGenType;
use crate::types::*;
use crate::tables::*;

use self::helpers::*;

use std::collections::HashMap;

pub trait EndGame {

    // const NPM: Score;
    // const PAWNS: u8;

    fn evaluate(self, ts: &Tables, g: &Game) -> Score;

    fn verify_material(self, g: &Game, side: Color) -> bool;

}


pub mod helpers {
    use crate::types::*;

    pub fn _verify_material(g: &Game, side: Color, npm: Score, pawns: u8) -> bool {
        g.state.material.non_pawn_value(side) == npm
            && g.state.material.get(Pawn, side) == pawns
    }

    fn edge_distance(rank_or_file: u8) -> u8 {
        rank_or_file.min(8 - rank_or_file)
    }

    pub fn push_king_to_edge(ksq: Coord) -> Score {
        let rd = edge_distance(ksq.rank()) as Score;
        let fd = edge_distance(ksq.file()) as Score;

        // XXX: sf magic
        90 - (7 * fd * fd / 2 + 7 * rd * rd / 2)
    }

    pub fn push_king_to_corner(ksq: Coord) -> Score {
        let x = 7 - ksq.rank() as Score - ksq.file() as Score;
        x.abs()
    }

    pub fn push_close(c0: Coord, c1: Coord) -> Score {
        let d = c0.square_dist(c1);
        140 - 20 * d as Score
    }

}

/// from stockfish
#[derive(Debug,PartialEq,PartialOrd,Clone,Copy)]
pub enum ScaleFactor {
    Draw   = 0,
    Normal = 64,
    Max    = 128,
    None   = 255,
}

#[derive(Debug,Clone)]
pub struct EndGameMaps {

    // value:          HashMap<Score, EndGame>,
    // scalefactors:   HashMap<ScaleFactor, EndGame>,

    // value:          HashMap<Score, Box<dyn EndGame>>,
    // scalefactors:   HashMap<ScaleFactor, Box<dyn EndGame>>,

}

impl EndGameMaps {
    pub fn init() -> Self {
        // Self {}
        unimplemented!()
    }

    // pub fn get_value(&self, score: Score) -> Option<EndGame> {
    //     unimplemented!()
    // }

    // pub fn get_scalefactor(&self, scale: ScaleFactor) -> Option<EndGame> {
    //     unimplemented!()
    // }

    // pub fn get_endgame(g: &Game) -> Option<EndGame> {
    //     unimplemented!()
    // }

}

#[derive(Debug,PartialEq,PartialOrd,Clone,Copy)]
pub enum EndGameType {
    KXvK,
    KPvK,
}

impl EndGameType {
    pub fn evaluate(self, ts: &Tables, g: &Game) -> Score {
        match self {
            Self::KXvK => endgame_kx_vs_k(ts, g),
            _          => unimplemented!(),
        }
    }
}

pub struct KX_VS_K;

impl EndGame for KX_VS_K {

    fn evaluate(self, ts: &Tables, g: &Game) -> Score {
        unimplemented!()
    }

    fn verify_material(self, g: &Game, side: Color) -> bool {
        // _verify_material(g, side, npm, pawns)
        unimplemented!()
    }

}

fn endgame_kx_vs_k(ts: &Tables, g: &Game) -> Score {

    let weak_side: Color = unimplemented!();

    // assert!(EndGame::KXvK.verify_material(g, side));
    assert!(!g.state.in_check);

    if g.state.side_to_move == weak_side {
        if MoveGen::generate_list_legal(ts, g, Some(MoveGenType::Captures)).is_empty()
            && MoveGen::generate_list_legal(ts, g, Some(MoveGenType::Quiets)).is_empty() {
                return DRAW_VALUE;
            }
    }

    let strong_side = !weak_side;

    let ksq_strong = g.get(King, strong_side).bitscan();
    let ksq_weak   = g.get(King, weak_side).bitscan();

    let mat = &g.state.material;

    let mut score = mat.non_pawn_value(strong_side)
        + mat.get(Pawn, strong_side) as Score * Pawn.score_endgame()
        + push_king_to_edge(ksq_weak)
        + push_close(ksq_strong, ksq_weak);

    if mat.has_piece_side(Queen, strong_side)
        || mat.has_piece_side(Rook, strong_side)
        || (mat.has_piece_side(Bishop, strong_side) && mat.has_piece_side(Bishop, strong_side))
        || ((g.get(Bishop, strong_side) & DARK_SQUARES).is_not_empty()
            && (g.get(Bishop, strong_side) & LIGHT_SQUARES).is_not_empty())
    {
        score = score + KNOWN_WIN_VALUE;
    }

    if strong_side == g.state.side_to_move {
        score
    } else {
        -score
    }
}

fn endgame_kp_vs_k(ts: &Tables, g: &Game, side: Color) -> Score {
    unimplemented!()
}

fn endgame_kbn_vs_k(ts: &Tables, g: &Game, side: Color) -> Score {
    unimplemented!()
}

