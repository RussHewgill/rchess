
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;
use crate::hashing::*;

pub use self::castling::*;
// pub use self::ghistory::*;

use std::collections::{HashMap,VecDeque};
use std::hash::{Hash,Hasher};

// use arrayvec::ArrayVec;
use rustc_hash::FxHashMap;

use serde::{Serialize,Deserialize};

// pub use crate::stack_game::*;

pub type Phase = u8;

// #[derive(PartialEq,Clone)]
#[derive(PartialEq,Clone,Serialize,Deserialize)]
pub struct Game {
    // pub move_history: Vec<Move>,
    pub state:        GameState,
    pub zobrist:      Zobrist,
    pub pawn_zb:      Zobrist,
    pub last_move:    Option<Move>,

    // pub history:      GHistory,
    // pub history:      Vec<(Zobrist,Move)>,
    // pub history:      HashMap<Zobrist, u8>,
    pub history:      FxHashMap<Zobrist, u8>,

    pub halfmove:    u8,
}


#[derive(Debug,Default,PartialOrd,Clone,Copy,Serialize,Deserialize)]
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
    pub material:           Material,

    // pub checkers:           Option<BitBoard>,
    // pub king_blocks_w:      Option<BitBoard>,
    // pub king_blocks_b:      Option<BitBoard>,
    // pub check_block_mask:   Option<BitBoard>,
    pub checkers:           BitBoard,
    pub king_blocks_w:      BitBoard,
    pub king_blocks_b:      BitBoard,
    pub check_block_mask:   BitBoard,
}

impl Default for Game {
    fn default() -> Self {
        let history = FxHashMap::default();

        Self {
            state:        GameState::default(),
            zobrist:      Zobrist(0),
            pawn_zb:      Zobrist(0),
            last_move:    None,
            history,
            halfmove:    0,
            // ..Default::default()
        }
    }
}

#[derive(Default,Hash,Eq,Ord,PartialEq,PartialOrd,Clone,Copy,Serialize,Deserialize)]
pub struct Material {
    pub buf:  [[u8; 6]; 2],
}

/// Construction
impl Material {

    pub fn from_str(s: &str) -> Option<Self> {
        Self::from_ascii(s.as_bytes())
    }

    pub fn from_ascii(s: &[u8]) -> Option<Self> {

        const fn f(c: Color, ch: char) -> (Color,Piece) {
            match ch {
                'P' | 'p' => (c,Pawn),
                'N' | 'n' => (c,Knight),
                'B' | 'b' => (c,Bishop),
                'R' | 'r' => (c,Rook),
                'Q' | 'q' => (c,Queen),
                'K' | 'k' => (c,King),
                _         => unimplemented!(),
            }
        }

        let mut parts = s.splitn(2, |ch| *ch == b'v');
        // let white = parts.next().expect(&format!("wat white {:?}", std::str::from_utf8(&s)));
        // let black = parts.next().expect(&format!("wat white {:?}", std::str::from_utf8(&s)));
        let white = parts.next()?;
        let black = parts.next()?;

        let white = white.iter().map(|&ch| f(White,char::from(ch)));
        let black = black.iter().map(|&ch| f(Black,char::from(ch)));

        Some(Self::from_iter(white.chain(black)))

        // unimplemented!()
    }

    pub fn from_iter<T>(iter: T) -> Self where T: IntoIterator<Item = (Color,Piece)> {
        let mut out = Self::default();
        for (side,pc) in iter {
            out.buf[side][pc] += 1;
        }
        out
    }

    pub fn into_flipped(self) -> Self {
        Self {
            buf: [self.buf[Black], self.buf[White]],
        }
    }

}

/// Queries
impl Material {

    pub fn get(&self, pc: Piece, side: Color) -> u8 {
        self.buf[side][pc]
    }

    pub fn into_normalized(self) -> Self {
        if self.buf[White] < self.buf[Black] {
            Self {
                buf: [self.buf[Black], self.buf[White]],
            }
        } else {
            self
        }
    }

