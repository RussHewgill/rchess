
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
use crate::hashing::*;

pub use self::castling::*;
pub use self::ghistory::*;

use std::collections::VecDeque;
use std::hash::{Hash,Hasher};

// use arrayvec::ArrayVec;
use ringbuffer::ConstGenericRingBuffer;

// #[derive(Default,PartialEq,Clone)]
#[derive(Default,PartialEq,Clone,Copy)]
pub struct Game {
    // pub move_history: Vec<Move>,
    pub state:        GameState,
    pub zobrist:      Zobrist,
    // pub history:      ArrayVec<Zobrist, 5>,
    // pub history:      VecDeque<Zobrist>,
    // pub history:      GHistory,
    pub last_move:    Option<Move>,
}

mod ghistory {
    use ringbuffer::{ConstGenericRingBuffer, RingBuffer, RingBufferExt, RingBufferWrite, RingBufferRead};

    use crate::hashing::Zobrist;

    #[derive(Debug,Default,PartialEq,Clone)]
    pub struct GHistory {
        buf: ConstGenericRingBuffer<Zobrist, 5>,
    }

    impl GHistory {

        pub fn len(&self) -> usize {
            self.buf.len()
        }

        pub fn push_back(&mut self, zb: Zobrist) {
            self.buf.push(zb);
        }

        pub fn pop_front(&mut self) -> Option<Zobrist> {
            self.buf.dequeue()
        }

        pub fn get_at(&self, idx: isize) -> Option<&Zobrist> {
            self.buf.get(idx)
        }

    }

}

// #[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
// #[derive(Debug,Hash,Default,PartialEq,PartialOrd,Clone,Copy)]
#[derive(Debug,Default,PartialOrd,Clone,Copy)]
pub struct GameState {
    pub side_to_move:       Color,

    pub white:              BitBoard,
    pub black:              BitBoard,

    pub pawns:              BitBoard,
    pub rooks:              BitBoard,
    pub knights:            BitBoard,
    pub bishops:            BitBoard,
    pub queens:             BitBoard,
    pub kings:              BitBoard,

    pub en_passant:         Option<Coord>,
    pub castling:           Castling,

    pub phase:              Phase,
    pub last_capture:       Option<Coord>,
    pub material:           [[u8; 5]; 2],

    // pub checkers:           Option<BitBoard>,
    // pub king_blocks_w:      Option<BitBoard>,
    // pub king_blocks_b:      Option<BitBoard>,
    // pub check_block_mask:   Option<BitBoard>,
    pub checkers:           BitBoard,
    pub king_blocks_w:      BitBoard,
    pub king_blocks_b:      BitBoard,
    pub check_block_mask:   BitBoard,
}

pub type Phase = u8;

mod castling {
    use crate::types::*;
    use crate::tables::*;

    #[derive(Debug,Hash,Eq,PartialEq,PartialOrd,Clone,Copy)]
    pub struct Castling(u8);

    impl Castling {

        const WK: u8 = 0b0001;
        const WQ: u8 = 0b0010;
        const BK: u8 = 0b0100;
        const BQ: u8 = 0b1000;

        pub fn get(&self) -> u8 {
            self.0
        }

        pub fn mirror_sides(&self) -> Self {

            let (wk,wq) = self.get_color(White);
            let (bk,bq) = self.get_color(Black);
            let mut out = *self;

            out.set_king(White, bk);
            out.set_king(Black, wk);
            out.set_queen(White, bq);
            out.set_queen(Black, wq);

            out
        }

        pub fn set_king(&mut self, col: Color, b: bool) {
            match (col,b) {
                (White,true)  => { self.0 |= Self::WK; },
                (White,false) => { self.0 &= !Self::WK; },
                (Black,true)  => { self.0 |= Self::BK; },
                (Black,false) => { self.0 &= !Self::BK; },
            }
        }

        pub fn set_queen(&mut self, col: Color, b: bool) {
            match (col,b) {
                (White,true)  => { self.0 |= Self::WQ; },
                (White,false) => { self.0 &= !Self::WQ; },
                (Black,true)  => { self.0 |= Self::BQ; },
                (Black,false) => { self.0 &= !Self::BQ; },
            }
        }

