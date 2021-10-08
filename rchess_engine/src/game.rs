
use crate::types::*;

#[derive(Eq,PartialEq,PartialOrd,Clone)]
pub struct Game {
    pub move_history: Vec<Move>,
    pub state:        GameState
}

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct GameState {
    pub side_to_move: Color,
    // TODO: castling
    // TODO: en passant
    pub pawns:        BitBoard,
    pub rooks:        BitBoard,
    pub knights:      BitBoard,
    pub bishops:      BitBoard,
    pub queens:       BitBoard,
    pub kings:        BitBoard,
    pub white:        BitBoard,
    pub black:        BitBoard,
}

impl Game {
    pub fn make_move_unchecked(&self, m: &Move) -> Option<Self> {
        let out = match m {
            Move::Quiet      { from, to } => {
                let (c,pc) = self.get_at(*from)?;
                let mut out = self.clone();
                out.delete_piece_mut_unchecked(*from, pc, c);
                out.insert_piece_mut_unchecked(*to, pc, c);
                Some(out)
            },
            Move::PawnDouble { from, to } => {
                let (c,pc) = self.get_at(*from)?;
                let mut out = self.clone();
                out.delete_piece_mut_unchecked(*from, pc, c);
                out.insert_piece_mut_unchecked(*to, pc, c);
                Some(out)
            },
            Move::Capture    { from, to } => {
                let (c0,pc0) = self.get_at(*from)?;
                let (c1,pc1) = self.get_at(*to)?;
                let mut out = self.clone();
                out.delete_piece_mut_unchecked(*from, pc0, c0);
                out.delete_piece_mut_unchecked(*to, pc1, c1);
                out.insert_piece_mut_unchecked(*to, pc0, c0);
                Some(out)
            },
            Move::EnPassant  { from, to } => {
                unimplemented!()
            },
            Move::Promotion  { from, to, new_piece} => {
                unimplemented!()
            },
            Move::PromotionCapture  { from, to, new_piece} => {
                unimplemented!()
            },
            Move::Castle     { from, to, rook } => {
                unimplemented!()
            },
        };

        if let Some(mut x) = out {
            x.state.side_to_move = !x.state.side_to_move;
            x.move_history.push(*m);
            Some(x)
        } else {
            panic!("Game::make_move?");
        }

    }

    fn delete_piece_mut_unchecked<T: Into<Coord>>(&mut self, at: T, p: Piece, c: Color) {
        let at = at.into();

        let mut bc = self.get_color_mut(c);
        *bc = bc.set_zero(at);

        let mut bp = self.get_piece_mut(p);
        *bp = bp.set_zero(at);
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

    pub fn get(&self, piece: Piece, c: Color) -> BitBoard {
        self.get_color(c) & self.get_piece(piece)
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

    pub fn empty() -> Game {
        let state = GameState {
            side_to_move: White,
            pawns:        BitBoard::empty(),
            rooks:        BitBoard::empty(),
            knights:      BitBoard::empty(),
            bishops:      BitBoard::empty(),
            queens:       BitBoard::empty(),
            kings:        BitBoard::empty(),
            white:        BitBoard::empty(),
            black:        BitBoard::empty(),
        };
        Game {
            move_history: vec![],
            state,
        }
    }

    pub fn new() -> Game {

        let mut pawns   = BitBoard::empty();
        pawns |= BitBoard::mask_rank(1) | BitBoard::mask_rank(6);

        let rooks   = BitBoard::new(&vec![
            Coord(0,0),Coord(7,0),Coord(0,7),Coord(7,7),
        ]);
        let knights = BitBoard::new(&vec![
            Coord(1,0),Coord(6,0),Coord(1,7),Coord(6,7),
        ]);
        let bishops = BitBoard::new(&vec![
            Coord(2,0),Coord(5,0),Coord(2,7),Coord(5,7),
        ]);

        // let queens  = BitBoard::empty();
        let queens  = BitBoard::new(&vec![Coord(3,0),Coord(3,7)]);
        let kings   = BitBoard::new(&vec![Coord(4,0),Coord(4,7)]);

        let mut white = BitBoard::empty();
        let mut black = BitBoard::empty();

        let k = (!0u8) as u64 | (((!0u8) as u64) << 8);
        white.0 |= k;
        black.0 |= k << (6 * 8);

        white &= pawns | rooks | knights | bishops | queens | kings;
        black &= pawns | rooks | knights | bishops | queens | kings;

        let state = GameState {
            side_to_move: White,
            pawns,
            rooks,
            knights,
            bishops,
            queens,
            kings,
            white,
            black,
        };
        Game {
            move_history: vec![],
            state,
        }
    }

}

impl Game {

    pub fn get_at(&self, c: Coord) -> Option<(Color, Piece)> {
        let b = BitBoard::empty().flip(c);
        if (b & self.all_occupied()).0 == 0 { return None; }

        let color = if (b & self.get_color(White)).0 != 0 { White } else { Black };
        // eprintln!("color = {:?}", color);

        for p in vec![Pawn,Rook,Knight,Bishop,Queen,King].iter() {
            if (b & self.get_piece(*p)).0 != 0 {
                return Some((color,*p));
            }
        }

        unimplemented!()
    }

    // pub fn print(&self) {
    //     let w = char::from_u32(0x25A1).unwrap();
    //     let b = char::from_u32(0x25A0).unwrap();
    //     for y0 in 0..8 {
    //         let y = 7-y0;
    //         let mut line = String::new();
    //         line.push_str(&format!("{}  ", y + 1));
    //         for x in 0..8 {
    //             let ch: char = match self.get_at(Coord(x,y)) {
    //                 Some((c,p)) => p.print(c),
    //                 None        => w,
    //             };
    //             line.push(ch);
    //             line.push(' ');
    //         }
    //         println!("{}", line);
    //     }
    //     let mut line = String::new();
    //     line.push_str(&format!("   "));
    //     let cs = vec!['A','B','C','D','E','F','G','H'];
    //     for x in 0..8 {
    //         line.push_str(&format!("{} ", cs[x]));
    //     }
    //     println!("{}", line);
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

        Ok(())
    }
}