    pub fn count(&self) -> u8 {
        self.buf[White].iter().sum::<u8>() + self.buf[Black].iter().sum::<u8>()
    }

    pub fn count_side(&self, side: Color) -> u8 {
        self.buf[side].iter().sum::<u8>()
    }

    pub fn count_piece(&self, pc: Piece) -> u8 {
        self.buf[White][pc] + self.buf[Black][pc]
    }

    pub fn min_like_man(&self) -> u8 {
        let c0 = Piece::iter_pieces()
            .map(|pc| self.buf[Black][pc.index()]);
        Piece::iter_pieces()
            .map(|pc| self.buf[Black][pc.index()])
            .chain(c0)
            .filter(|&c| 2 <= c)
            .min()
            .unwrap_or(0)
    }

    pub fn is_symmetric(&self) -> bool {
        self.buf[White] == self.buf[Black]
    }

    pub fn unique_pieces(&self) -> u8 {
        self.unique_pieces_side(White) + self.unique_pieces_side(Black)
    }

    pub fn unique_pieces_side(&self, side: Color) -> u8 {
        // self.buf[side].iter().map(|&pc| )
        let mut out = 0;
        for &x in self.buf[side].iter() {
            if x == 1 { out += 1; }
        }
        out
    }

    pub fn has_pawns(&self) -> bool {
        self.has_piece(Pawn)
    }

    pub fn has_piece_side(&self, pc: Piece, side: Color) -> bool {
        self.buf[side][pc.index()] != 0
    }

    pub fn has_piece(&self, pc: Piece) -> bool {
        self.buf[White][pc.index()] != 0 || self.buf[Black][pc.index()] != 0
    }
}

mod castling {
    use crate::types::*;
    use crate::tables::*;

    use serde::{Serialize,Deserialize};

    #[derive(Debug,Hash,Eq,PartialEq,PartialOrd,Clone,Copy,Serialize,Deserialize)]
    pub struct Castling(u8);

    impl Castling {

        const WK: u8 = 0b0001;
        const WQ: u8 = 0b0010;
        const BK: u8 = 0b0100;
        const BQ: u8 = 0b1000;

        pub fn any(&self) -> bool {
            self.0 != 0
        }

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

    // pub fn make_move(&mut self, ts: &Tables, mv: Move) {
    //     unimplemented!()
    // }

    // pub fn unmake_move(&mut self, ts: &Tables) {
    //     unimplemented!()
    // }

    pub fn swap_side_to_move(&mut self, ts: &Tables) {
        self.state.side_to_move = !self.state.side_to_move;
        self.zobrist = self.zobrist.update_side_to_move(&ts);
    }