        pub fn get_color(&self, col: Color) -> (bool,bool) {
            match col {
                White => ((self.0 & Self::WK) != 0,(self.0 & Self::WQ) != 0),
                Black => ((self.0 & Self::BK) != 0,(self.0 & Self::BQ) != 0),
            }
        }

        pub fn new(wk: bool, bk: bool, wq: bool, bq: bool) -> Castling {
            let mut out = Castling(0);
            out.set_king(White, wk);
            out.set_king(Black, bk);
            out.set_queen(White, wq);
            out.set_queen(Black, bq);
            out
        }

        pub fn new_with(w: bool, b: bool) -> Castling {
            let mut out = 0;
            if w { out |= Self::WK | Self::WQ; }
            if b { out |= Self::BK | Self::BQ; }
            Castling(out)
        }
    }

    // #[derive(Debug,Hash,Eq,PartialEq,PartialOrd,Clone,Copy)]
    // pub struct Castling {
    //     pub white_queen:   bool,
    //     pub white_king:    bool,
    //     pub black_queen:   bool,
    //     pub black_king:    bool,
    // }

    // impl Castling {
    //     pub fn new_with(w: bool, b: bool) -> Castling {
    //         Castling {
    //             white_queen:   w,
    //             white_king:    w,
    //             black_queen:   b,
    //             black_king:    b,
    //         }
    //     }
    //     pub fn get_color(&self, col: Color) -> (bool, bool) {
    //         match col {
    //             White => (self.white_king,self.white_queen),
    //             Black => (self.black_king,self.black_queen),
    //         }
    //     }
    // }

}

impl GameState {
    pub fn game_equal(&self, other: Self) -> bool {
        (self.side_to_move == other.side_to_move)
            & (self.white == other.white)
            & (self.black == other.black)

            & (self.pawns == other.pawns)
            & (self.rooks == other.rooks)
            & (self.knights == other.knights)
            & (self.bishops == other.bishops)
            & (self.queens == other.queens)
            & (self.kings == other.kings)

            & (self.en_passant == other.en_passant)
            & (self.castling == other.castling)
    }


    pub fn debug_equal(&self, other: Self) {
        println!("side_to_move: {}", self.side_to_move == other.side_to_move);
        println!("white: {}", self.white == other.white);
        println!("black: {}", self.black == other.black);

        println!("pawns: {}", self.pawns == other.pawns);
        println!("rooks: {}", self.rooks == other.rooks);
        println!("knights: {}", self.knights == other.knights);
        println!("bishops: {}", self.bishops == other.bishops);
        println!("queens: {}", self.queens == other.queens);
        println!("kings: {}", self.kings == other.kings);

        println!("en_passant: {}", self.en_passant == other.en_passant);
        println!("castling: {}", self.castling == other.castling);
    }

}

/// make move
impl Game {

    pub fn swap_side_to_move(&mut self, ts: &Tables) {
        self.state.side_to_move = !self.state.side_to_move;
        self.zobrist = self.zobrist.update_side_to_move(&ts);
    }

