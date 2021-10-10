
use crate::types::*;

pub use self::movesets::*;

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
pub struct Tables {
    // pub knight_moves: HashMap<Coord, BitBoard>,
    // pub rook_moves:   HashMap<Coord, MoveSetRook>,
    knight_moves: [[BitBoard; 8]; 8],
    rook_moves:   [[MoveSetRook; 8]; 8],
    bishop_moves: [[MoveSetBishop; 8]; 8],
    pawn_moves:   [[MoveSetPawn; 8]; 8],
    king_moves:   [[BitBoard; 8]; 8],
    line_bb:      [[BitBoard; 64]; 64],
    between_bb:   [[BitBoard; 64]; 64],
    // endgames: 
}

/// Piece getters
impl Tables {
    // pub fn get_rook(&self, Coord(x,y): Coord) -> &MoveSetRook {
    pub fn get_rook<T: Into<Coord>>(&self, c: T) -> &MoveSetRook {
        let Coord(x,y) = c.into();
        &self.rook_moves[x as usize][y as usize]
    }
    // pub fn get_bishop(&self, Coord(x,y): Coord) -> &MoveSetBishop {
    pub fn get_bishop<T: Into<Coord>>(&self, c: T) -> &MoveSetBishop {
        let Coord(x,y) = c.into();
        &self.bishop_moves[x as usize][y as usize]
    }
    // pub fn get_knight(&self, Coord(x,y): Coord) -> &BitBoard {
    pub fn get_knight<T: Into<Coord>>(&self, c: T) -> &BitBoard {
        let Coord(x,y) = c.into();
        &self.knight_moves[x as usize][y as usize]
    }
    // pub fn get_pawn(&self, Coord(x,y): Coord) -> &MoveSetPawn {
    pub fn get_pawn<T: Into<Coord>>(&self, c: T) -> &MoveSetPawn {
        let Coord(x,y) = c.into();
        &self.pawn_moves[x as usize][y as usize]
    }
    // pub fn get_king(&self, Coord(x,y): Coord) -> &BitBoard {
    pub fn get_king<T: Into<Coord>>(&self, c: T) -> &BitBoard {
        let Coord(x,y) = c.into();
        &self.king_moves[x as usize][y as usize]
    }
}

impl Tables {

    pub fn new() -> Self {
        let rook_moves   = Self::gen_rooks();
        let bishop_moves = Self::gen_bishops();
        Self {
            knight_moves: Self::gen_knights(),
            rook_moves,
            bishop_moves,
            pawn_moves:   Self::gen_pawns(),
            king_moves:   Self::gen_kings(),
            line_bb:      Self::gen_linebb(bishop_moves),
            between_bb:   Self::gen_betweenbb(bishop_moves),
        }
    }

}

/// Lookup table getters
impl Tables {

    /// Excludes s1, Includes s2
    /// When not aligned, returns s2
    pub fn between<T: Into<Coord>>(&self, s1: T, s2: T) -> BitBoard {
        let (s1,s2): (Coord,Coord) = (s1.into(),s2.into());
        let (s1,s2): (u32,u32) = (s1.into(),s2.into());
        self.between_bb[s1 as usize][s2 as usize]
    }

    pub fn between_exclusive<T: Into<Coord>>(&self, s1: T, s2: T) -> BitBoard {
        let (s1,s2): (Coord,Coord) = (s1.into(),s2.into());
        let (s1,s2): (u32,u32) = (s1.into(),s2.into());
        self.between_bb[s1 as usize][s2 as usize].set_zero(s2.into())
    }

    pub fn line<T: Into<Coord>>(&self, s1: T, s2: T) -> BitBoard {
        let (s1,s2): (Coord,Coord) = (s1.into(),s2.into());
        let (s1,s2): (u32,u32) = (s1.into(),s2.into());
        self.line_bb[s1 as usize][s2 as usize]
    }

    pub fn aligned<T: Into<Coord>>(&self, s1: T, s2: T, s3: T) -> BitBoard {
        self.line(s1, s2) & BitBoard::single(s3.into())
    }

}

/// Lookup table Generators
impl Tables {