    pub fn _apply_move_unchecked(&self, ts: &Tables, mv: &Move, calc_zb: bool) -> Option<Game> {
        match mv {
            &Move::Quiet      { from, to, pc } => {
                let (side,pc) = self.get_at(from)?;
                let mut out = self.clone();
                out.move_piece_mut_unchecked(&ts, from, to, pc, side, calc_zb);
                Some(out)
            },
            &Move::PawnDouble { from, to } => {
                let (side,pc) = self.get_at(from)?;
                let mut out = self.clone();
                out.move_piece_mut_unchecked(&ts, from, to, pc, side, calc_zb);

                let ep = ts.between_exclusive(from, to).bitscan().into();
                if calc_zb {
                    if let Some(ep) = out.state.en_passant {
                        // remove previous EP
                        out.zobrist = out.zobrist.update_ep(&ts, ep);
                    }
                    // add new EP
                    out.zobrist = out.zobrist.update_ep(&ts, ep);
                }
                out.state.en_passant = Some(ep);

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
                out.delete_piece_mut_unchecked(&ts, to, victim, !col, calc_zb);
                out.move_piece_mut_unchecked(&ts, from, to, pc, col, calc_zb);
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
                out.delete_piece_mut_unchecked(&ts, capture, Pawn, !col, calc_zb);
                out.move_piece_mut_unchecked(&ts, from, to, Pawn, col, calc_zb);
                Some(out)
            },
            &Move::Promotion  { from, to, new_piece } => {
                let col = self.state.side_to_move;
                // let (c0,pc0) = self.get_at(from)?;
                let mut out = self.clone();
                out.delete_piece_mut_unchecked(&ts, from, Pawn, col, calc_zb);
                out.insert_piece_mut_unchecked(&ts, to, new_piece, col, calc_zb);
                Some(out)
            },
            &Move::PromotionCapture  { from, to, new_piece, victim } => {
                let col = self.state.side_to_move;
                // let (c0,pc0) = self.get_at(from)?;
                // let (c1,pc1) = self.get_at(to)?;
                let mut out = self.clone();
                out.delete_piece_mut_unchecked(&ts, from, Pawn, col, calc_zb);
                out.delete_piece_mut_unchecked(&ts, to, victim, !col, calc_zb);
                out.insert_piece_mut_unchecked(&ts, to, new_piece, col, calc_zb);
                Some(out)
            },
            &Move::Castle     { from, to, rook_from, rook_to } => {
                let mut out = self.clone();
                let col = self.state.side_to_move;
                out.delete_piece_mut_unchecked(&ts, from, King, col, calc_zb);
                out.delete_piece_mut_unchecked(&ts, rook_from, Rook, col, calc_zb);
                out.insert_pieces_mut_unchecked(&ts, &[(to,King,col),(rook_to,Rook,col)], true, calc_zb);
                Some(out)
            },
            &Move::NullMove => {
                let mut out = self.clone();
                Some(out)
            }
        }
    }

    #[must_use]
    pub fn make_move_unchecked(&self, ts: &Tables, mv: Move) -> GameResult<Game> {
        self._make_move_unchecked(ts, mv, None)
    }