    pub fn _make_move_unchecked(&self, ts: &Tables, mv: &Move) -> Option<Game> {
        match mv {
            &Move::Quiet      { from, to, pc } => {
                let (side,pc) = self.get_at(from)?;
                let mut out = self.clone();
                out.move_piece_mut_unchecked(&ts, from, to, pc, side);
                Some(out)
            },
            &Move::PawnDouble { from, to } => {
                let (side,pc) = self.get_at(from)?;
                let mut out = self.clone();
                out.move_piece_mut_unchecked(&ts, from, to, pc, side);

                let ep = ts.between_exclusive(from, to).bitscan().into();
                if let Some(ep) = out.state.en_passant {
                    // remove previous EP
                    out.zobrist = out.zobrist.update_ep(&ts, ep);
                }
                out.state.en_passant = Some(ep);
                // add new EP
                out.zobrist = out.zobrist.update_ep(&ts, ep);

                Some(out)
            },
            // &Move::Capture    { from, to } => {
            &Move::Capture    { from, to, pc, victim } => {
                let col = self.state.side_to_move;
                // let (c0,_) = self.get_at(from)?;
                // let (c1,_) = self.get_at(to)?;
                let mut out = self.clone();
                // out.delete_piece_mut_unchecked(&ts, from, pc, col);
                // out.insert_piece_mut_unchecked(&ts, to, pc, col);
                out.delete_piece_mut_unchecked(&ts, to, victim, !col);
                out.move_piece_mut_unchecked(&ts, from, to, pc, col);
                Some(out)
            },
            &Move::EnPassant  { from, to, capture } => {
                let col = self.state.side_to_move;
                // let (c0,pc0) = self.get_at(from)?;
                // let to1 = if col == White { S.shift_coord(to)? } else { N.shift_coord(to)? };
                // let (c1,_) = self.get_at(capture)?;
                let mut out = self.clone();
                // out.delete_piece_mut_unchecked(&ts, from, Pawn, col);
                // out.insert_piece_mut_unchecked(&ts, to, Pawn, col);
                out.delete_piece_mut_unchecked(&ts, capture, Pawn, !col);
                out.move_piece_mut_unchecked(&ts, from, to, Pawn, col);
                Some(out)
            },
            &Move::Promotion  { from, to, new_piece } => {
                let col = self.state.side_to_move;
                // let (c0,pc0) = self.get_at(from)?;
                let mut out = self.clone();
                out.delete_piece_mut_unchecked(&ts, from, Pawn, col);
                out.insert_piece_mut_unchecked(&ts, to, new_piece, col);
                Some(out)
            },
            &Move::PromotionCapture  { from, to, new_piece, victim } => {
                let col = self.state.side_to_move;
                // let (c0,pc0) = self.get_at(from)?;
                // let (c1,pc1) = self.get_at(to)?;
                let mut out = self.clone();
                out.delete_piece_mut_unchecked(&ts, from, Pawn, col);
                out.delete_piece_mut_unchecked(&ts, to, victim, !col);
                out.insert_piece_mut_unchecked(&ts, to, new_piece, col);
                Some(out)
            },
            &Move::Castle     { from, to, rook_from, rook_to } => {
                let mut out = self.clone();
                let col = self.state.side_to_move;
                out.delete_piece_mut_unchecked(&ts, from, King, col);
                out.delete_piece_mut_unchecked(&ts, rook_from, Rook, col);
                out.insert_pieces_mut_unchecked(&ts, &[(to,King,col),(rook_to,Rook,col)], true);
                Some(out)
            },
            &Move::NullMove => {
                let mut out = self.clone();
                Some(out)
            }
        }
    }

    #[must_use]
    // pub fn make_move_unchecked(&self, ts: &Tables, m: &Move) -> Option<Self> {
    pub fn make_move_unchecked(&self, ts: &Tables, mv: Move) -> GameResult<Game> {

        if mv != Move::NullMove {
            let (side,_) = self.get_at(mv.sq_from()).unwrap();
            if self.state.side_to_move != side {
                panic!("non legal move: {:?}", self);
            }
        }

        match self._make_move_unchecked(&ts, &mv) {
            Some(mut next) => {
                match mv {
                    Move::PawnDouble { .. }                   => {
                    },
                    _                                         => {
                        if let Some(ep) = next.state.en_passant {
                            next.zobrist = next.zobrist.update_ep(&ts, ep);
                        }
                        next.state.en_passant = None;
                    },
                }

                if let Move::EnPassant { capture, .. } = mv {
                    next.state.last_capture = Some(capture);
                } else if mv.filter_all_captures() {
                    next.state.last_capture = Some(mv.sq_to());
                } else {
                    next.state.last_capture = None;
                }

                next.state.side_to_move = !next.state.side_to_move;
                next.zobrist = next.zobrist.update_side_to_move(&ts);
                // x.move_history.push(*m);
                next.reset_gameinfo_mut();

                self.update_castles(&ts, mv, &mut next);

                next.last_move = Some(mv);

                match next.recalc_gameinfo_mut(&ts) {
                    // Err(win) => panic!("wot"),
                    Err(win) => Err(win),
                    Ok(_)    => {
                        if self._check_history() {
                            Err(GameEnd::DrawRepetition)
                        } else {
                            Ok(next)
                        }
                    },
                }
            },
            _ => {
                return Err(GameEnd::Error);
            },
        }

        // if let Some(mut x) = self._make_move_unchecked(&ts, &m) {
        // } else {
        // }

    }

