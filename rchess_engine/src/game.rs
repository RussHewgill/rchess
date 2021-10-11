
use crate::types::*;
use crate::tables::*;
use crate::evaluate::*;

#[derive(PartialEq,PartialOrd,Clone)]
pub struct Game {
    pub move_history: Vec<Move>,
    pub state:        GameState
}

// #[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
#[derive(Debug,Default,PartialEq,PartialOrd,Clone,Copy)]
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

    // pub score:              Score,

    pub checkers:           Option<BitBoard>,
    pub king_blocks_w:      Option<BitBoard>,
    pub king_blocks_b:      Option<BitBoard>,
    pub pinners:            Option<BitBoard>,
    // pub pinned:         Option<(BitBoard,BitBoard)>,

    pub check_block_mask:   Option<BitBoard>,
}

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct Castling {
    pub white_queen:   bool,
    pub white_king:    bool,
    pub black_queen:   bool,
    pub black_king:    bool,
}

impl Castling {

    pub fn new_with(w: bool, b: bool) -> Castling {
        Castling {
            white_queen:   w,
            white_king:    w,
            black_queen:   b,
            black_king:    b,
        }
    }

    pub fn get_color(&self, col: Color) -> (bool, bool) {
        match col {
            White => (self.white_king,self.white_queen),
            Black => (self.black_king,self.black_queen),
        }
    }

}

impl Game {

    #[must_use]
    pub fn make_move_unchecked(&self, ts: &Tables, m: &Move) -> Option<Self> {
        let out = match m {
            &Move::Quiet      { from, to } => {
                let (c,pc) = self.get_at(from)?;
                let mut out = self.clone();
                out.delete_piece_mut_unchecked(from, pc, c);
                out.insert_piece_mut_unchecked(to, pc, c);
                Some(out)
            },
            &Move::PawnDouble { from, to } => {
                let (c,pc) = self.get_at(from)?;
                let mut out = self.clone();
                out.delete_piece_mut_unchecked(from, pc, c);
                out.insert_piece_mut_unchecked(to, pc, c);

                out.state.en_passant = Some(ts.between_exclusive(from, to).bitscan().into());

                Some(out)
            },
            &Move::Capture    { from, to } => {
                let (c0,pc0) = self.get_at(from)?;
                let (c1,pc1) = self.get_at(to)?;
                let mut out = self.clone();
                out.delete_piece_mut_unchecked(from, pc0, c0);
                out.delete_piece_mut_unchecked(to, pc1, c1);
                out.insert_piece_mut_unchecked(to, pc0, c0);
                Some(out)
            },
            &Move::EnPassant  { from, to } => {
                let col = self.state.side_to_move;
                let (c0,pc0) = self.get_at(from)?;
                let to1 = if col == White { S.shift_coord(to)? } else { N.shift_coord(to)? };
                let (c1,pc1) = self.get_at(to1)?;
                let mut out = self.clone();
                out.delete_piece_mut_unchecked(from, pc0, c0);
                out.delete_piece_mut_unchecked(to1, pc1, c1);
                out.insert_piece_mut_unchecked(to, pc0, c0);
                Some(out)
            },
            &Move::Promotion  { from, to, new_piece} => {
                let (c0,pc0) = self.get_at(from)?;
                let mut out = self.clone();
                out.delete_piece_mut_unchecked(from, pc0, c0);
                out.insert_piece_mut_unchecked(to, new_piece, c0);
                Some(out)
            },
            &Move::PromotionCapture  { from, to, new_piece} => {
                let (c0,pc0) = self.get_at(from)?;
                let (c1,pc1) = self.get_at(to)?;
                let mut out = self.clone();
                out.delete_piece_mut_unchecked(from, pc0, c0);
                out.delete_piece_mut_unchecked(to, pc1, c1);
                out.insert_piece_mut_unchecked(to, new_piece, c0);
                Some(out)
            },
            &Move::Castle     { from, to, rook_from, rook_to } => {
                let mut out = self.clone();
                let col = self.state.side_to_move;
                out.delete_piece_mut_unchecked(from, King, col);
                out.delete_piece_mut_unchecked(rook_from, Rook, col);
                out.insert_pieces_mut_unchecked(&[(to,King,col),(rook_to,Rook,col)]);
                Some(out)
            },
        };

        if let Some(mut x) = out {
            match m {
                Move::PawnDouble { .. }                   => {},
                _                                         => x.state.en_passant = None,
            }
            match m {
                Move::Quiet { from, .. } | Move::Capture { from, .. } => {
                    match (self.state.side_to_move, self.get_at(*from)) {
                        (White, Some((_,King))) => {
                            x.state.castling.white_king = false;
                            x.state.castling.white_queen = false;
                        }
                        (Black, Some((_,King))) => {
                            x.state.castling.black_king = false;
                            x.state.castling.black_queen = false;
                        }
                        (White, Some((_,Rook))) => {
                            if *from == "H1".into() { x.state.castling.white_king = false; };
                            if *from == "A1".into() { x.state.castling.white_queen = false; };
                        },
                        (Black, Some((_,Rook))) => {
                            if *from == "H8".into() { x.state.castling.black_king = false; };
                            if *from == "A8".into() { x.state.castling.black_queen = false; };
                        },
                        _              => {},
                    }
                },
                Move::Castle { .. }                       => {
                    match self.state.side_to_move {
                        White => {
                            x.state.castling.white_king  = false;
                            x.state.castling.white_queen = false;
                        },
                        Black => {
                            x.state.castling.black_king  = false;
                            x.state.castling.black_queen = false;
                        },
                    }
                },
                _ => {},
            }

            x.state.side_to_move = !x.state.side_to_move;
            x.move_history.push(*m);
            x.reset_gameinfo_mut();
            x.recalc_gameinfo_mut(&ts);
            Some(x)
        } else {
            panic!("Game::make_move?");
        }

    }

