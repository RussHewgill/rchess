
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

    // pub fn search_sliding_iter<'a>(
    //     &'a self,
    //     ts:           &'a Tables,
    //     pc:           Piece,
    //     col:          Color,
    // ) -> impl Iterator<Item = Move> + 'a {
    //     let pieces = self.get(pc, col);
    //     let moves = pieces.into_iter().flat_map(move |sq| {
    //         let ms = self._search_sliding_single(&ts, pc, sq.into(), col, None);
    //         let sq2: Coord = sq.into();
    //         // let attacks = moves & self.get_color(!col);
    //         // let quiets  = moves & self.all_empty();
    //         let attacks = self.get_color(!col);
    //         ms.into_iter().map(move |to| {
    //             if attacks.is_one_at(to.into()) {
    //                 let to = to.into();
    //                 let (_,victim) = self.get_at(to).unwrap();
    //                 Move::Capture { from: sq2, to: to, pc, victim }
    //             } else {
    //                 Move::Quiet { from: sq2, to: to.into(), pc }
    //             }
    //         })
    //     });
    //     moves
    // }

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
        let empty = self.all_empty();
        let own   = self.get_color(side);
        let other = self.get_color(!side);

        let pawns = self.get(Pawn, side) & !(BitBoard::mask_rank(r_prom));

        let qs = if side == White {
            BitBoard(pawns.0.overflowing_shl(8).0)
        } else {
            BitBoard(pawns.0.overflowing_shr(8).0)
        };
        let qs = empty & qs;

        let quiets = qs.into_iter()
            // .flat_map(|to| {
            .map(move |to| {
                let t = to.into();
                let f = (!dir).shift_coord_idx_unchecked(to, 1);
                let m = Move::Quiet { from: f.into(), to: t, pc: Pawn };
                m

                // if self.move_is_legal(&ts, m) { Some(m) } else { None }

                // if let Some(f) = (!dir).shift_coord(t) {
                // } else { None }
            });

        let doubles = if side == White {
            BitBoard(qs.0.overflowing_shl(8).0)
        } else {
            BitBoard(qs.0.overflowing_shr(8).0)
        };
        let doubles = empty & doubles;

        let doubles = doubles.into_iter()
            // .flat_map(|to| {
            .map(move |to| {
                let f = (!dir).shift_coord_idx_unchecked(to, 2);
                let m = Move::PawnDouble { from: f.into(), to: to.into() };
                // if self.move_is_legal(&ts, m) { Some(m) } else { None }
                m
            });

        // let captures = (pawns.shift_dir(dw) && other)
        //     | (pawns.shift_dir(de) && other);

        quiets
            .chain(doubles)
            .filter(move |m| self.move_is_legal(&ts, *m))


        // unimplemented!()
        // vec![].into_iter()
    }

}


