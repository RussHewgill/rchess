
use crate::types::*;
use crate::tables::*;

use lazy_static::lazy_static;

// lazy_static! { /// TABLES
//     pub static ref CUCKOO_TABLE: CuckooTable = CuckooTable::new(&_TABLES);
// }

#[derive(Debug,Clone)]
pub struct CuckooTable {
    pub cuckoo:        [Zobrist; 8192],
    pub cuckoo_move:   [Option<(Coord,Coord)>; 8192],
}

impl CuckooTable {

    pub fn get_key(&self, zb: Zobrist) -> Option<usize> {
        let k = Self::h1(zb);
        if self.cuckoo[k] == zb {
            return Some(k);
        }

        let k = Self::h2(zb);
        if self.cuckoo[k] == zb {
            return Some(k);
        }

        // panic!("CuckooTable get_key, missing key?");
        None
    }

}

/// New
impl CuckooTable {

    // pub fn h1(zb: Zobrist) -> Zobrist {
    //     Zobrist(zb.0 & 0x1FFF)
    // }
    // pub fn h2(zb: Zobrist) -> Zobrist {
    //     Zobrist(zb.0.overflowing_shr(16).0 & 0x1FFF)
    // }

    pub fn h1(zb: Zobrist) -> usize {
        (zb.0 & 0x1FFF) as usize
    }
    pub fn h2(zb: Zobrist) -> usize {
        (zb.0.overflowing_shr(16).0 & 0x1FFF) as usize
    }

    pub fn new(ts: &'static Tables) -> Self {

        let zt = &ts.zobrist_tables;

        let mut cuckoo      = [Zobrist::default(); 8192];
        // let mut cuckoo_move = [Move::NullMove; 8192];
        let mut cuckoo_move: [Option<(Coord,Coord)>; 8192] = [None; 8192];
        let mut count       = 0;

        for side in [White,Black] {
            for pc in Piece::iter_pieces() {
                for s1 in 0u8..64 {
                    for s2 in (s1 + 1)..64 {

                        let (s1,s2) = (Coord::new_int(s1), Coord::new_int(s2));
                        let can_move = match pc {
                            Pawn   => false,
                            Knight => {
                                ts.get_knight(s1).is_one_at(s2)
                            },
                            Bishop | Rook | Queen => {
                                ts.get_sliding(pc, s1, BitBoard::empty()).is_one_at(s2)
                            },
                            King   => {
                                ts.get_king(s1).is_one_at(s2)
                            },
                        };

                        if can_move {

                            let mut mv = Some((s1,s2));

                            let key0 = zt.pieces[side][pc][s1]
                                ^ zt.pieces[side][pc][s2]
                                ^ zt.black_to_move;
                            let mut key = Zobrist(key0);

                            let mut i = Self::h1(key);
                            loop {

                                std::mem::swap(&mut cuckoo[i], &mut key);
                                std::mem::swap(&mut cuckoo_move[i], &mut mv);

                                if mv == None { break; } // arrived at empty slot

                                i = if i == Self::h1(key) {
                                    Self::h2(key)
                                } else {
                                    Self::h1(key)
                                };
                            }

                            count += 1;
                        }

                    }
                }
            }
        }

        // eprintln!("count = {:?}", count);
        assert!(count == 3668);

        // let cuckoo_move = array_init::array_init(|x| cuckoo_move[x].unwrap());

        Self {
            cuckoo,
            cuckoo_move,
        }
    }

}