    fn gen_betweenbb(bishops: [[MoveSetBishop; 8]; 8]) -> [[BitBoard; 64]; 64] {
        let mut out = [[BitBoard::empty(); 64]; 64];
        for x in 0..64 {
            for y in 0..64 {
                let b = Self::mask_between(bishops, x.into(), y.into());
                let b = b | BitBoard::single(y.into());
                out[x as usize][y as usize] = b;
            }
        }
        out
    }

    fn mask_between(bishops: [[MoveSetBishop; 8]; 8], c0: Coord, c1: Coord) -> BitBoard {

        let Coord(x0,y0) = c0;
        let Coord(x1,y1) = c1;

        if x0 == x1 {
            // File
            let (x0,x1) = (x0.min(x1),x0.max(x1));
            let (y0,y1) = (y0.min(y1),y0.max(y1));
            let b0 = BitBoard::single(Coord(x0,y0));
            let b1 = BitBoard::single(Coord(x1,y1));
            let b = 2u64.overflowing_mul(b1.0).0;
            let b = b.overflowing_sub(b0.0).0;
            let b = BitBoard(b);
            // let b = BitBoard(2 * b1.0 - b0.0);
            let m = BitBoard::mask_file(x0.into());
            (b & m) & !(b0 | b1)
        } else if y0 == y1 {
            // Rank
            let (x0,x1) = (x0.min(x1),x0.max(x1));
            let (y0,y1) = (y0.min(y1),y0.max(y1));
            let b0 = BitBoard::single(Coord(x0,y0));
            let b1 = BitBoard::single(Coord(x1,y1));
            let b = 2u64.overflowing_mul(b1.0).0;
            let b = b.overflowing_sub(b0.0).0;
            let b = BitBoard(b);
            // let b = BitBoard(2 * b1.0 - b0.0);
            let m = BitBoard::mask_rank(y0.into());
            (b & m) & !(b0 | b1)
        // } else if (x1 - x0) == (y1 - y0) {
        } else if (x1 as i64 - x0 as i64).abs() == (y1 as i64 - y0 as i64).abs() {
            // Diagonal
            let b0 = BitBoard::single(Coord(x0,y0));
            let b1 = BitBoard::single(Coord(x1,y1));
            // let b = BitBoard::new(&[Coord(x0,y0),Coord(x1,y1)])

            let (bb0,bb1) = (b0.0.min(b1.0),b0.0.max(b1.0));

            // eprintln!("b0 = {:?}", b0);
            // eprintln!("b1 = {:?}", b1);

            // eprintln!("bb0 = {:?}", b0.bitscan());
            // eprintln!("bb1 = {:?}", b1.bitscan());

            let b = BitBoard(2u64.overflowing_mul(bb1).0.overflowing_sub(bb0).0);
            // let b = BitBoard(2 * b0.0 - b1.0);
            // eprintln!("b = {:?}", b);
            // let m = BitBoard::mask_rank(y0.into());
            let m = {
                let Coord(x,y) = c0.into();
                bishops[x as usize][y as usize]
            };

            let xx = x1 as i64 - x0 as i64;
            let yy = y1 as i64 - y0 as i64;

            let m = if xx.signum() == yy.signum() {
                m.ne | m.sw
            } else {
                m.nw | m.se
            };

            (b & m) & !(b0 | b1)
        } else {
            // println!("wat 2");
            // unimplemented!()
            BitBoard::empty()
        }
    }

    fn gen_linebb(bishops: [[MoveSetBishop; 8]; 8]) -> [[BitBoard; 64]; 64] {
        let mut out = [[BitBoard::empty(); 64]; 64];
        for x in 0u32..64 {
            for y in 0u32..64 {

                let Coord(x0,y0) = x.into();
                let Coord(x1,y1) = y.into();

                let f = BitBoard::mask_file(x0.into());
                let r = BitBoard::mask_rank(y0.into());

                let ds = bishops[x0 as usize][y0 as usize];
                let dp = (ds.ne | ds.sw).set_one(x.into());
                let dn = (ds.nw | ds.se).set_one(x.into());

                if (f & BitBoard::single(y.into())).0 != 0 {
                    out[x as usize][y as usize] = f;
                } else if (r & BitBoard::single(y.into())).0 != 0 {
                    out[x as usize][y as usize] = r;
                } else if (dp & BitBoard::single(y.into())).0 != 0 {
                    out[x as usize][y as usize] = dp;
                } else if (dn & BitBoard::single(y.into())).0 != 0 {
                    out[x as usize][y as usize] = dn;
                }

            }
        }
        out
    }

}