    pub fn recalc_gameinfo_mut(&mut self, ts: &Tables) {
        self.state.checkers      = None;
        self.state.king_blocks_w = None;
        self.state.king_blocks_b = None;
        self.state.pinners       = None;

        self.update_pins_mut(&ts);
        self.update_checkers_mut(&ts);
        self.update_check_block_mut(&ts);

    }

    fn reset_gameinfo_mut(&mut self) {
        self.state.checkers      = None;
        self.state.king_blocks_w = None;
        self.state.king_blocks_b = None;
        self.state.pinners       = None;
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

        self.state.king_blocks_w = Some(bs_w);
        self.state.king_blocks_b = Some(bs_b);

        self.state.pinners = Some(ps_b | ps_w);

    }

    fn update_checkers_mut(&mut self, ts: &Tables) {
        // let col = self.state.side_to_move;
        // let p0: Coord = self.get(King, col).bitscan().into();

        // let moves = self.find_attackers_to(&ts, p0);
        // let moves = moves & self.get_color(!col);
        // eprintln!("moves = {:?}", moves);
        let moves = self.find_checkers(&ts, self.state.side_to_move);

        // XXX: trim this unless needed?
        let moves = moves | self.find_checkers(&ts, !self.state.side_to_move);

        self.state.checkers = Some(moves);

        // unimplemented!()
    }

    fn update_check_block_mut(&mut self, ts: &Tables) {
        let c0 = self.state.checkers.unwrap();
        if c0.is_empty() | c0.more_than_one() {
            self.state.check_block_mask = Some(BitBoard::empty());
            return;
        }

        let king = self.get(King, self.state.side_to_move).bitscan();
        let b = ts.between_exclusive(king, c0.bitscan());

        self.state.check_block_mask = Some(b);
    }

    // fn update_checkers_mut(&mut self, ts: &Tables) {
    //     unimplemented!()
    // }

}

/// Insertion and Deletion of Pieces
impl Game {

    fn delete_piece_mut_unchecked<T: Into<Coord>>(&mut self, at: T, p: Piece, c: Color) {
        let at = at.into();

        let mut bc = self.get_color_mut(c);
        *bc = bc.set_zero(at);

        let mut bp = self.get_piece_mut(p);
        *bp = bp.set_zero(at);
    }

    pub fn insert_piece_mut_unchecked<T: Into<Coord>>(&mut self, at: T, p: Piece, c: Color) {
        let at = at.into();

        let mut bc = self.get_color_mut(c);
        *bc = bc.set_one(at);

        let mut bp = self.get_piece_mut(p);
        *bp = bp.set_one(at);
    }

