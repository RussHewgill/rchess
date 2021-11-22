
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
use crate::hashing::*;

use rustc_hash::FxHashMap;

use crate::game::Castling;

/// mv:         Move,
/// ep:         Option<Coord>,
/// castling:   Castling,
/// halfmove:   u8,
#[derive(Debug,PartialEq,Clone,Copy)]
pub struct GDiff {
    pub mv:         Move,
    pub ep:         Option<Coord>,
    pub castling:   Castling,
    pub halfmove:   u8,
}

impl GDiff {
    pub fn new(g: &Game, mv: Move) -> Self {
        Self {
            mv,
            ep:       g.state.en_passant,
            castling: g.state.castling,
            halfmove: g.halfmove,
        }
    }
}

#[derive(PartialEq,Clone)]
pub struct Game {
    pub state:        GameState,
    pub zobrist:      Zobrist,

    pub stack:        Vec<GDiff>,

    pub last_move:    Option<Move>,
    pub history:      FxHashMap<Zobrist, u8>,
    pub halfmove:     u8,

}

impl Default for Game {
    fn default() -> Self {
        let history = FxHashMap::default();
        Self {
            state:        GameState::default(),
            zobrist:      Zobrist(0),

            stack:        vec![],

            last_move:    None,
            history,
            halfmove:     0,
        }
    }
}

/// make, unmake
impl Game {

    // #[must_use]
    pub fn make_move(&mut self, ts: &Tables, mv: Move) {
    // pub fn make_move(&mut self, ts: &Tables, mv: Move) -> GameResult<()> {
        let diff = GDiff::new(&self, mv);
        self.stack.push(diff);

        // self.zobrist = self.zobrist.update_move_unchecked(ts, &self, mv);

        self.reset_gameinfo_mut();

        self.make_move_unchecked_mut(ts, mv);

        if let Some(mut k) = self.history.get_mut(&self.zobrist) {
            *k += 1;
        } else {
            self.history.insert(self.zobrist, 1);
        }

        self.update_castles(&ts, mv);
        self.last_move = Some(mv);

        // match self.recalc_gameinfo_mut(ts) {
        //     // Err(win) => panic!("wot"),
        //     Err(win) => Err(win),
        //     Ok(_)    => {
        //         // if self._check_history() {
        //         //     Err(GameEnd::DrawRepetition)
        //         // } else {
        //         // }
        //         Ok(())
        //     },
        // }

        // self
    }

    // #[must_use]
    // pub fn unmake_move(mut self, ts: &Tables) -> Self {
    pub fn unmake_move(&mut self, ts: &Tables) {

        let g2 = self.clone();

        let diff = self.stack.pop().unwrap();

        self.unmake_move_unchecked(ts, &diff);

        match self.recalc_gameinfo_mut(ts) {
            Ok(_)  => {},
            Err(e) => {
                panic!("unmake_move: diff: {:?}\n{:?}\n{:?}\n===\n{:?}", diff, self.to_fen(), g2, self);
            },
        }

        // unimplemented!()
    }
}

/// make
impl Game {

    fn make_move_unchecked_mut(&mut self, ts: &Tables, mv: Move) {

        if mv != Move::NullMove {
            let (side,_) = self.get_at(mv.sq_from()).unwrap();
            if self.state.side_to_move != side {
                panic!("non legal move: {:?}", self);
            }
        }

        self._make_move_unchecked_mut(ts, mv);

        match mv {
            Move::PawnDouble { .. }                   => {},
            _                                         => {
                if let Some(ep) = self.state.en_passant {
                    self.zobrist = self.zobrist.update_ep(&ts, ep);
                }
                self.state.en_passant = None;
            },
        }

        if let Move::EnPassant { capture, .. } = mv {
            self.state.last_capture = Some(capture);
        } else if mv.filter_all_captures() {
            self.state.last_capture = Some(mv.sq_to());
        } else {
            self.state.last_capture = None;
        }

        self.state.side_to_move = !self.state.side_to_move;
        self.zobrist = self.zobrist.update_side_to_move(&ts);

        if mv.is_zeroing() { self.halfmove = 0; } else { self.halfmove += 1; }

        // self.recalc_gameinfo_mut(ts).unwrap();

    }