    /// [m-4, m-3, m-2, m-1, m]
    /// draw when:
    ///     m == m-2
    fn _check_history(&self) -> bool {

        // // self.history
        // let m0 = self.history.get_at(0);
        // let m2 = self.history.get_at(2);

        // unimplemented!()
        false
    }

}

/// update info
impl Game {

    pub fn init_gameinfo_mut(&mut self, ts: &Tables) -> GameResult<()> {
        self.state.material = self.count_material();
        Ok(())
    }

    pub fn count_material(&self) -> [[u8; 5]; 2] {
        const COLS: [Color; 2] = [White,Black];
        const PCS:  [Piece; 5] = [Pawn,Knight,Bishop,Rook,Queen];

        let mut out = [[0; 5]; 2];
        for side in COLS {
            for pc in PCS {
                out[side][pc.index()] = self.get(pc, side).popcount() as u8;
            }
        }
        out
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

        self.state.phase = self.game_phase();

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

    fn update_castles(&self, ts: &Tables, m: Move, x: &mut Self) {
        match m {
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

    fn move_piece_mut_unchecked<T: Into<Coord>>(
        &mut self, ts: &Tables, from: T, to: T, pc: Piece, side: Color) {
        self._delete_piece_mut_unchecked(&ts, from, pc, side, false);
        self._insert_piece_mut_unchecked(&ts, to, pc, side, false);
    }

    fn delete_piece_mut_unchecked<T: Into<Coord>>(
        &mut self, ts: &Tables, at: T, pc: Piece, side: Color) {
        self._delete_piece_mut_unchecked(&ts, at, pc, side, true);
    }

    fn _delete_piece_mut_unchecked<T: Into<Coord>>(
        &mut self, ts: &Tables, at: T, pc: Piece, side: Color, mat: bool) {
        let at = at.into();

        let mut bc = self.get_color_mut(side);
        *bc = bc.set_zero(at);

        let mut bp = self.get_piece_mut(pc);
        *bp = bp.set_zero(at);

        if mat && pc != King {
            assert!(self.state.material[side][pc.index()] > 0);
            self.state.material[side][pc.index()] -= 1;
        }

        self.zobrist = self.zobrist.update_piece(&ts, pc, side, at.into());
    }

    pub fn insert_piece_mut_unchecked<T: Into<Coord>>(
        &mut self, ts: &Tables, at: T, pc: Piece, side: Color) {
        self._insert_piece_mut_unchecked(&ts, at, pc, side, true);
    }

    pub fn _insert_piece_mut_unchecked<T: Into<Coord>>(
        &mut self, ts: &Tables, at: T, pc: Piece, side: Color, mat: bool) {
        let at = at.into();

        let mut bc = self.get_color_mut(side);
        *bc = bc.set_one(at);

        let mut bp = self.get_piece_mut(pc);
        *bp = bp.set_one(at);

        if mat && pc != King {
            self.state.material[side][pc.index()] += 1;
        }

        self.zobrist = self.zobrist.update_piece(&ts, pc, side, at.into());
    }

    pub fn insert_pieces_mut_unchecked<T: Into<Coord> + Clone + Copy>(
        &mut self, ts: &Tables, ps: &[(T, Piece, Color)], mat: bool) {
        for (at,pc,side) in ps.iter() {
            self._insert_piece_mut_unchecked(&ts, *at, *pc, *side, mat);
        }
    }

    pub fn insert_piece_mut_unchecked_nohash<T: Into<Coord>>(&mut self, at: T, p: Piece, c: Color) {
        let at = at.into();

        let mut bc = self.get_color_mut(c);
        *bc = bc.set_one(at);

        let mut bp = self.get_piece_mut(p);
        *bp = bp.set_one(at);
    }

}

/// Convert Move
impl Game {

    pub fn run_moves(&self, ts: &Tables, moves: Vec<&str>) -> Self {
        let mut g = self.clone();

        for m in moves.into_iter() {
            let from = &m[0..2];
            let to = &m[2..4];
            let other = &m[4..];
            let mm = g.convert_move(from, to, other).unwrap();
            g = g.make_move_unchecked(&ts, mm).unwrap();
        }

        g
    }

    pub fn convert_move(&self, from: &str, to: &str, other: &str) -> Option<Move> {
        let from: Coord = from.into();
        let to: Coord = to.into();
        // eprintln!("from,to = {:?}, {:?}", from, to);
        match (self.get_at(from), self.get_at(to)) {
            (Some((col,pc)),None) => {
                let cc = if col == White { 7 } else { 0 };
                if (pc == King) & (from.file_dist(to) == 2) {
                    // Queenside
                    let (rook_from,rook_to) = if to.0 == 2 {
                        (0,3)
                    } else if to.0 == 6 {
                        (7,5)
                    } else {
                        panic!("bad castle?");
                    };
                    let r = if col == White { 0 } else { 7 };
                    let (rook_from,rook_to) = (Coord(rook_from,r),Coord(rook_to,r));
                    Some(Move::Castle { from, to, rook_from, rook_to })
                } else if pc == Pawn && Some(to) == self.state.en_passant {
                    let capture = if col == White { S.shift_coord(to).unwrap() }
                        else { N.shift_coord(to).unwrap() };
                    Some(Move::EnPassant { from, to, capture })
                } else if (pc == Pawn) && (to.1 == cc) {
                    // XXX: bad
                    let new_piece = Queen;
                    Some(Move::Promotion { from, to, new_piece })
                } else if (pc == Pawn) && SQUAREDIST[from][to] == 2 {
                    Some(Move::PawnDouble { from, to })
                } else {
                    Some(Move::Quiet { from, to, pc })
                }
            },
            (Some((col0,pc0)),Some((col1,pc1))) => {
                if col0 == col1 { panic!("self capture?"); }

                let cc = if col0 == White { 7 } else { 0 };
                if (pc0 == Pawn) & (to.1 == cc) {
                    let (_,victim) = self.get_at(to).unwrap();
                    Some(Move::PromotionCapture { from, to, new_piece: Queen, victim })
                } else {
                    let (_,victim) = self.get_at(to).unwrap();
                    Some(Move::Capture { from, to, pc: pc0, victim })
                }
            },
            (None,None) => None,
            _ => unimplemented!(),
        }
    }
}

/// Misc Queries
impl Game {

    // pub fn iter_all_pieces(&self, side: Color) -> impl Iterator<Item = Piece> {
    pub fn iter_all_pieces(&self, side: Color) -> Vec<(Piece,Coord)> {
        let mut out = vec![];
        for pc in Piece::iter_pieces() {
            self.get(pc, side).into_iter().for_each(|sq| {
                let c0: Coord = sq.into();
                out.push((pc,c0));
            });
        }
        out
    }

    pub fn in_check(&self) -> bool {
        self.state.checkers.is_not_empty()
    }

}

/// get bitboards
impl Game {

    pub fn get_color(&self, c: Color) -> BitBoard {
        match c {
            White => self.state.white,
            Black => self.state.black,
        }
    }

    pub fn get_color_mut(&mut self, c: Color) -> &mut BitBoard {
        match c {
            White => &mut self.state.white,
            Black => &mut self.state.black,
        }
    }

    pub fn get_piece(&self, piece: Piece) -> BitBoard {
        match piece {
            Pawn   => self.state.pawns,
            Rook   => self.state.rooks,
            Knight => self.state.knights,
            Bishop => self.state.bishops,
            Queen  => self.state.queens,
            King   => self.state.kings,
        }
    }

    pub fn get_piece_mut(&mut self, piece: Piece) -> &mut BitBoard {
        match piece {
            Pawn   => &mut self.state.pawns,
            Rook   => &mut self.state.rooks,
            Knight => &mut self.state.knights,
            Bishop => &mut self.state.bishops,
            Queen  => &mut self.state.queens,
            King   => &mut self.state.kings,
        }
    }

    pub fn get(&self, piece: Piece, col: Color) -> BitBoard {
        self.get_color(col) & self.get_piece(piece)
    }

    pub fn get_pins(&self, col: Color) -> BitBoard {

        match col {
            White => self.state.king_blocks_w,
            Black => self.state.king_blocks_b,
        }

        // if let Some(pins) = match col {
        //     White => self.state.king_blocks_w,
        //     Black => self.state.king_blocks_b,
        // } {
        //     return pins;
        // } else {
        //     panic!("no pinned BBs?");
        // }

        // match self.state.pinned {
        //     None => panic!("no pinned BBs?"),
        //     Some((w,b)) => match col {
        //         White => w,
        //         Black => b,
        //     }
        // }

    }

    pub fn all_occupied(&self) -> BitBoard {
        // self.state.occupied
        self.state.pawns
            | self.state.rooks
            | self.state.knights
            | self.state.bishops
            | self.state.queens
            | self.state.kings
    }

    pub fn all_empty(&self) -> BitBoard {
        !self.all_occupied()
    }

}

/// creation
impl Game {

    pub fn empty() -> Game {
        // Game {
        //     // move_history: vec![],
        //     // state: GameState::empty(),
        //     state:        GameState::default(),
        //     zobrist:      Zobrist(0),
        //     history:      ArrayVec::default(),
        // }
        Game::default()
    }

    // pub fn new() -> Game {

    //     // let mut state = GameState::empty();
    //     let mut state = GameState::default();

    //     let mut pawns   = BitBoard::empty();
    //     pawns |= BitBoard::mask_rank(1) | BitBoard::mask_rank(6);
    //     state.pawns = pawns;

    //     let rooks   = BitBoard::new(&vec![
    //         Coord(0,0),Coord(7,0),Coord(0,7),Coord(7,7),
    //     ]);
    //     state.rooks = rooks;

    //     let knights = BitBoard::new(&vec![
    //         Coord(1,0),Coord(6,0),Coord(1,7),Coord(6,7),
    //     ]);
    //     state.knights = knights;

    //     let bishops = BitBoard::new(&vec![
    //         Coord(2,0),Coord(5,0),Coord(2,7),Coord(5,7),
    //     ]);
    //     state.bishops = bishops;

    //     let queens   = BitBoard::new(&vec![Coord(3,0),Coord(3,7)]);
    //     state.queens = queens;
    //     let kings    = BitBoard::new(&vec![Coord(4,0),Coord(4,7)]);
    //     state.kings  = kings;

    //     let mut white = BitBoard::empty();
    //     let mut black = BitBoard::empty();

    //     let k = (!0u8) as u64 | (((!0u8) as u64) << 8);
    //     white.0 |= k;
    //     black.0 |= k << (6 * 8);

    //     white &= pawns | rooks | knights | bishops | queens | kings;
    //     black &= pawns | rooks | knights | bishops | queens | kings;

    //     state.side_to_move = White;
    //     state.castling = Castling::new_with(true, true);

    //     // let state = GameState {
    //     //     side_to_move: White,
    //     //     pawns,
    //     //     rooks,
    //     //     knights,
    //     //     bishops,
    //     //     queens,
    //     //     kings,
    //     //     white,
    //     //     black,
    //     //     pinned:     None,
    //     //     en_passent: None,
    //     //     castling:   Castling::new_with(true),
    //     // };

    //     let mut g = Game {
    //         move_history: vec![],
    //         state,
    //     };
    //     // g.recalc_gameinfo_mut();
    //     g
    // }

}

/// Debugging
impl Game {

    pub fn open_with_lichess(&self) -> std::io::Result<()> {

        let fen = self.to_fen();
        let url = format!("https://lichess.org/analysis/fromPosition/{}", fen);

        open::with(url, "firefox")
    }


    pub fn zobrist_from_fen(ts: &Tables, fen: &str) -> Zobrist {
        let g = Game::from_fen(ts, fen).unwrap();
        g.zobrist
    }

    pub fn flip_sides(&self, ts: &Tables) -> Self {
        let mut st = self.state.clone();
        st.side_to_move = !st.side_to_move;

        let mw = st.white;
        let mb = st.black;
        st.black = mw;
        st.white = mb;

        st.white   = st.white.rotate_180().mirror_horiz();
        st.black   = st.black.rotate_180().mirror_horiz();
        st.pawns   = st.pawns.rotate_180().mirror_horiz();
        st.rooks   = st.rooks.rotate_180().mirror_horiz();
        st.knights = st.knights.rotate_180().mirror_horiz();
        st.bishops = st.bishops.rotate_180().mirror_horiz();
        st.queens  = st.queens.rotate_180().mirror_horiz();
        st.kings   = st.kings.rotate_180().mirror_horiz();

        st.castling = st.castling.mirror_sides();

        let mut out = Game::default();

        out.state = st;
        out.zobrist = Zobrist::new(&ts, out.clone());
        out
    }
}

/// get_at
impl Game {

    pub fn get_at(&self, c0: Coord) -> Option<(Color, Piece)> {
        let b0 = BitBoard::single(c0);
        // if (self.all_occupied() & b0).is_empty() { return None; }
        let color = if (b0 & self.get_color(White)).is_not_empty() { White } else { Black };
        if (b0 & self.state.pawns).is_not_empty()   { return Some((color,Pawn)); }
        else if (b0 & self.state.knights).is_not_empty() { return Some((color,Knight)); }
        else if (b0 & self.state.bishops).is_not_empty() { return Some((color,Bishop)); }
        else if (b0 & self.state.rooks).is_not_empty()   { return Some((color,Rook)); }
        else if (b0 & self.state.queens).is_not_empty()  { return Some((color,Queen)); }
        else if (b0 & self.state.kings).is_not_empty()   { return Some((color,King)); }
        None
    }

}

pub fn square_color(Coord(x,y): Coord) -> Color {
    if y % 2 == 0 {
        if x % 2 == 0 {
            Black
        } else {
            White
        }
    } else {
        if x % 2 == 0 {
            White
        } else {
            Black
        }
    }
}

/// to_fen
impl Game {
    // pub fn show_moveset(&self, moves: BitBoard) 

    pub fn to_fen(&self) -> String {
        let mut out = String::new();

        for y0 in 0..8 {
            let y = 7-y0;

            let pieces = (0..8)
                .map(|x| self.get_at(Coord(x,y)));
                // .collect::<Vec<Option<(Color,Piece)>>>();

            let mut n = 0;
            for pc in pieces {
                match pc {
                    None     => n += 1,
                    Some((col,pc)) => {
                        if n != 0 {
                            out.push_str(&format!("{}", n));
                        }
                        n = 0;
                        let mut c = match pc {
                            Pawn   => 'p',
                            Rook   => 'r',
                            Knight => 'n',
                            Bishop => 'b',
                            Queen  => 'q',
                            King   => 'k',
                        };
                        if col == White { c = c.to_ascii_uppercase(); }
                        out.push_str(&format!("{}", c));
                    },
                }
            }
            if n != 0 {
                out.push_str(&format!("{}", n));
            }
            out.push_str(&"/");
        }
        out.truncate(out.len() - 1);

        let s = if self.state.side_to_move == White { 'w' } else { 'b' };
        out.push_str(&format!(" {} ", s));

        let c = self.state.castling;


        let (wk,wq) = c.get_color(White);
        let (bk,bq) = c.get_color(Black);

        if !wk & !wq & !bk & !bq { out.push('-'); }
        // if !c.white_king & !c.white_queen & !c.black_king & !c.black_queen { out.push('-'); }

        if wk { out.push_str(&"K"); }
        if wq { out.push_str(&"Q"); }
        if bk { out.push_str(&"k"); }
        if bq { out.push_str(&"q"); }

        if let Some(ep) = self.state.en_passant {
            let s = format!(" {:?}", ep);
            out.push_str(&s.to_ascii_lowercase());
        } else {
            out.push_str(&" -");
        }

        // out.push_str(&" 0 ");
        // if self.state.side_to_move == White {
        //     out.push_str(&"");
        // }

        out
        // unimplemented!()
    }

}

impl GameState {

    // pub fn empty() -> GameState {
    //     GameState {
    //         side_to_move: White,

    //         white:        BitBoard::empty(),
    //         black:        BitBoard::empty(),

    //         pawns:        BitBoard::empty(),
    //         rooks:        BitBoard::empty(),
    //         knights:      BitBoard::empty(),
    //         bishops:      BitBoard::empty(),
    //         queens:       BitBoard::empty(),
    //         kings:        BitBoard::empty(),

    //         en_passent:   None,
    //         castling:     Castling::new_with(false),

    //         score:        0.0,
    //         pinned:       None,
    //     }
    // }

}

impl Default for Castling {
    fn default() -> Self { Self::new_with(false, false) }
}

impl PartialEq for GameState {
    fn eq(&self, other: &Self) -> bool {
        (self.side_to_move == other.side_to_move)
            & (self.white == other.white)
            & (self.black == other.black)

            & (self.pawns == other.pawns)
            & (self.rooks == other.rooks)
            & (self.knights == other.knights)
            & (self.bishops == other.bishops)
            & (self.queens == other.queens)
            & (self.kings == other.kings)

            & (self.castling == other.castling)
    }
}

impl Eq for GameState {}

impl Hash for GameState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.side_to_move.hash(state);
        self.white.hash(state);
        self.black.hash(state);

        self.pawns.hash(state);
        self.rooks.hash(state);
        self.knights.hash(state);
        self.bishops.hash(state);
        self.queens.hash(state);
        self.kings.hash(state);

        self.castling.hash(state);
    }
}

impl std::fmt::Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        f.write_str(&format!("{:?} to move\n", self.state.side_to_move))?;