    pub fn insert_pieces_mut_unchecked<T: Into<Coord> + Clone>(&mut self, ps: &[(T, Piece, Color)]) {
        for (at,p,c) in ps.iter() {
            self.insert_piece_mut_unchecked(at.clone(), *p, *c);
        }
    }

}

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

        if let Some(pins) = match col {
            White => self.state.king_blocks_w,
            Black => self.state.king_blocks_b,
        } {
            return pins;
        } else {
            panic!("no pinned BBs?");
        }

        // match self.state.pinned {
        //     None => panic!("no pinned BBs?"),
        //     Some((w,b)) => match col {
        //         White => w,
        //         Black => b,
        //     }
        // }

    }

    pub fn all_occupied(&self) -> BitBoard {
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

impl Game {

    pub fn empty() -> Game {
        Game {
            move_history: vec![],
            // state: GameState::empty(),
            state: GameState::default(),
        }
    }

    pub fn new() -> Game {

        // let mut state = GameState::empty();
        let mut state = GameState::default();

        let mut pawns   = BitBoard::empty();
        pawns |= BitBoard::mask_rank(1) | BitBoard::mask_rank(6);
        state.pawns = pawns;

        let rooks   = BitBoard::new(&vec![
            Coord(0,0),Coord(7,0),Coord(0,7),Coord(7,7),
        ]);
        state.rooks = rooks;

        let knights = BitBoard::new(&vec![
            Coord(1,0),Coord(6,0),Coord(1,7),Coord(6,7),
        ]);
        state.knights = knights;

        let bishops = BitBoard::new(&vec![
            Coord(2,0),Coord(5,0),Coord(2,7),Coord(5,7),
        ]);
        state.bishops = bishops;

        let queens   = BitBoard::new(&vec![Coord(3,0),Coord(3,7)]);
        state.queens = queens;
        let kings    = BitBoard::new(&vec![Coord(4,0),Coord(4,7)]);
        state.kings  = kings;

        let mut white = BitBoard::empty();
        let mut black = BitBoard::empty();

        let k = (!0u8) as u64 | (((!0u8) as u64) << 8);
        white.0 |= k;
        black.0 |= k << (6 * 8);

        white &= pawns | rooks | knights | bishops | queens | kings;
        black &= pawns | rooks | knights | bishops | queens | kings;

        state.side_to_move = White;
        state.castling = Castling::new_with(true, true);

        // let state = GameState {
        //     side_to_move: White,
        //     pawns,
        //     rooks,
        //     knights,
        //     bishops,
        //     queens,
        //     kings,
        //     white,
        //     black,
        //     pinned:     None,
        //     en_passent: None,
        //     castling:   Castling::new_with(true),
        // };

        let mut g = Game {
            move_history: vec![],
            state,
        };
        // g.recalc_gameinfo_mut();
        g
    }

}

impl Game {

    pub fn get_at(&self, c0: Coord) -> Option<(Color, Piece)> {
        let b0 = BitBoard::single(c0);
        if (self.all_occupied() & b0).is_empty() { return None; }
        let color = if (b0 & self.get_color(White)).is_not_empty() { White } else { Black };
        if (b0 & self.state.pawns).is_not_empty()   { return Some((color,Pawn)); }
        if (b0 & self.state.rooks).is_not_empty()   { return Some((color,Rook)); }
        if (b0 & self.state.knights).is_not_empty() { return Some((color,Knight)); }
        if (b0 & self.state.bishops).is_not_empty() { return Some((color,Bishop)); }
        if (b0 & self.state.queens).is_not_empty()  { return Some((color,Queen)); }
        if (b0 & self.state.kings).is_not_empty()   { return Some((color,King)); }
        None
    }

    // pub fn get_at(&self, c: Coord) -> Option<(Color, Piece)> {
    //     let b = BitBoard::empty().flip(c);
    //     if (b & self.all_occupied()).0 == 0 { return None; }
    //     let color = if (b & self.get_color(White)).0 != 0 { White } else { Black };
    //     // eprintln!("color = {:?}", color);
    //     for p in vec![Pawn,Rook,Knight,Bishop,Queen,King].iter() {
    //         if (b & self.get_piece(*p)).0 != 0 {
    //             return Some((color,*p));
    //         }
    //     }
    //     unimplemented!()
    // }

}

fn square_color(Coord(x,y): Coord) -> Color {
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

        if !c.white_king & !c.white_queen & !c.black_king & !c.black_queen { out.push('-'); }

        if c.white_king  { out.push_str(&"K"); }
        if c.white_queen { out.push_str(&"Q"); }
        if c.black_king  { out.push_str(&"k"); }
        if c.black_queen { out.push_str(&"q"); }

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

        f.write_str(&format!("En Passant: {:?}\n", self.state.en_passant))?;
        let c = self.state.castling;
        f.write_str(&format!("Castling (KQkq): {} {} {} {}\n",
                             c.white_king,
                             c.white_queen,
                             c.black_king,
                             c.black_queen,
        ))?;

        f.write_str(&format!("Moves: \n"))?;
        let mut k = 0;
        for m in self.move_history.iter() {
            f.write_str(&format!("{:>2}: {:?}\n", k, m))?;
            k += 1;
        }

        Ok(())
    }
}