    fn _make_move_unchecked_mut(&mut self, ts: &Tables, mv: Move) {
        let side = self.state.side_to_move;
        match mv {
            Move::Quiet      { from, to, pc } => {
                self.move_piece_mut_unchecked(ts, from, to, pc, side);
            },
            Move::PawnDouble { from, to } => {
                self.move_piece_mut_unchecked(ts, from, to, Pawn, side);

                let ep = ts.between_exclusive(from, to).bitscan().into();

                if let Some(ep) = self.state.en_passant {
                    // remove previous EP
                    self.zobrist = self.zobrist.update_ep(ts, ep);
                }
                self.state.en_passant = Some(ep);
                // add new EP
                self.zobrist = self.zobrist.update_ep(ts, ep);

            },
            Move::Capture    { from, to, pc, victim } => {
                self.delete_piece_mut_unchecked(ts, to, victim, !side);
                self.move_piece_mut_unchecked(ts, from, to, pc, side);
            },
            Move::EnPassant  { from, to, capture } => {
                self.delete_piece_mut_unchecked(ts, capture, Pawn, !side);
                self.move_piece_mut_unchecked(ts, from, to, Pawn, side);
            },
            Move::Promotion  { from, to, new_piece } => {
                self.delete_piece_mut_unchecked(ts, from, Pawn, side);
                self.insert_piece_mut_unchecked(ts, to, new_piece, side);
            },
            Move::PromotionCapture  { from, to, new_piece, victim } => {
                self.delete_piece_mut_unchecked(ts, from, Pawn, side);
                self.delete_piece_mut_unchecked(ts, to, victim, !side);
                self.insert_piece_mut_unchecked(ts, to, new_piece, side);
            },
            Move::Castle     { from, to, rook_from, rook_to } => {
                self.move_piece_mut_unchecked(ts, from, to, King, side);
                self.move_piece_mut_unchecked(ts, rook_from, rook_to, Rook, side);
            },
            Move::NullMove => {
                // let mut out = self.clone();
                // Some(out)
            }
        }
    }

}

/// unmake
impl Game {

    fn unmake_move_unchecked(&mut self, ts: &Tables, diff: &GDiff) {

        self._unmake_move_unchecked(ts, &diff);

        if let Some(ep) = self.state.en_passant {
            self.zobrist = self.zobrist.update_ep(ts, ep);
        }
        if let Some(ep) = diff.ep {
            self.zobrist = self.zobrist.update_ep(ts, ep);
        }

        self.state.en_passant = diff.ep;
        self.halfmove         = diff.halfmove;
        self.state.castling   = diff.castling;

        self.state.side_to_move = !self.state.side_to_move;
        self.zobrist = self.zobrist.update_side_to_move(&ts);

    }

    fn _unmake_move_unchecked(&mut self, ts: &Tables, diff: &GDiff) {
        let side = !self.state.side_to_move;
        match diff.mv {
            Move::Quiet      { from, to, pc } => {
                self.move_piece_mut_unchecked(ts, to, from, pc, side);
            },
            Move::PawnDouble { from, to } => {
                self.move_piece_mut_unchecked(ts, to, from, Pawn, side);
            },
            Move::Capture    { from, to, pc, victim } => {
                self.move_piece_mut_unchecked(ts, to, from, pc, side);
                self.insert_piece_mut_unchecked(ts, to, victim, !side);
            },
            Move::EnPassant  { from, to, capture } => {
                self.move_piece_mut_unchecked(ts, to, from, Pawn, side);
                self.insert_piece_mut_unchecked(ts, capture, Pawn, !side);
            },
            Move::Promotion  { from, to, new_piece } => {
                self.delete_piece_mut_unchecked(ts, to, new_piece, side);
                self.insert_piece_mut_unchecked(ts, from, Pawn, side);
            },
            Move::PromotionCapture  { from, to, new_piece, victim } => {
                self.delete_piece_mut_unchecked(ts, to, new_piece, side);
                self.insert_piece_mut_unchecked(ts, from, Pawn, side);
                self.insert_piece_mut_unchecked(ts, to, victim, !side);
            },
            Move::Castle     { from, to, rook_from, rook_to } => {
                // self.move_piece_mut_unchecked(ts, to, from, King, side);
                // self.move_piece_mut_unchecked(ts, rook_to, rook_from, Rook, side);
                self.delete_piece_mut_unchecked(ts, to, King, side);
                self.delete_piece_mut_unchecked(ts, rook_to, Rook, side);
                self.insert_piece_mut_unchecked(ts, from, King, side);
                self.insert_piece_mut_unchecked(ts, rook_from, Rook, side);
            },
            Move::NullMove => {
                // let mut out = self.clone();
                // Some(out)
            }
        }
    }

}

