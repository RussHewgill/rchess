
use crate::brain::trainer::TDOutcome;
use crate::brain::trainer::TrainingData;
use crate::types::*;
use crate::tables::*;

use nom::bytes::streaming::take_till;
use nom::bytes::streaming::take_while;
use nom::bytes::streaming::take_while1;
use nom::character::complete::multispace0;
use nom::character::is_newline;
use nom::character::streaming::char;
use nom::character::streaming::anychar;
use nom::character::streaming::digit1;
use nom::character::streaming::line_ending;
use nom::character::streaming::none_of;
use nom::character::streaming::not_line_ending;
use nom::character::streaming::space0;
use nom::multi::many0;
use nom::multi::many_till;
use nom::sequence::tuple;
use serde::{Serialize,Deserialize};
use derive_new::new;

use std::io;
use std::path::Path;
use std::str::FromStr;
use nom::multi::many1;
use nom::sequence::delimited;
use nom::{
    IResult,
    bytes::streaming::tag,
    character::streaming::{one_of,alphanumeric1},
    branch::alt,
};

#[derive(Debug,Eq,PartialEq,Clone,new,Serialize,Deserialize)]
pub struct PGN {
    pub time_control:    String,
    pub ply_count:       u32,
    pub result:          TDOutcome,
    pub moves:           Vec<String>,
    pub elo:             [u32; 2],
}

impl PGN {
    pub fn empty() -> Self {
        Self {
            time_control: String::new(),
            ply_count:    0,
            result:       TDOutcome::Draw,
            moves:        vec![],
            elo:          [0, 0],
        }
    }
}

pub fn process_pgns(ts: &Tables, pgns: &[PGN]) -> Vec<TrainingData> {
    let g0 = Game::from_fen(ts, STARTPOS).unwrap();
    // let mut out = vec![];

    use rayon::prelude::*;

    let out: Vec<TrainingData> = pgns.par_iter().map(|pgn| {
        let mut g = g0.clone();
        let mut moves = vec![];
        'inner: for mv0 in pgn.moves.iter() {
            let mv0 = mv0.chars().filter(|c| *c != '+' && *c != '#').collect::<String>();
            let mv = g.convert_from_algebraic(ts, &mv0).unwrap();
            if let Ok(g2) = g.make_move_unchecked(ts, mv) {
                g = g2;
            } else {
                // panic!("{:?}\n{:?}\nbad move: {} {:?}", g.to_fen(), g, mv0, mv);
                break 'inner;
            }
        }
        let td = TrainingData {
            opening: vec![],
            moves,
            result: pgn.result,
        };
        // out.push(td);
        td
    }).collect();
    out
}

pub fn parse_pgns<P: AsRef<Path>>(
    path:          P,
    count:         Option<usize>,
) -> io::Result<Vec<PGN>> {
    let mut buf = std::fs::read_to_string(path)?;

    // let (_,pgn) = parse_pgn(&buf).unwrap();
    // eprintln!("pgn = {:?}", pgn);

    let mut pgns = vec![];
    let mut ss: &str = &buf;
    let mut n = 0;

    loop {
        // eprintln!("ss = {}", ss);
        match parse_pgn(&ss) {
            Ok((s,pgn)) => {
                pgns.push(pgn);
                ss = s;
            }
            Err(nom::Err::Incomplete(_)) => {
                eprintln!("Incomplete = {}", ss);
                break;
            },
            Err(e) => {
                eprintln!("e = {:?}\n{}", e, ss);
                break;
            }
        }

        n += 1;
        if count.map_or(false, |c| n >= c) { break; }
    }

    // eprintln!("pgns.len() = {:?}", pgns.len());

    // for p in pgns.iter() {
    //     eprintln!("p = {:?}", p);
    // }

    // unimplemented!()
    Ok(pgns)
}

fn parse_pgn(s: &str) -> IResult<&str, PGN> {

    let (s,headers) = many1(parse_header)(s)?;

    let mut out = PGN::empty();

    for (h,x) in headers.iter() {
        match h {
            &"Result" => {
                let (_,res) = parse_result(x)?;
                out.result = res;
            },
            _ => {}
        }
    }
    let (s,_) = line_ending(s)?;

    let (s,moves) = parse_moves(s)?;

    let mvs: Vec<String> = moves.iter().map(|x| x.to_string()).collect();
    out.moves = mvs;

    let (s,_) = space0(s)?;
    let (s,_) = char('{')(s)?;
    let (s,_) = many0(none_of("}"))(s)?;
    let (s,_) = char('}')(s)?;
    let (s,_) = space0(s)?;

    let (s,_) = parse_result(s)?;

    let (s,_) = multispace0(s)?;

    Ok((s, out))
}

fn parse_moves(s: &str) -> IResult<&str, Vec<&str>> {
    let (s,mvs) = many0(|s| {
        let (s,_) = digit1(s)?;
        let (s,_) = char('.')(s)?;
        let (s,_) = space0(s)?;
        let (s,mv0) = parse_move(s)?;
        let (s,_) = space0(s)?;
        match parse_move(s) {
            Ok((s,mv1)) => {
                let (s,_) = space0(s)?;
                Ok((s,vec![mv0,mv1]))
            },
            Err(e) => {
                Ok((s,vec![mv0]))
            },
        }
        // Ok((s,vec![mv0,mv1]))
    })(s)?;
    let mvs: Vec<&str> = mvs.concat();
    Ok((s,mvs))
}

fn parse_move(s: &str) -> IResult<&str, &str> {

    // let files = one_of("abcdefgh");
    // let pcs   = one_of("PNBRQK");

    // let (s,xs) = many_till(one_of("abcdefghPNBRQK12345678x+"), char(' '))(s)?;
    // let (s,xs) = many_till(one_of("abcdefghPNBRQK12345678x+"), char(' '))(s)?;
    let (s,xs) = take_while1(|c| c != ' ' && c != '{')(s)?;
    Ok((s, xs))
}

fn parse_header(s: &str) -> IResult<&str, (&str,&str)> {
    let (s,xs) = delimited(
        char('['),
        tuple((alphanumeric1, space0,
               delimited(char('"'), take_while1(|c| c != '"'), char('"')))),
        char(']')
    )(s)?;
    let (s,_) = line_ending(s)?;
    Ok((s, (xs.0, xs.2)))
}

// fn parse_plycount(s: &str) -> IResult<&str, u32> {
// }

fn parse_result(s: &str) -> IResult<&str, TDOutcome> {
    let t0 = alt((tag("0"),tag("1")));
    let t1 = alt((tag("0"),tag("1")));
    let (s0,(w,_,b)) = alt((
        // tuple((one_of("01"), char('-'), one_of("01"))),
        tuple((t0, char('-'), t1)),
        tuple((tag("1/2"),char('-'),tag("1/2"))),
    ))(s)?;
    match (w,b) {
        ("1/2","1/2") => Ok((s0,TDOutcome::Draw)),
        ("1","0")     => Ok((s0,TDOutcome::Win(White))),
        ("0","1")     => Ok((s0,TDOutcome::Win(Black))),
        _             => panic!("parse_result: {:?}", s0),
    }
}







