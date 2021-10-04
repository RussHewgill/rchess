
use crate::types::*;



impl Game {

    pub fn search_king(&self, c: Color) -> BitBoard {

        let b0 = self.get(King, c);

        let b1 = b0.shift_unwrapped(D::W);
        let b2 = b0.shift_unwrapped(D::E);

        b0 | b1 | b2
        // unimplemented!()
    }

}




