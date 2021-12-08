
use crate::brain::trainer::*;
use crate::explore::ExHelper;
use crate::evaluate::*;
use crate::pawn_hash_table::PHTableFactory;
use crate::qsearch::exhelper_once;
use crate::searchstats::SearchStats;
use crate::texel::TxPosition;
use crate::types::*;
use crate::tables::*;

use crossbeam_channel::{Sender,Receiver};
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

use rayon::prelude::*;

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

// pub fn load_pgns_td<P: AsRef<Path>>(ts: &Tables, path: P, count: Option<usize>)
//                                  -> io::Result<Vec<TrainingData>> {
//     let pgns = parse_pgns(path, count)?;
//     let tds = process_pgns_td(ts, &pgns);
//     Ok(tds)
// }

pub fn load_pgns_tx<P: AsRef<Path>>(
    ts:             &Tables,
    mut exhelper:   &mut ExHelper,
    count:          Option<usize>,
    path:           P,
) -> io::Result<Vec<TxPosition>> {

    let t0 = std::time::Instant::now();
    let pgns = parse_pgns(path, count).unwrap();
    eprintln!("finished 0 in {:.3} seconds", t0.elapsed().as_secs_f64());
    let t0 = std::time::Instant::now();
    let ps = process_pgns_tx(&ts, &mut exhelper, count, &pgns);
    eprintln!("finished 1 in {:.3} seconds", t0.elapsed().as_secs_f64());

    // let pgns = parse_pgns(path, count)?;
    // let ps = process_pgns_tx(ts, exhelper, count, &pgns);
    Ok(ps)
}

pub fn process_pgns_tx(
    ts:             &Tables,
    exhelper:       &mut ExHelper,
    count:          Option<usize>,
    pgns:           &[PGN]
) -> Vec<TxPosition> {
    let g0 = Game::from_fen(ts, STARTPOS).unwrap();
    // let mut out = vec![];

    let (ev_mid,ev_end) = (&exhelper.cfg.eval_params_mid,&exhelper.cfg.eval_params_end);

    let out: Vec<TxPosition> = pgns.par_iter().map(|pgn| {
    // let out: Vec<TxPosition> = pgns.iter().map(|pgn| {
        let mut g = g0.clone();
        let mut games = vec![];

        let mut stats = SearchStats::default();
        let mut exhelper2 = exhelper_once(&g, g.state.side_to_move, &ev_mid, &ev_end, None, None);

        for mv0 in pgn.moves.iter() {
            let mv0 = mv0.chars().filter(|c| *c != '+' && *c != '#').collect::<String>();
            let mv = g.convert_from_algebraic(ts, &mv0).unwrap();
            if let Ok(g2) = g.make_move_unchecked(ts, mv) {
                g = g2;

                let moves = g.search_all(&ts);

                if !mv.filter_all_captures()
                    && g.state.checkers.is_empty()
                    && !moves.is_end() {

                        let score   = g.sum_evaluate(&ts, &ev_mid, &ev_end, Some(&exhelper2.ph_rw));

                        let q_score = exhelper2.qsearch_once_mut(&ts, &g, &mut stats);
                        let q_score = g.state.side_to_move.fold(q_score, -q_score);

                        if score == q_score {
                            // ps.push(TxPosition::new(g.clone(), td.result));
                            games.push(g.clone());
                        } else {
                            // non_q += 1;
                        }

                }

                // games.push(g.clone());
            } else {
                eprintln!("{:?}\n{:?}\nbad move: {} {:?}", g.to_fen(), g, mv0, mv);
                break;
            }
        }

        let xs: Vec<TxPosition> = games.into_iter().map(|game| {
            TxPosition {
                game,
                result: pgn.result,
            }
        }).collect();
        xs
    }).filter(|xs| xs.len() > 0).flatten().collect();
    out
}

pub fn process_pgns_td(
    ts:               &Tables,
    (ev_mid,ev_end):  &(EvalParams,EvalParams),
    pgns:             &[PGN]
) -> Vec<TrainingData> {
    let g0 = Game::from_fen(ts, STARTPOS).unwrap();

    let ncpus = num_cpus::get();
    let ph_factory = PHTableFactory::new();

    let pgns2 = pgns.chunks(pgns.len() / ncpus).map(|xs| {
        let ph_rw = ph_factory.handle();
        let exhelper = exhelper_once(&g0, White, ev_mid, ev_end, Some(&ph_rw), None);
        (exhelper.clone(), xs)
    }).collect::<Vec<(ExHelper, &[PGN])>>();

    let out: Vec<TrainingData> = pgns2.into_par_iter().map(|(exhelper, xs)| {
        let mut stats = SearchStats::default();

        xs.into_iter().map(|pgn| {

            let mut g = g0.clone();
            let mut moves = vec![];
            'inner: for mv0 in pgn.moves.iter() {
                let mv0 = mv0.chars().filter(|c| *c != '+' && *c != '#').collect::<String>();
                let mv = g.convert_from_algebraic(ts, &mv0).unwrap();
                if let Ok(g2) = g.make_move_unchecked(ts, mv) {
                    g = g2;

                    let (skip,score) = {
                        if mv.filter_all_captures()
                            || !g.state.checkers.is_empty()
                            {
                                (true, 0)
                            } else {
                                let score   = g.sum_evaluate(&ts, &ev_mid, &ev_end, None);
                                let q_score = exhelper.qsearch_once(&ts, &g, &mut stats);
                                let q_score = g.state.side_to_move.fold(q_score, -q_score);

                                if score == q_score {
                                    (false,score)
                                } else if score.abs() > STALEMATE_VALUE - 100 {
                                    (true, 0)
                                } else {
                                    (true, 0)
                                }
                            }
                    };

                    let te = TDEntry {
                        mv,
                        eval:  score,
                        skip,
                    };

                    moves.push(te);
                } else {
                    eprintln!("{:?}\n{:?}\nbad move: {} {:?}", g.to_fen(), g, mv0, mv);
                    break 'inner;
                }
            }
            let td = TrainingData {
                opening: vec![],
                moves,
                result: pgn.result,
            };
            td
        }).collect::<Vec<_>>()
    }).flatten().collect();
    out
}