/// update info
impl Game {

    pub fn init_gameinfo_mut(&mut self, ts: &Tables) -> GameResult<()> {
        self.state.material = self.count_material();
        Ok(())
    }

    // pub fn count_material(&self) -> [[u8; 5]; 2] {
    pub fn count_material(&self) -> Material {
        const COLS: [Color; 2] = [White,Black];
        // const PCS:  [Piece; 5] = [Pawn,Knight,Bishop,Rook,Queen,];

        // let mut out = [[0; 5]; 2];
        let mut out = [[0; 6]; 2];
        for side in COLS {
            for pc in Piece::iter_pieces() {
                out[side][pc.index()] = self.get(pc, side).popcount() as u8;
            }
        }
        Material { buf: out }
        // out
    }

    pub fn recalc_gameinfo_mut(&mut self, ts: &Tables) -> GameResult<()> {

        let king = self.get(King, self.state.side_to_move);
        if king.is_empty() {
            return Err(GameEnd::Checkmate{ win: !self.state.side_to_move});
        }

        self.state.checkers      = BitBoard::empty();
        self.state.king_blocks_w = BitBoard::empty();
        self.state.king_blocks_b = BitBoard::empty();
        // self.state.pinners       = None;

        self.update_pins_mut(&ts);
        self.update_checkers_mut(&ts);
        self.update_check_block_mut(&ts);
        // self.update_occupied_mut();

        // TODO: game phase
        // self.state.phase = self.game_phase();
        trace!("game phase not set");

        // if self.history.len() > 5 {
        //     self.history.pop_front();
        // }
        // self.history.push_back(self.zobrist);

        Ok(())
    }

    fn reset_gameinfo_mut(&mut self) {
        self.state.checkers      = BitBoard::empty();
        self.state.king_blocks_w = BitBoard::empty();
        self.state.king_blocks_b = BitBoard::empty();
        // self.state.pinners       = None;
    }

    fn update_pins_mut(&mut self, ts: &Tables) {
        // let pw = self.find_pins_absolute(&ts, White);
        // let pb = self.find_pins_absolute(&ts, Black);
        // self.state.pinned = Some((pw,pb));
        let c0 = self.get(King, White);
        if c0.is_empty() {
            panic!("No King? g = {:?}", self);
        }
        let c0 = c0.bitscan().into();
        let (bs_w, ps_b) = self.find_slider_blockers(&ts, c0, White);

        let c1 = self.get(King, Black);
        if c1.is_empty() {
            panic!("No King? g = {:?}", self);
        }
        let c1 = c1.bitscan().into();
        let (bs_b, ps_w) = self.find_slider_blockers(&ts, c1, Black);

        // let bs_w = bs_w & self.get_color(White);
        // let bs_b = bs_b & self.get_color(Black);

        self.state.king_blocks_w = bs_w;
        self.state.king_blocks_b = bs_b;

        // self.state.pinners = Some(ps_b | ps_w);

    }

    fn update_checkers_mut(&mut self, ts: &Tables) {
        // let col = self.state.side_to_move;
        // let p0: Coord = self.get(King, col).bitscan().into();

        // let moves = self.find_attackers_to(&ts, p0);
        // let moves = moves & self.get_color(!col);
        // eprintln!("moves = {:?}", moves);
        let moves = self.find_checkers(&ts, self.state.side_to_move);

        // // XXX: trim this unless needed?
        // let moves = moves | self.find_checkers(&ts, !self.state.side_to_move);

        self.state.checkers = moves;

        // unimplemented!()
    }

    fn update_check_block_mut(&mut self, ts: &Tables) {
        let c0 = self.state.checkers;
        if c0.is_empty() | c0.more_than_one() {
            self.state.check_block_mask = BitBoard::empty();
            return;
        }

        let king = self.get(King, self.state.side_to_move).bitscan();
        let b = ts.between_exclusive(king, c0.bitscan());

        self.state.check_block_mask = b;
    }