    #[must_use]
    // pub fn make_move_unchecked(&self, ts: &Tables, m: &Move) -> Option<Self> {
    pub fn _make_move_unchecked(
        &self,
        ts:          &Tables,
        mv:          Move,
        use_zb:      Option<Zobrist>,
    ) -> GameResult<Game> {
        let calc_zb = use_zb.is_none();

        if mv != Move::NullMove {
            match self.get_at(mv.sq_from()) {
                Some((side,_)) => if self.state.side_to_move != side {
                    trace!("non legal move: {:?}\n{:?}\n{:?}", mv, self.to_fen(), self);
                    // return Err(GameEnd::Error);
                    // panic!();
                    return Err(GameEnd::Error);
                },
                None => {
                    trace!("non legal move, no piece?: {:?}\n{:?}\n{:?}", mv, self.to_fen(), self);
                    // return Err(GameEnd::Error);
                    // panic!();
                    return Err(GameEnd::Stalemate);
                }
            }
        }

        match self._apply_move_unchecked(&ts, &mv, calc_zb) {
            Some(mut next) => {
                match mv {
                    Move::PawnDouble { .. }                   => {
                    },
                    _                                         => {
                        if calc_zb {
                            if let Some(ep) = next.state.en_passant {
                                next.zobrist = next.zobrist.update_ep(&ts, ep);
                            }
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
                if calc_zb { next.zobrist = next.zobrist.update_side_to_move(&ts); }
                // x.move_history.push(*m);
                next.reset_gameinfo_mut();

                self.update_castles(&ts, mv, &mut next, calc_zb);

                next.last_move = Some(mv);

                // // XXX: current or prev Zobrist ??
                // next.history.push((next.zobrist, mv));

                if let Some(zb) = use_zb {
                    next.zobrist = zb;
                };

                if let Some(mut k) = next.history.get_mut(&next.zobrist) {
                    *k += 1;
                    // if *k >= 2 { return Err(GameEnd::DrawRepetition); }
                } else {
                    next.history.insert(next.zobrist, 1);
                }

                if mv.is_zeroing() {
                    next.halfmove = 0;
                } else {
                    next.halfmove += 1;
                }

                match next.recalc_gameinfo_mut(&ts) {
                    // Err(win) => panic!("wot"),
                    Err(win) => Err(win),
                    Ok(_)    => {
                        // if self._check_history() {
                        //     Err(GameEnd::DrawRepetition)
                        // } else {
                        // }
                        Ok(next)
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

    // /// [m-4, m-3, m-2, m-1, m]
    // /// draw when:
    // ///     m == m-2
    // fn _check_history(&self) -> bool {

    //     // // self.history
    //     // let m0 = self.history.get_at(0);
    //     // let m2 = self.history.get_at(2);

    //     // unimplemented!()
    //     false
    // }

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

        // self.state.phase = self.game_phase();

        if let Some(mv) = self.last_move {
            if mv.filter_all_captures() {
                self.state.phase = self.game_phase();
            }
        } else {
            self.state.phase = self.game_phase();
        }

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

    fn update_pins_mut2(&mut self, ts: &Tables) {
        let side = self.state.side_to_move;
        let c0: Coord = self.get(King, side).bitscan().into();
        let bs = self.find_slider_blockers(&ts, c0, side);

        match side {
            White => self.state.king_blocks_w = bs,
            Black => self.state.king_blocks_b = bs,
        }
    }

    // fn _old_update_pins_mut(&mut self, ts: &Tables) {
    fn update_pins_mut(&mut self, ts: &Tables) {

        let c0 = self.get(King, White);
        if c0.is_empty() {
            panic!("No King? g = {:?}", self);
        }
        let c0 = c0.bitscan().into();
        // let (bs_w, ps_b) = self.find_slider_blockers(&ts, c0, White);
        let bs_w = self.find_slider_blockers(&ts, c0, White);

        let c1 = self.get(King, Black);
        if c1.is_empty() {
            panic!("No King? g = {:?}", self);
        }
        let c1 = c1.bitscan().into();
        let bs_b = self.find_slider_blockers(&ts, c1, Black);

        self.state.king_blocks_w = bs_w;
        self.state.king_blocks_b = bs_b;

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

    fn update_castles(&self, ts: &Tables, m: Move, x: &mut Self, calc_zb: bool) {
        match m {
            Move::Quiet { from, pc, .. } | Move::Capture { from, pc, .. } => {
                match (self.state.side_to_move, pc) {
                    (col, King) => {
                        if calc_zb { x.zobrist = x.zobrist.update_castling(&ts, x.state.castling); }
                        x.state.castling.set_king(col,false);
                        x.state.castling.set_queen(col,false);
                        if calc_zb { x.zobrist = x.zobrist.update_castling(&ts, x.state.castling); }
                    }
                    (White, Rook) => {
                        if calc_zb { x.zobrist = x.zobrist.update_castling(&ts, x.state.castling); }
                        if from == Coord(7,0) { x.state.castling.set_king(White,false); };
                        if from == Coord(0,0) { x.state.castling.set_queen(White,false); };
                        if calc_zb { x.zobrist = x.zobrist.update_castling(&ts, x.state.castling); }
                    },
                    (Black, Rook) => {
                        if calc_zb { x.zobrist = x.zobrist.update_castling(&ts, x.state.castling); }
                        if from == Coord(7,7) { x.state.castling.set_king(Black,false); };
                        if from == Coord(0,7) { x.state.castling.set_queen(Black,false); };
                        if calc_zb { x.zobrist = x.zobrist.update_castling(&ts, x.state.castling); }
                    },
                    _              => {},
                }
            },
            Move::Castle { .. }                       => {
                if calc_zb { x.zobrist = x.zobrist.update_castling(&ts, x.state.castling); }
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
                if calc_zb { x.zobrist = x.zobrist.update_castling(&ts, x.state.castling); }
            },
            _ => {},
        }

    }

}

/// Insertion and Deletion of Pieces
impl Game {

    fn move_piece_mut_unchecked<T: Into<Coord>>(
        &mut self, ts: &Tables, from: T, to: T, pc: Piece, side: Color, calc_zb: bool) {
        self._delete_piece_mut_unchecked(&ts, from, pc, side, false, calc_zb);
        self._insert_piece_mut_unchecked(&ts, to, pc, side, false, calc_zb);
    }

    fn delete_piece_mut_unchecked<T: Into<Coord>>(
        &mut self, ts: &Tables, at: T, pc: Piece, side: Color, calc_zb: bool) {
        self._delete_piece_mut_unchecked(&ts, at, pc, side, true, calc_zb);
    }

    fn _delete_piece_mut_unchecked<T: Into<Coord>>(
        &mut self, ts: &Tables, at: T, pc: Piece, side: Color, mat: bool, calc_zb: bool) {
        let at = at.into();

        let mut bc = self.get_color_mut(side);
        *bc = bc.set_zero(at);

        let mut bp = self.get_piece_mut(pc);
        *bp = bp.set_zero(at);

        if mat && pc != King {
            assert!(self.state.material.buf[side][pc.index()] > 0);
            self.state.material.buf[side][pc.index()] -= 1;
        }

        if calc_zb {
            self.zobrist = self.zobrist.update_piece(&ts, pc, side, at.into());
            if pc == Pawn {
                self.pawn_zb = self.pawn_zb.update_piece(ts, pc, side, at.into())
            }
        }
    }

    pub fn insert_piece_mut_unchecked<T: Into<Coord>>(
        &mut self, ts: &Tables, at: T, pc: Piece, side: Color, calc_zb: bool) {
        self._insert_piece_mut_unchecked(&ts, at, pc, side, true, calc_zb);
    }

    pub fn _insert_piece_mut_unchecked<T: Into<Coord>>(
        &mut self, ts: &Tables, at: T, pc: Piece, side: Color, mat: bool, calc_zb: bool) {
        let at = at.into();

        let mut bc = self.get_color_mut(side);
        *bc = bc.set_one(at);

        let mut bp = self.get_piece_mut(pc);
        *bp = bp.set_one(at);

        if mat && pc != King {
            self.state.material.buf[side][pc.index()] += 1;
        }

        if calc_zb {
            self.zobrist = self.zobrist.update_piece(&ts, pc, side, at.into());
            if pc == Pawn {
                self.pawn_zb = self.pawn_zb.update_piece(ts, pc, side, at.into())
            }
        }
    }

    pub fn insert_pieces_mut_unchecked<T: Into<Coord> + Clone + Copy>(
        &mut self, ts: &Tables, ps: &[(T, Piece, Color)], mat: bool, calc_zb: bool) {
        for (at,pc,side) in ps.iter() {
            self._insert_piece_mut_unchecked(&ts, *at, *pc, *side, mat, calc_zb);
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
        // unimplemented!()
    }

    pub fn convert_move(&self, from: &str, to: &str, other: &str) -> Option<Move> {
        let from: Coord = from.into();
        let to: Coord = to.into();
        self._convert_move(from, to, other, false)
    }

    pub fn _convert_move(&self, from: Coord, to: Coord, other: &str, ob_castle: bool) -> Option<Move> {
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
                if col0 == col1 {
                    if ob_castle && pc0 == King && pc1 == Rook && col0 == col1 {

                        let king_to = match to {
                            Coord(0,0) => Coord(2,0),
                            Coord(7,0) => Coord(6,0),
                            Coord(0,7) => Coord(2,7),
                            Coord(7,7) => Coord(6,7),
                            _          =>
                                panic!("polyglot castle king_to ??: ({:?},{:?})",
                                       from, to,
                                ),
                        };
                        let rook_from = to;
                        let rook_to   = match to {
                            Coord(0,0) => Coord(3,0),
                            Coord(7,0) => Coord(5,0),
                            Coord(0,7) => Coord(0,7),
                            Coord(7,7) => Coord(0,7),
                            _          =>
                                panic!("polyglot castle rook_to ??: ({:?},{:?})",
                                    from, to,
                                ),
                        };

                        // panic!("convert move polyglot castle");

                        return Some(Move::Castle { from, to: king_to, rook_from, rook_to });

                    } else {
                        panic!("self capture?: {:?}->{:?}\n{:?}", from, to, self);
                    }
                }

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
            _ => panic!("_convert_move: from: {:?}, to: {:?}, other: {:?}\n{:?}", from, to, other, self),
        }
    }

    pub fn convert_from_algebraic(&self, ts: &Tables, mv: &str) -> Option<Move> {
        let bs = mv.as_bytes();
        let side = self.state.side_to_move;
        if mv == "O-O" {
            let from = if side == White { Coord(4,0) } else { Coord(4,7) };
            let to   = if side == White { Coord(6,0) } else { Coord(6,7) };
            let rook_from = if side == White { Coord(7,0) } else { Coord(7,7) };
            let rook_to   = if side == White { Coord(5,0) } else { Coord(5,7) };
            Some(Move::Castle {
                from,
                to,
                rook_from,
                rook_to,
            })
        } else if mv == "O-O-O" {
            let from = if side == White { Coord(4,0) } else { Coord(4,7) };
            let to   = if side == White { Coord(2,0) } else { Coord(2,7) };
            let rook_from = if side == White { Coord(0,0) } else { Coord(0,7) };
            let rook_to   = if side == White { Coord(3,0) } else { Coord(3,7) };
            Some(Move::Castle {
                from,
                to,
                rook_from,
                rook_to,
            })
        } else if bs[0].is_ascii_lowercase() {
            // pawn move
            if bs[1] as char == 'x' {
                // pawn capture

                let to = &bs[2..4];

                let f = to[0] - 97;
                let r = to[1] - 49;

                let to = Coord(f, r);

                let from = (if side == White { S } else { N }).shift_coord(to).unwrap();
                let from = Coord(bs[0] - 97,from.rank());

                if self.state.en_passant == Some(to) {
                    let capture = (if side == White { S } else { N }).shift_coord(to).unwrap();
                    return Some(Move::EnPassant { from, to, capture });
                }

                let (_,victim) = self.get_at(to).unwrap_or_else(|| {
                    panic!("no victim? {:?}\n{:?}\n{:?}", self.to_fen(), self, mv);
                });

                if bs.get(4) == Some(&('=' as u8)) {
                    let new_piece = Piece::from_char(bs[5] as char);
                    Some(Move::PromotionCapture { from, to, new_piece, victim })
                } else {
                    Some(Move::new_capture(from, to, Pawn, victim))
                }
            } else {
                let to = &bs[0..2];

                let f = to[0] - 97;
                let r = to[1] - 49;

                let to = Coord(f, r);

                let from = (if side == White { S } else { N }).shift_coord(to).unwrap();

                if let Some((_,Pawn)) = self.get_at(from) {
                    if bs.get(2) == Some(&('=' as u8)) {
                        let new_piece = Piece::from_char(bs[3] as char);
                        Some(Move::Promotion { from, to, new_piece })
                    } else {
                        Some(Move::new_quiet(from, to, Pawn))
                    }
                } else {
                    let from = (if side == White { S } else { N }).shift_coord(from).unwrap();
                    Some(Move::new_double(from, to))
                }

            }
        } else {
            let pc = match bs[0] as char {
                'N' => Knight,
                'B' => Bishop,
                'R' => Rook,
                'Q' => Queen,
                'K' => King,
                _   => unimplemented!(),
            };

            let to = if bs[bs.len() - 2] as char == '=' {
                panic!("non pawn promotion? {:?}\n{:?}\n{:?}", self.to_fen(), self, mv);
            } else {
                let to = &bs[bs.len()-2..];
                let f = to[0] - 97;
                let r = to[1] - 49;
                Coord(f, r)
            };

            let mut mvs = self.search_for_piece(&ts, pc, side, false);

            mvs.retain(|mv| mv.sq_to() == to);

            if mvs.len() == 0 {
                panic!("wat: {:?}\n{:?}\n,{:?}", self.to_fen(), self, mv);
            } else if mvs.len() == 1 {
                Some(mvs[0])
            } else {
                let mut cs = vec![];
                for b in bs.iter() {
                    match *b as char {
                        'N' | 'B' | 'R' | 'Q' | 'K' => {},
                        '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' => cs.push(*b),
                        'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | 'h' => cs.push(*b),
                        _ => {},
                    }
                }

                match cs.len() {
                    0 | 1 | 2 => panic!("ambigious move: {:?}", mv),
                    3 => match cs[0] as char {
                        '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' => {
                            mvs.retain(|mv| mv.sq_from().rank() == cs[0] - 49);
                            assert!(mvs.len() == 1);
                            Some(mvs[0])
                        },
                        'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | 'h' => {
                            mvs.retain(|mv| mv.sq_from().file() == cs[0] - 97);
                            assert!(mvs.len() == 1);
                            Some(mvs[0])
                        },
                        _ => panic!(),
                    },
                    4 => {
                        mvs.retain(|mv| {
                            (mv.sq_from().rank() == cs[1] - 49)
                                && (mv.sq_from().file() == cs[0] - 97)
                        });
                        assert!(mvs.len() == 1);
                        Some(mvs[0])
                    }
                    _ => panic!(),
                }
            }
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

    pub fn get(&self, piece: Piece, side: Color) -> BitBoard {
        self.get_color(side) & self.get_piece(piece)
    }

    pub fn get_pins(&self, side: Color) -> BitBoard {

        match side {
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

    pub fn start_pos(ts:  &Tables) -> Self {
        Game::from_fen(ts, STARTPOS).unwrap()
    }

    // pub fn empty() -> Game {
    //     // Game {
    //     //     // move_history: vec![],
    //     //     // state: GameState::empty(),
    //     //     state:        GameState::default(),
    //     //     zobrist:      Zobrist(0),
    //     //     history:      ArrayVec::default(),
    //     // }
    //     Game::default()
    // }

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
        out.zobrist = Zobrist::new(&ts, &out);
        out.pawn_zb = Zobrist::new_pawns(&ts, &out);

        out.init_gameinfo_mut(ts).unwrap();
        out.recalc_gameinfo_mut(ts).unwrap();

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

impl std::fmt::Debug for Material {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let mut white = vec![];
        self.buf[White].iter().enumerate().rev().for_each(|(p,&k)| {
            let pc = Piece::from_index(p as u8);
            for _ in 0..k {
                white.push(pc);
            }
        });
        let mut black = vec![];
        self.buf[Black].iter().enumerate().rev().for_each(|(p,&k)| {
            let pc = Piece::from_index(p as u8);
            for _ in 0..k {
                black.push(pc);
            }
        });

        let white = white.into_iter().map(|pc| {
            pc.print_char().to_ascii_uppercase()
        }).collect::<String>();

        let black = black.into_iter().map(|pc| {
            pc.print_char().to_ascii_uppercase()
        }).collect::<String>();

        f.write_str(&format!("{} v {}", white, black))?;

        Ok(())
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

        // if self.state.checkers.is_not_empty() {
        //     f.write_str(&format!("In Check\n"))?;
        // } else {
        //     f.write_str(&format!("Not In Check\n"))?;
        // }

        f.write_str(&format!("Last Move: {:?}\n", self.last_move))?;
        f.write_str(&format!("En Passant: {:?}\n", self.state.en_passant))?;
        let c = self.state.castling;

        let (wk,wq) = c.get_color(White);
        let (bk,bq) = c.get_color(Black);
        f.write_str(&format!("Castling (KQkq): {} {} {} {}",wk,wq,bk,bq))?;

        // f.write_str(&format!("Moves: \n"))?;
        // let mut k = 0;
        // for m in self.move_history.iter() {
        //     f.write_str(&format!("{:>2}: {:?}\n", k, m))?;
        //     k += 1;
        // }

        Ok(())
    }
}


