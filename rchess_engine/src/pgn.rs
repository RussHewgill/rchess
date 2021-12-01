
use crate::brain::trainer::TDOutcome;
use crate::brain::trainer::TrainingData;
use crate::types::*;
use crate::tables::*;

use nom::character::complete::digit1;
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
    bytes::complete::tag,
    character::complete::{one_of,alphanumeric1},
    branch::alt,
};

#[derive(Debug,Eq,PartialEq,Clone,new,Serialize,Deserialize)]
pub struct PGN {
    time_control:    String,
    ply_count:       u32,
    result:          TDOutcome,
    moves:           Vec<Move>,
    elo:             [u32; 2],
}

pub fn parse_pgns<P: AsRef<Path>>(
    path:          P,
) -> io::Result<Vec<TrainingData>> {
    unimplemented!()
}

fn parse_pgn(s: &str) -> IResult<&str, PGN> {
    let (s,headers) = many1(parse_header)(s)?;

    let result: Vec<IResult<&str, TDOutcome>> = headers.iter().map(|x| parse_result(x)).collect();

    unimplemented!()
}

fn parse_result(s: &str) -> IResult<&str, TDOutcome> {
    let (s0,(w,_,b)) = delimited(
        tag("\""),
        alt((
            tuple((digit1, tag("-"), digit1)),
            tuple((tag("1/2"),tag("-"),tag("1/2"))),
        )),
        tag("\""))(s)?;
    match (w,b) {
        ("1/2","1/2") => Ok((s0,TDOutcome::Draw)),
        ("1","0")     => Ok((s0,TDOutcome::Win(White))),
        ("0","1")     => Ok((s0,TDOutcome::Win(Black))),
        _             => panic!("parse_result: {:?}", s0),
    }
}

fn parse_header(s: &str) -> IResult<&str, String> {
    let (s,xs) = delimited(
        tag("["),
        many1(alphanumeric1),
        tag("]")
    )(s)?;
    Ok((s,xs))
}






