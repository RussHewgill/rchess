
use crate::types::*;

impl Game {

    pub fn search_king(&self, c: Color) -> BitBoard {
        let b0 = self.get(King, c);
        let b1 = b0
            | b0.shift(D::W)
            | b0.shift(D::E);
        let b2 = b1
            | b1.shift(D::N)
            | b1.shift(D::S);
        b2
    }

    // pub fn search_knight(&self, )

    pub fn search_pawns(&self, c: Color) -> Vec<Move> {
        let ps = self.get(Pawn, c);

        let pushes = ps.shift(D::N);

        unimplemented!()
    }

}