    fn update_castles(&mut self, ts: &Tables, mv: Move) {
        match mv {
            Move::Quiet { from, pc, .. } | Move::Capture { from, pc, .. } => {
                match (self.state.side_to_move, pc) {
                    (side, King) => {
                        self.zobrist = self.zobrist.update_castling(&ts, self.state.castling);
                        self.state.castling.set_king(side,false);
                        self.state.castling.set_queen(side,false);
                        self.zobrist = self.zobrist.update_castling(&ts, self.state.castling);
                    }
                    (White, Rook) => {
                        self.zobrist = self.zobrist.update_castling(&ts, self.state.castling);
                        if from == Coord(7,0) { self.state.castling.set_king(White,false); };
                        if from == Coord(0,0) { self.state.castling.set_queen(White,false); };
                        self.zobrist = self.zobrist.update_castling(&ts, self.state.castling);
                    },
                    (Black, Rook) => {
                        self.zobrist = self.zobrist.update_castling(&ts, self.state.castling);
                        if from == Coord(7,7) { self.state.castling.set_king(Black,false); };
                        if from == Coord(0,7) { self.state.castling.set_queen(Black,false); };
                        self.zobrist = self.zobrist.update_castling(&ts, self.state.castling);
                    },
                    _              => {},
                }
            },
            Move::Castle { .. }                       => {
                self.zobrist = self.zobrist.update_castling(&ts, self.state.castling);
                let side = self.state.side_to_move;
                self.state.castling.set_king(side,false);
                self.state.castling.set_queen(side,false);
                self.zobrist = self.zobrist.update_castling(&ts, self.state.castling);
            },
            _ => {},
        }
    }

    fn update_castles2(&self, ts: &Tables, mv: Move, x: &mut Self) {
        match mv {
            Move::Quiet { from, .. } | Move::Capture { from, .. } => {
                match (self.state.side_to_move, self.get_at(from)) {
                    (col, Some((_,King))) => {
                        x.zobrist = x.zobrist.update_castling(&ts, x.state.castling);
                        x.state.castling.set_king(col,false);
                        x.state.castling.set_queen(col,false);
                        x.zobrist = x.zobrist.update_castling(&ts, x.state.castling);
                    }
                    (White, Some((_,Rook))) => {
                        x.zobrist = x.zobrist.update_castling(&ts, x.state.castling);
                        if from == Coord(7,0) { x.state.castling.set_king(White,false); };
                        if from == Coord(0,0) { x.state.castling.set_queen(White,false); };
                        x.zobrist = x.zobrist.update_castling(&ts, x.state.castling);
                    },
                    (Black, Some((_,Rook))) => {
                        x.zobrist = x.zobrist.update_castling(&ts, x.state.castling);
                        if from == Coord(7,7) { x.state.castling.set_king(Black,false); };
                        if from == Coord(0,7) { x.state.castling.set_queen(Black,false); };
                        x.zobrist = x.zobrist.update_castling(&ts, x.state.castling);
                    },
                    _              => {},
                }
            },
            Move::Castle { .. }                       => {
                x.zobrist = x.zobrist.update_castling(&ts, x.state.castling);
                let col = self.state.side_to_move;
                x.state.castling.set_king(col,false);
                x.state.castling.set_queen(col,false);
                // match self.state.side_to_move {
                //     White => {
                //         x.state.castling.set_king(col,false);
                //         x.state.castling.set_queen(col,false);
                //     },
                //     Black => {
                //         // x.state.castling.black_king  = false;
                //         // x.state.castling.black_queen = false;
                //     },
                // }
                x.zobrist = x.zobrist.update_castling(&ts, x.state.castling);
            },
            _ => {},
        }

    }

}

/// Insertion and Deletion of Pieces
impl Game {

    fn move_piece_mut_unchecked(
        &mut self, ts: &Tables, from: Coord, to: Coord, pc: Piece, side: Color) {
        self._delete_piece_mut_unchecked(ts, from, pc, side, false);
        self._insert_piece_mut_unchecked(ts, to, pc, side, false);
    }

    fn insert_piece_mut_unchecked(&mut self, ts: &Tables, at: Coord, pc: Piece, side: Color) {
        self._insert_piece_mut_unchecked(ts, at, pc, side, true);
    }

