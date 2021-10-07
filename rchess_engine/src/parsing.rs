

use crate::types::*;

use std::str::FromStr;
use nom::{
    IResult,
    bytes::complete::tag,
};

impl Game {

    pub fn from_fen(s: &str) -> Option<Game> {

        let (_,ss) = parse_piece_lines(&s).unwrap();
        // eprintln!("ss = {:?}", ss);

        let g = build_from_fen(ss);

        Some(g)
    }

}

fn build_from_fen(v: Vec<Vec<Option<(Piece,Color)>>>) -> Game {
    let mut out = Game::empty();

    for (rank,y) in v.iter().rev().zip(0..8) {
        for (sq,x) in rank.iter().zip(0..8) {
            match sq {
                Some((p,c)) => {
                    out.insert_piece_mut_unchecked(Coord(x,y), *p, *c);
                },
                None => {},
            }
        }
    }
    out
}

fn parse_fen(s: &str) -> IResult<&str, Game> {

    unimplemented!()
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
        nom::character::complete::alphanumeric1
        )(s0)?;

    let mut out = vec![];
    for line in ss.iter() {
        let xs = parse_piece_line(line);
        out.push(xs);
    }
    Ok((s, out))
}

fn parse_piece(c: char) -> Result<(Piece,Color), u8> {
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


