
use std::collections::HashMap;

use crate::types::*;
use crate::tables::*;


// pub trait EndGame {

//     fn evaluate(&self, ts: &Tables, g: &Game) -> Score;

//     fn verify_material(g: &Game, side: Color, npm: Score, pawns: u8) -> bool {
//         g.state.material.non_pawn_value(side) == npm
//             && g.state.material.get(Pawn, side) == pawns
//     }

// }

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
    value:          HashMap<Score, EndGame>,
    scalefactors:   HashMap<ScaleFactor, EndGame>,
}

impl EndGameMaps {
    pub fn init() -> Self {
        // Self {}
        unimplemented!()
    }

    pub fn get_value(&self, score: Score) -> Option<EndGame> {
        unimplemented!()
    }

    pub fn get_scalefactor(&self, scale: ScaleFactor) -> Option<EndGame> {
        unimplemented!()
    }

    pub fn get_endgame(g: &Game) -> Option<EndGame> {
        unimplemented!()
    }

}

#[derive(Debug,PartialEq,PartialOrd,Clone,Copy)]
pub enum EndGame {
    KXvK,
    KPvK,
}

impl EndGame {

    fn evaluate(self, ts: &Tables, g: &Game) -> Score {
        match self {
            Self::KXvK => endgame_kx_vs_k(ts, g),
            _          => unimplemented!(),
        }
    }

    fn verify_material(g: &Game, side: Color, npm: Score, pawns: u8) -> bool {
        g.state.material.non_pawn_value(side) == npm
            && g.state.material.get(Pawn, side) == pawns
    }

}

fn endgame_kx_vs_k(ts: &Tables, g: &Game) -> Score {
    unimplemented!()
}

