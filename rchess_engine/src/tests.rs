
use crate::types::*;
use crate::tables::*;

#[test]
fn test_check_moves_01() {

    let ts = Tables::new();
    let mut g = Game::empty();
    g.state.side_to_move = Black;
    g.insert_pieces_mut_unchecked(&vec![
        ("F5", Rook, White),
        ("E1", King, White),
        ("E8", King, Black),
    ]);

    let moves = g.search_all(&ts, g.state.side_to_move);

    unimplemented!()
}





