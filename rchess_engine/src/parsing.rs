
use crate::types::*;
use crate::tables::*;

use std::str::FromStr;
use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::{one_of,alphanumeric1},
};

impl Game {

    pub fn from_fen(ts: &Tables, s: &str) -> Option<Game> {

        let (s,ss) = parse_piece_lines(&s).unwrap();
        // eprintln!("ss = {:?}", ss);

        let (s,side) = parse_side(&s).unwrap();
        // let (s,castle) = parse_castle(&s).unwrap();
        // let (s,ep) = parse_enpassant(&s).unwrap();

        let (s,castle) = parse_castle(&s).unwrap();
        let (s,ep) = parse_enpassant(&s).unwrap();

        let (s,halfmove) = parse_halfmove_fullmove(&s).unwrap();

        let mut g = build_from_fen(ss, side, castle, ep, halfmove);
        // g.recalc_gameinfo_mut();

        let _ = g.recalc_gameinfo_mut(&ts);
        let _ = g.init_gameinfo_mut(&ts);

        g.zobrist = Zobrist::new(&ts, &g);
        g.pawn_zb = Zobrist::new_pawns(ts, &g);

        Some(g)
    }

}
fn build_from_fen(
    v:          Vec<Vec<Option<(Piece,Color)>>>,
    col:        Color,
    castling:   Castling,
    ep:         Option<Coord>,
    halfmove:   Depth,
) -> Game {
    // let mut out = Game::empty();
    let mut out = Game::default();

    for (rank,y) in v.iter().rev().zip(0..8) {
        for (sq,x) in rank.iter().zip(0..8) {
            match sq {
                Some((p,c)) => {
                    out.insert_piece_mut_unchecked_nohash(Coord::new(x,y), *p, *c);
                },
                None => {},
            }
        }
    }
    out.state.side_to_move = col;
    out.state.castling = castling;
    out.state.en_passant = ep;
    out
}

fn parse_halfmove_fullmove(s: &str) -> IResult<&str, Depth> {
    let (s,_) = tag(" ")(s)?;
    let (s,hm) = nom::character::complete::digit1(s)?;
    let (s,_) = tag(" ")(s)?;
    let (s,fm) = nom::character::complete::digit1(s)?;

    let hm: Depth = Depth::from_str(&hm).unwrap();
    let fm: Depth = Depth::from_str(&fm).unwrap();

    Ok((s,hm))
}

fn parse_side(s: &str) -> IResult<&str, Color> {
    let (s,_) = tag(" ")(s)?;
    let (s,c) = one_of("w,b")(s)?;

    match c {
        'w' => Ok((s,White)),
        'b' => Ok((s,Black)),
        _   => panic!(),
    }
}

fn parse_castle(s: &str) -> IResult<&str, Castling> {
    let (s,_) = tag(" ")(s)?;

    let (s,cs) = nom::branch::alt((
        nom::multi::many1(one_of("-")),
        nom::multi::many1(one_of("KQkq")),
    ))(s)?;

    match cs.get(0) {
        Some('-') => Ok((s,Castling::new_with(false, false))),
        None      => panic!("parse_castle?"),
        _         => {
            let mut out = Castling::new_with(false, false);
            if cs.contains(&'K') { out.set_king(White,true); }
            if cs.contains(&'Q') { out.set_queen(White,true); }
            if cs.contains(&'k') { out.set_king(Black, true); }
            if cs.contains(&'q') { out.set_queen(Black,true); }
            Ok((s,out))
        },
    }
}

fn parse_enpassant(s: &str) -> IResult<&str, Option<Coord>> {
    let (s,_) = tag(" ")(s)?;
    let (s,cs) = nom::branch::alt((
        tag("-"),
        alphanumeric1,
    ))(s)?;

    match cs {
        "-" => {
            Ok((s,None))
        },
        _   => {
            Ok((s, Some(Coord::from_str(cs).expect("parse_enpassant"))))
            // unimplemented!()
        },
    }
}

fn parse_piece_line(s: &str) -> Vec<Option<(Piece,Color)>> {
    let mut out = vec![];
    for c in s.chars() {
        let x = match parse_piece(c) {
            Ok((p,c)) => {
                out.push(Some((p,c)));
            },
            Err(n) => {
                for _ in 0..n {
                    out.push(None);
                }
            },
        };
    }
    out
}

fn parse_piece_lines(s0: &str) -> IResult<&str, Vec<Vec<Option<(Piece,Color)>>>> {

    let (s, ss) = nom::multi::separated_list1(
        tag("/"),
        // nom::character::complete::one_of("0123456789PNBRQKpnbrqk"),
        alphanumeric1,
        )(s0)?;

    let mut out = vec![];
    for line in ss.iter() {
        let xs = parse_piece_line(line);
        out.push(xs);
    }
    Ok((s, out))
}

fn parse_piece(c: char) -> std::result::Result<(Piece,Color), u8> {
    // nom::character::complete::one_of("0123456789PNBRQKpnbrqk")(s)
    if c.is_digit(10) {
        Err(u8::from_str(&format!("{}", c)).unwrap())
    } else {
        let col = if c.is_ascii_uppercase() { White } else { Black };
        let p = match c.to_ascii_lowercase() {
            'p' => Pawn,
            'n' => Knight,
            'b' => Bishop,
            'r' => Rook,
            'q' => Queen,
            'k' => King,
            _ => panic!(),
        };
        Ok((p,col))
    }
}


