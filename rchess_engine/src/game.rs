
use crate::types::*;

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone)]
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

    pub fn get_color(&self, c: Color) -> BitBoard {
        match c {
            White => self.state.white,
            Black => self.state.black,
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

    pub fn new() -> Game {

        let mut pawns   = BitBoard::empty();
        pawns |= BitBoard::mask_rank(1) | BitBoard::mask_rank(6);

        let mut rooks   = BitBoard::empty();
        let mut knights = BitBoard::empty();
        let mut bishops = BitBoard::empty();

        let queens  = BitBoard::empty();
        // let queens  = BitBoard::new(&vec![Coord(3,0),Coord(3,7)]);
        let kings   = BitBoard::new(&vec![Coord(4,0),Coord(4,7)]);

        let mut white = BitBoard::empty();
        let mut black = BitBoard::empty();
        // for x in 0..8 {
        //     for y in 0..3 {
        //         white.flip_mut(Coord(x,y));
        //     }
        //     for y in 6..8 {
        //         black.flip_mut(Coord(x,y));
        //     }
        // }

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