pub fn _process_pgn_td(
    ts:               &Tables,
    g0:               &Game,
    exhelper:         &ExHelper,
    pgn:              &PGN,
) -> TrainingData {
    let mut g = g0.clone();
    let mut moves = vec![];
    let mut stats = SearchStats::default();
    let (ev_mid,ev_end) = (&exhelper.cfg.eval_params_mid,&exhelper.cfg.eval_params_end);
    'inner: for mv0 in pgn.moves.iter() {
        let mv0 = mv0.chars().filter(|c| *c != '+' && *c != '#').collect::<String>();
        let mv = g.convert_from_algebraic(ts, &mv0).unwrap();
        if let Ok(g2) = g.make_move_unchecked(ts, mv) {
            g = g2;

            let (skip,score) = {
                if mv.filter_all_captures()
                    || !g.state.checkers.is_empty()
                    {
                        (true, 0)
                    } else {
                        let score   = g.sum_evaluate(&ts, &ev_mid, &ev_end, None);
                        let q_score = exhelper.qsearch_once(&ts, &g, &mut stats);
                        let q_score = g.state.side_to_move.fold(q_score, -q_score);

                        if score == q_score {
                            (false,score)
                        } else if score.abs() > STALEMATE_VALUE - 100 {
                            (true, 0)
                        } else {
                            (true, 0)
                        }
                    }
            };

            let te = TDEntry {
                mv,
                eval:  score,
                skip,
            };

            moves.push(te);
        } else {
            eprintln!("{:?}\n{:?}\nbad move: {} {:?}", g.to_fen(), g, mv0, mv);
            break 'inner;
        }
    }
    let td = TrainingData {
        opening: vec![],
        moves,
        result: pgn.result,
    };
    td
}

pub fn process_pgns_td_par<P: AsRef<Path> + Send>(
    ts:               &Tables,
    (ev_mid,ev_end):  &(EvalParams,EvalParams),
    pgn_path:        P,
    out_path:        P,
) {
    crossbeam::scope(|s| {

        let (tx,rx)           = crossbeam_channel::unbounded();
        let (tx_save,rx_save) = crossbeam_channel::unbounded();

        s.spawn(|_| {
            parse_pgns_par(pgn_path, None, tx).unwrap();
        });

        s.spawn(|_| {
            _process_pgns_td_par(&ts, (ev_mid,ev_end), rx, tx_save);
        });

        s.spawn(move |_| {
            let mut file = std::fs::File::create(out_path).unwrap();
            loop {
                match rx_save.recv() {
                    Ok(td) => {
                        TrainingData::save_into(true, &mut file, &td).unwrap();
                    },
                    Err(e) => {
                        eprintln!("breaking save loop, e = {:?}", e);
                        break;
                    },
                }
            }
        });

    }).unwrap();
}

pub fn _process_pgns_td_par(
    ts:               &Tables,
    (ev_mid,ev_end):  (&EvalParams,&EvalParams),
    rx:               Receiver<PGN>,
    tx_save:          Sender<TrainingData>,
) {
    let g0 = Game::from_fen(ts, STARTPOS).unwrap();

    let ncpus = num_cpus::get();
    let ph_factory = PHTableFactory::new();
    let ph_rw = ph_factory.handle();
    let exhelper = exhelper_once(&g0, White, ev_mid, ev_end, Some(&ph_rw), None);

    loop {
        match rx.recv() {
            Ok(pgn) => {
                let td = _process_pgn_td(ts, &g0, &exhelper, &pgn);
                tx_save.send(td).unwrap();
            },
            Err(e) => {
                eprintln!("breaking process_pgns_td_par, e = {:?}", e);
                break;
            },
        }
    }
}

pub fn parse_pgns_par<P: AsRef<Path>>(
    path:            P,
    count:           Option<usize>,
    tx:              Sender<PGN>,
) -> io::Result<()> {
    let mut buf = std::fs::read_to_string(path)?;

    let mut ss: &str = &buf;
    let mut n = 0;

    loop {
        // eprintln!("ss = {}", ss);
        // println!("wat 0");
        match parse_pgn(&ss) {
            Ok((s,pgn)) => {
                // eprintln!("pgn = {:?}", pgn);
                // pgns.push(pgn);
                tx.send(pgn).unwrap();
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

    Ok(())
}

pub fn parse_pgns<P: AsRef<Path>>(path: P, count: Option<usize>) -> io::Result<Vec<PGN>> {
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







