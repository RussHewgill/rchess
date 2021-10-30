
use crate::types::*;
use crate::tables::*;

use rayon::prelude::*;

/// Search All
impl Game {

    // pub fn search_other<'a>(&'a self, ts: &'a Tables) -> impl Iterator<Item = Move> + 'a {
    //     unimplemented!();
    //     vec![].into_iter()
    // }

    // pub fn search_captures<'a>(&'a self, ts: &'a Tables) -> impl Iterator<Item = Move> + 'a {
    //     let k = self.search_king_iter(&ts, col);
    //     let b = self.search_sliding_iter(&ts, Bishop, col);
    //     let r = self.search_sliding_iter(&ts, Rook, col);
    //     let q = self.search_sliding_iter(&ts, Queen, col);
    //     unimplemented!();
    //     vec![].into_iter()
    // }

}


/// Sliding
impl Game {

    pub fn search_sliding_iter<'a>(
        &'a self,
        ts:           &'a Tables,
        pc:           Piece,
        col:          Color,
    ) -> impl Iterator<Item = Move> + 'a {
        let pieces = self.get(pc, col);

        let moves = pieces.into_iter().flat_map(move |sq| {
            let ms = self._search_sliding_single(&ts, pc, sq.into(), col, None);
            let sq2: Coord = sq.into();
            // let attacks = moves & self.get_color(!col);
            // let quiets  = moves & self.all_empty();
            let attacks = self.get_color(!col);
            ms.into_iter().map(move |to| {
                if attacks.is_one_at(to) {
                    let to = to.into();
                    let (_,victim) = self.get_at(to).unwrap();
                    Move::Capture { from: sq2, to: to, pc, victim }
                } else {
                    Move::Quiet { from: sq2, to: to.into(), pc }
                }
            })
        });
        moves

    }

}

/// Pawns
impl Game {

    // fn _shift_pawn(sq: u32, side: Color) -> u32 {
    // }

    pub fn search_pawns_iter<'a>(&'a self, ts: &'a Tables, side: Color) -> impl Iterator<Item = Move> + 'a {

        let (r_first, r_last, r_prom) = if side == White { (1,7,6) } else { (6,0,1) };

        let (dir,dw,de) = match side {
            White => (N,NW,NE),
            Black => (S,SW,SE),
        };

        let pawns = self.get(Pawn, side) & !(BitBoard::mask_rank(r_prom));

        let qs = if side == White {
            BitBoard(pawns.0.overflowing_shl(8).0)
        } else {
            BitBoard(pawns.0.overflowing_shr(8).0)
        };
        let quiets = self.all_empty() & qs;

        let doubles = pawns & BitBoard::mask_rank(r_first);
        let doubles = if side == White {
            BitBoard(doubles.0.overflowing_shl(16).0)
        } else {
            BitBoard(doubles.0.overflowing_shr(16).0)
        };

        let quiets = quiets.into_iter()
            .flat_map(move |to| {
                let t = to.into();
                if let Some(f) = (!dir).shift_coord(t) {
                    let m = Move::Quiet { from: f, to: t, pc: Pawn };
                    if self.move_is_legal(&ts, m) { Some(m) } else { None }
                } else { None }
            });

        // pushes.iter_bitscan(|t| {
        //     let t = t.into();
        //     if let Some(f) = (!dir).shift_coord(t) {
        //         // out.push(Move::Quiet { from: f, to: t });
        //         let m = Move::Quiet { from: f, to: t, pc: Pawn };
        //         if self.move_is_legal(&ts, m) { out.push(m); }
        //     }
        // });

        quiets
        // unimplemented!()
        // vec![].into_iter()
    }

}