        for y0 in 0..8 {
            let y = 7-y0;
            let mut line = String::new();
            line.push_str(&format!("{}  ", y + 1));
            for x in 0..8 {
                let ch: char = match self.get_at(Coord(x,y)) {
                    Some((c,p)) => p.print(c),
                    None        => {
                        let c = square_color(Coord(x,y));
                        c.print()
                    },
                };
                line.push(ch);
                line.push(' ');
            }
            f.write_str(&format!("{}\n", line))?;
        }
        let mut line = String::new();
        line.push_str(&format!("   "));
        let cs = vec!['A','B','C','D','E','F','G','H'];
        for x in 0..8 {
            line.push_str(&format!("{} ", cs[x]));
        }
        f.write_str(&format!("{}\n", line))?;

        if self.state.checkers.is_not_empty() {
            f.write_str(&format!("In Check\n"))?;
        } else {
            f.write_str(&format!("Not In Check\n"))?;
        }

        f.write_str(&format!("En Passant: {:?}\n", self.state.en_passant))?;
        let c = self.state.castling;

        let (wk,wq) = c.get_color(White);
        let (bk,bq) = c.get_color(Black);

        f.write_str(&format!("Castling (KQkq): {} {} {} {}\n",wk,wq,bk,bq))?;

        // f.write_str(&format!("Moves: \n"))?;
        // let mut k = 0;
        // for m in self.move_history.iter() {
        //     f.write_str(&format!("{:>2}: {:?}\n", k, m))?;
        //     k += 1;
        // }

        Ok(())
    }
}