/// Rooks
impl Tables {

    // fn gen_rooks() -> HashMap<Coord, MoveSetRook> {
    fn gen_rooks() -> [[MoveSetRook; 8]; 8] {
        let m0 = MoveSetRook::empty();
        let mut out = [[m0; 8]; 8];

        for y in 0..8 {
            for x in 0..8 {
                out[x as usize][y as usize] = Self::gen_rook_move(Coord(x,y));
            }
        }

        out
    }

    fn gen_rook_move(c: Coord) -> MoveSetRook {

        let sq = BitBoard::index_square(c) as u32;

        let n = Self::rook_n(sq);
        let e = Self::rook_e(sq);
        let s = Self::rook_s(sq);
        let w = Self::rook_w(sq);

        // n | e | s | w
        MoveSetRook { n,e,w,s }
    }

    fn rook_n(sq: u32) -> BitBoard {
        let n0 = 0x0101010101010100u64;
        BitBoard(n0.overflowing_shl(sq).0)
            // & !(BitBoard::mask_file(7))
    }

    fn rook_e(sq: u32) -> BitBoard {
        BitBoard(2 * ( (1u64.overflowing_shl(sq | 7).0) - (1u64.overflowing_shl(sq).0)))
            // & !(BitBoard::mask_rank(0))
    }

    fn rook_s(sq: u32) -> BitBoard {
        let n0 = 0x0080808080808080u64;
        BitBoard(n0.overflowing_shr(sq ^ 63).0)
            // & !(BitBoard::mask_file(7))
    }

    fn rook_w(sq: u32) -> BitBoard {
        BitBoard(1u64.overflowing_shl(sq).0 - 1u64.overflowing_shl(sq & 56).0)
            // & !(BitBoard::mask_rank(0))
    }

}

/// Bishops
impl Tables {

    fn gen_bishops() -> [[MoveSetBishop; 8]; 8] {
        let m0 = MoveSetBishop::empty();
        let mut out = [[m0; 8]; 8];
        for y in 0..8 {
            for x in 0..8 {
                out[x as usize][y as usize] = Self::gen_bishop_move(Coord(x,y));
            }
        }
        out
    }

    pub fn gen_bishop_move(c: Coord) -> MoveSetBishop {
        let sq: u32 = c.into();

        let ne = Self::gen_diagonal(c, true);
        let sw = Self::gen_diagonal(c, false);

        let se = Self::gen_antidiagonal(c, true);
        let nw = Self::gen_antidiagonal(c, false);

        MoveSetBishop {ne, nw, se, sw}
    }

    pub fn gen_diagonal(c0: Coord, positive: bool) -> BitBoard {
        let mut out = BitBoard::single(c0);
        let mut c = c0;
        let d = if positive { NE } else { SW };
        while let Some(k) = d.shift_coord(c) {
            c = k;
            out.flip_mut(c);
        }
        out &= !BitBoard::single(c0);
        out
    }

    pub fn gen_antidiagonal(c0: Coord, positive: bool) -> BitBoard {
        let mut out = BitBoard::single(c0);
        let mut c = c0;
        let d = if positive { SE } else { NW };
        while let Some(k) = d.shift_coord(c) {
            c = k;
            out.flip_mut(c)
        }
        out &= !BitBoard::single(c0);
        out
    }

    // fn gen_antidiagonal(c: Coord, positive: bool) -> BitBoard {
    //     // let mut out = BitBoard::single(c);
    //     let mut out = BitBoard::empty();
    //     if positive {
    //         // out |= out.shift(SE);
    //         for k in c.0..8 {
    //             out.flip_mut(Coord(k+1,c.1));
    //             eprintln!("{:?}\n", out);
    //         }
    //     } else {
    //     }
    //     out
    // }

    // fn gen_diagonal(c: Coord) -> BitBoard {
    //     let v: Vec<Coord> = (0..8).map(|x| Coord(x,x)).collect();
    //     let b0 = BitBoard::new(&v);
    //     if c.0 == c.1 {
    //         b0
    //     } else if c.0 > c.1 {
    //         b0.shift_mult(E, c.0.into())
    //     } else {
    //         b0.shift_mult(N, c.1.into())
    //     }
    // }