    fn delete_piece_mut_unchecked(&mut self, ts: &Tables, at: Coord, pc: Piece, side: Color) {
        self._delete_piece_mut_unchecked(ts, at, pc, side, true);
    }

    fn _insert_piece_mut_unchecked(
        &mut self, ts: &Tables, at: Coord, pc: Piece, side: Color, mat: bool) {

        let mut bc = self.get_color_mut(side);
        *bc = bc.set_one(at);

        let mut bp = self.get_piece_mut(pc);
        *bp = bp.set_one(at);

        if mat && pc != King {
            self.state.material.buf[side][pc.index()] += 1;
        }

        self.zobrist = self.zobrist.update_piece(&ts, pc, side, at);
    }

    fn _delete_piece_mut_unchecked(
        &mut self, ts: &Tables, at: Coord, pc: Piece, side: Color, mat: bool) {
        let mut bc = self.get_color_mut(side);
        *bc = bc.set_zero(at);

        let mut bp = self.get_piece_mut(pc);
        *bp = bp.set_zero(at);

        if mat && pc != King {
            assert!(self.state.material.buf[side][pc.index()] > 0);
            self.state.material.buf[side][pc.index()] -= 1;
        }

        self.zobrist = self.zobrist.update_piece(&ts, pc, side, at);
    }

    pub fn insert_piece_mut_unchecked_nohash(&mut self, at: Coord, pc: Piece, side: Color) {
        let mut bc = self.get_color_mut(side);
        *bc = bc.set_one(at);

        let mut bp = self.get_piece_mut(pc);
        *bp = bp.set_one(at);
    }

}

// /// get_at
// impl Game {
//     pub fn get_at(&self, c0: Coord) -> Option<(Color, Piece)> {
//         let b0 = BitBoard::single(c0);
//         // if (self.all_occupied() & b0).is_empty() { return None; }
//         let color = if (b0 & self.get_color(White)).is_not_empty() { White } else { Black };
//         if (b0 & self.state.pawns).is_not_empty()   { return Some((color,Pawn)); }
//         else if (b0 & self.state.knights).is_not_empty() { return Some((color,Knight)); }
//         else if (b0 & self.state.bishops).is_not_empty() { return Some((color,Bishop)); }
//         else if (b0 & self.state.rooks).is_not_empty()   { return Some((color,Rook)); }
//         else if (b0 & self.state.queens).is_not_empty()  { return Some((color,Queen)); }
//         else if (b0 & self.state.kings).is_not_empty()   { return Some((color,King)); }
//         None
//     }
// }

// /// get bitboards
// impl Game {

//     pub fn get_color(&self, c: Color) -> BitBoard {
//         match c {
//             White => self.state.white,
//             Black => self.state.black,
//         }
//     }

//     pub fn get_color_mut(&mut self, c: Color) -> &mut BitBoard {
//         match c {
//             White => &mut self.state.white,
//             Black => &mut self.state.black,
//         }
//     }

//     pub fn get_piece(&self, piece: Piece) -> BitBoard {
//         match piece {
//             Pawn   => self.state.pawns,
//             Rook   => self.state.rooks,
//             Knight => self.state.knights,
//             Bishop => self.state.bishops,
//             Queen  => self.state.queens,
//             King   => self.state.kings,
//         }
//     }

//     pub fn get_piece_mut(&mut self, piece: Piece) -> &mut BitBoard {
//         match piece {
//             Pawn   => &mut self.state.pawns,
//             Rook   => &mut self.state.rooks,
//             Knight => &mut self.state.knights,
//             Bishop => &mut self.state.bishops,
//             Queen  => &mut self.state.queens,
//             King   => &mut self.state.kings,
//         }
//     }

//     pub fn get(&self, piece: Piece, col: Color) -> BitBoard {
//         self.get_color(col) & self.get_piece(piece)
//     }

//     pub fn get_pins(&self, col: Color) -> BitBoard {
//         match col {
//             White => self.state.king_blocks_w,
//             Black => self.state.king_blocks_b,
//         }
//     }

//     pub fn all_occupied(&self) -> BitBoard {
//         // self.state.occupied
//         self.state.pawns
//             | self.state.rooks
//             | self.state.knights
//             | self.state.bishops
//             | self.state.queens
//             | self.state.kings
//     }

//     pub fn all_empty(&self) -> BitBoard {
//         !self.all_occupied()
//     }

// }