    // fn gen_antidiagonal(c: Coord) -> BitBoard {
    //     let v: Vec<Coord> = (0..8).map(|x| Coord(x,7-x)).collect();
    //     let b0 = BitBoard::new(&v);
    //     if (7 - c.0) == c.1 {
    //         b0
    //     } else if (7 - c.0) > c.1 {
    //         b0.shift_mult(W, (7 - c.0).into())
    //     } else {
    //         b0.shift_mult(N, c.1.into())
    //     }
    // }

    fn index_diagonal(Coord(x,y): Coord) -> u8 {
        y.overflowing_sub(x).0 & 15
    }

    fn index_antidiagonal(Coord(x,y): Coord) -> u8 {
        (y + x) ^ 7
    }

    // pub fn gen_bishop_block_mask(c: Coord) -> BitBoard {
    //     unimplemented!()
    // }

    // pub fn gen_bishop_block_board(c: Coord) -> BitBoard {
    //     unimplemented!()
    // }


}

/// Knights
impl Tables {

    // fn gen_knights() -> HashMap<Coord, BitBoard> {
    fn gen_knights() -> [[BitBoard; 8]; 8] {
        let mut out = [[BitBoard::empty(); 8]; 8];

        for y in 0..8 {
            for x in 0..8 {
                out[x as usize][y as usize] = Self::gen_knight_move(Coord(x,y));
            }
        }
        out

        // (0..8).into_iter()
        //     .zip(0..8)
        //     .for_each(|(x,y)| out[x as usize][y as usize] = Self::gen_knight_move(Coord(x,y)));
        // (0..9).into_iter()
        //     .zip(0..9)
        //     .map(|(x,y)| (Coord(x,y), Self::gen_knight_move(Coord(x,y))))
        //     .collect()
    }

    fn gen_knight_move(c: Coord) -> BitBoard {
        let b = BitBoard::new(&vec![c]);

        let l1 = b.0.overflowing_shr(1).0 & !BitBoard::mask_file(7).0;
        let l2 = b.0.overflowing_shr(2).0 & !(BitBoard::mask_file(7).0 | BitBoard::mask_file(6).0);

        let r1 = b.0.overflowing_shl(1).0 & !BitBoard::mask_file(0).0;
        let r2 = b.0.overflowing_shl(2).0 & !(BitBoard::mask_file(0).0 | BitBoard::mask_file(1).0);

        let h1 = l1 | r1;
        let h2 = l2 | r2;

        BitBoard(h1.overflowing_shl(16).0
                 | h1.overflowing_shr(16).0
                 | h2.overflowing_shl(8).0
                 | h2.overflowing_shr(8).0
        )
    }

}

/// Kings
impl Tables {

    pub fn gen_kings() -> [[BitBoard; 8]; 8] {
        let mut out = [[BitBoard::empty(); 8]; 8];

        for y in 0..8 {
            for x in 0..8 {
                out[x as usize][y as usize] = Self::gen_king_move(Coord(x as u8,y as u8));
            }
        }
        out
    }

    fn gen_king_move(c0: Coord) -> BitBoard {
        let b0 = BitBoard::single(c0);
        let b1 = b0
            | b0.shift(W)
            | b0.shift(E);
        let b2 = b1
            | b1.shift(N)
            | b1.shift(S);

        b2 & !b0
    }

}

/// Pawns
impl Tables {

    fn gen_pawns() -> [[MoveSetPawn; 8]; 8] {
        let mut out = [[MoveSetPawn::empty(); 8]; 8];

        for y in 0..8 {
            for x in 0..8 {
                out[x as usize][y as usize] = Self::gen_pawn_move(Coord(x as u8,y as u8));
            }
        }
        out
    }

    fn gen_pawn_move(c0: Coord) -> MoveSetPawn {

        let mut wq = BitBoard::empty();
        if let Some(b) = N.shift_coord(c0) { wq = wq.set_one(b); }

        let mut bq = BitBoard::empty();
        if let Some(b) = S.shift_coord(c0) { bq = bq.set_one(b); }

        let mut wc = BitBoard::empty();
        if let Some(w0) = NE.shift_coord(c0) { wc = wc.set_one(w0); }
        if let Some(w1) = NW.shift_coord(c0) { wc = wc.set_one(w1); }

        let mut bc = BitBoard::empty();
        if let Some(b0) = SE.shift_coord(c0) { bc = bc.set_one(b0); }
        if let Some(b1) = SW.shift_coord(c0) { bc = bc.set_one(b1); }

        MoveSetPawn::new(wq, bq, wc, bc)
    }

}

mod movesets {
    use crate::types::*;

    /// pub n: BitBoard,
    /// pub e: BitBoard,
    /// pub w: BitBoard,
    /// pub s: BitBoard,
    #[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
    pub struct MoveSetRook {
        pub n: BitBoard,
        pub e: BitBoard,
        pub w: BitBoard,
        pub s: BitBoard,
    }

    impl MoveSetRook {
        pub fn to_vec(&self) -> Vec<(D,BitBoard)> {
            vec![(N,self.n),(E,self.e),(W,self.w),(S,self.s)]
        }
        pub fn empty() -> Self {
            Self {
                n: BitBoard::empty(),
                e: BitBoard::empty(),
                w: BitBoard::empty(),
                s: BitBoard::empty(),
            }
        }

        pub fn get_dir(&self, d: D) -> &BitBoard {
            match d {
                N => &self.n,
                E => &self.e,
                W => &self.w,
                S => &self.s,
                _ => panic!("MoveSetRook::get Diagonal rook?")
            }
        }
        pub fn concat(&self) -> BitBoard {
            self.n | self.e | self.w | self.s
        }
    }

    /// pub ne: BitBoard,
    /// pub nw: BitBoard,
    /// pub se: BitBoard,
    /// pub sw: BitBoard,
    #[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
    pub struct MoveSetBishop {
        pub ne: BitBoard,
        pub nw: BitBoard,
        pub se: BitBoard,
        pub sw: BitBoard,
    }

    impl MoveSetBishop {
        pub fn to_vec(&self) -> Vec<(D,BitBoard)> {
            vec![(NE,self.ne),(NW,self.nw),(SE,self.se),(SW,self.sw)]
        }
        pub fn empty() -> Self {
            Self {
                ne: BitBoard::empty(),
                nw: BitBoard::empty(),
                se: BitBoard::empty(),
                sw: BitBoard::empty(),
            }
        }
        pub fn get_dir(&self, d: D) -> &BitBoard {
            match d {
                NE => &self.ne,
                NW => &self.nw,
                SE => &self.se,
                SW => &self.sw,
                _ => panic!("MoveSetBishop::get Rank or File Bishop?")
            }
        }
        pub fn concat(&self) -> BitBoard {
            self.ne | self.nw | self.se | self.sw
        }
    }

    /// pub white_quiet:   BitBoard,
    /// pub black_quiet:   BitBoard,
    /// pub white_capture: BitBoard,
    /// pub black_capture: BitBoard,
    #[derive(Debug,Eq,PartialEq,PartialOrd,Clone,Copy)]
    pub struct MoveSetPawn {
        pub white_quiet:   BitBoard,
        pub black_quiet:   BitBoard,
        pub white_capture: BitBoard,
        pub black_capture: BitBoard,
    }

    impl MoveSetPawn {
        pub fn empty() -> Self {
            Self {
                white_quiet:   BitBoard::empty(),
                black_quiet:   BitBoard::empty(),
                white_capture: BitBoard::empty(),
                black_capture: BitBoard::empty(),
            }
        }
        pub fn new(white_quiet:   BitBoard,
                black_quiet:   BitBoard,
                white_capture: BitBoard,
                black_capture: BitBoard) -> Self {
            Self {
                white_quiet,
                black_quiet,
                white_capture,
                black_capture,
            }
        }
        pub fn get_quiet(&self, c: Color) -> &BitBoard {
            match c {
                White => &self.white_quiet,
                Black => &self.black_quiet,
            }
        }
        pub fn get_capture(&self, c: Color) -> &BitBoard {
            match c {
                White => &self.white_capture,
                Black => &self.black_capture,
            }
        }
    }

}

