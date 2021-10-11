
use crate::types::*;
use crate::tables::*;

use std::str::FromStr;
use std::collections::{HashMap,HashSet};
use std::io::Write;
use std::process::{Command,Stdio};

pub fn read_json_fens(path: &str) -> std::io::Result<Vec<(u64,u64,String)>> {
    let file = std::fs::read_to_string(path)?;
    let lines = file.lines();

    let mut out: Vec<(u64,u64,String)> = vec![];
    for line in lines.into_iter() {
        let line = line.split("; ").collect::<Vec<&str>>();
        if line.len() == 2 {
            let fen = line[0];
            // eprintln!("fen = {:?}", fen);
            let mut line = line[1].split(" = ");
            // eprintln!("line = {:?}", line);
            let depth = line.next().unwrap().replace("perft ", "");
            let depth = u64::from_str(&depth).unwrap();
            // eprintln!("depth = {:?}", depth);
            let nodes = line.next().unwrap();
            // eprintln!("nodes = {:?}", nodes);
            let nodes = u64::from_str(&nodes).unwrap();

            // let g = Game::from_fen(&fen).unwrap();
            out.push((depth, nodes, fen.to_string()))
        }
    }

    Ok(out)
}

pub fn find_move_error(
    ts:        &Tables,
    fen:       &str,
    depth:     u64,
    last_move: Option<Move>,
) -> std::io::Result<Option<(Move,String)>> {

    let (_, ((ns0,nodes0),(ns1,nodes1))) = test_stockfish(&fen, depth, false)?;

    // No errors found
    if ns0 == ns1 {
        panic!("find_move_error: No errors");
        // return Ok(None);
    }

    // moves in one but not both
    let diff: HashSet<String> = {
        let d0: HashSet<String> = nodes0.keys().cloned().collect();
        let d1: HashSet<String> = nodes1.keys().cloned().collect();
        let diff0: HashSet<String> = d0.difference(&d1).cloned().collect();
        let diff1: HashSet<String> = d1.difference(&d0).cloned().collect();
        // eprintln!("diff0 = {:?}", diff0);
        // eprintln!("diff1 = {:?}", diff1);
        diff0.union(&diff1).cloned().collect()
    };

    // if wrong moves exist or if legal moves are missing, return that FEN
    if !diff.is_empty() {
        return Ok(Some((last_move.unwrap(), fen.to_string())));
    } else {

        for (k0,(m0,v0)) in nodes0.iter() {
            let v1 = nodes1.get(k0).unwrap();

            // perft after move finds error
            if v0 != v1 {
                let mut g = Game::from_fen(&fen).unwrap();
                g.recalc_gameinfo_mut(&ts);
                // eprintln!("g0 = {:?}", g);

                let mut g = g.make_move_unchecked(&ts, m0).unwrap();
                g.recalc_gameinfo_mut(&ts);
                // eprintln!("g1 = {:?}", g);
                let fen2 = g.to_fen();

                return find_move_error(&ts, &fen2, depth - 1, Some(*m0))
            }

        }

        unimplemented!()
    }
}

/// (_, ((ns0,nodes0),(ns1,nodes1)))
/// ns0    = total nodes found
/// nodes0 = HashMap<Move String, (Move, nodes after Move)>
pub fn test_stockfish(
    fen:    &str,
    n:      u64,
    print:  bool,
) -> std::io::Result<(f64,((u64,HashMap<String,(Move,i64)>),(u64,HashMap<String,i64>)))> {

    let mut child = Command::new("stockfish")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    child.stdin
        .as_mut()
        .unwrap()
        .write_all(format!("position fen {}\n", fen).as_bytes())?;

    child.stdin
        .as_mut()
        .unwrap()
        .write_all(format!("go perft {}\n", n).as_bytes())?;

    let output = child.wait_with_output()?;

    let mut g = Game::from_fen(&fen).unwrap();
    let ts = Tables::new();
    g.recalc_gameinfo_mut(&ts);
    // eprintln!("g = {:?}", g);

    let now = std::time::Instant::now();
    let (ns0, ms) = g.perft(&ts, n);
    let done = now.elapsed().as_secs_f64();
    // println!("perft done in {} seconds.", now.elapsed().as_secs_f64());

    let mut nodes0: HashMap<String, (Move,i64)> = HashMap::new();
    for (m,k) in ms.into_iter() {
        match m {
            Move::Promotion { new_piece, .. } | Move::PromotionCapture { new_piece, .. } => {
                let c = match new_piece {
                    Queen  => 'q',
                    Knight => 'n',
                    Rook   => 'r',
                    Bishop => 'b',
                    _      => panic!("Bad promotion"),
                };
                let mm = format!("{:?}{:?}{}", m.sq_from(), m.sq_to(), c).to_ascii_lowercase();
                nodes0.insert(mm, (m,k as i64));
            },
            _ => {
                let mm = format!("{:?}{:?}", m.sq_from(), m.sq_to()).to_ascii_lowercase();
                nodes0.insert(mm, (m,k as i64));
            },
        }

        // eprintln!("str, move = {}: {:?}", mm, m);

    }

    if output.status.success() {
        let raw_output = String::from_utf8(output.stdout).unwrap();
        let mut out = raw_output.lines();
        out.next().unwrap();
        let mut out: Vec<&str> = out.collect();

        let ns1 = out[out.len() - 2];
        let ns1: Vec<&str> = ns1.split(": ").collect();
        let ns1 = u64::from_str(ns1[1]).unwrap();

        out.truncate(out.len() - 3);

        let mut nodes1: HashMap<String, i64> = HashMap::new();
        for x in out.into_iter() {
            // println!("{}", x);
            let mk: Vec<&str> = x.split(": ").collect();
            let (m,k) = (mk[0], u64::from_str(mk[1]).unwrap());
            // eprintln!("m, k = {:?}, {}", m, k);
            nodes1.insert(m.to_string(), k as i64);
        }

        // let x0 = nodes0.len();
        // let x1 = nodes0.len();
        // eprintln!("x0 = {:?}", x0);
        // eprintln!("x1 = {:?}", x1);

        let d0: HashSet<String> = nodes0.keys().cloned().collect();
        let d1: HashSet<String> = nodes1.keys().cloned().collect();
        let diff0: HashSet<String> = d0.difference(&d1).cloned().collect();
        let diff1: HashSet<String> = d1.difference(&d0).cloned().collect();
        let diff: HashSet<String> = diff0.union(&diff1).cloned().collect();

        for k in d0.union(&d1) {
            match (nodes0.get(k),nodes1.get(k)) {
                (Some((_,v0)),None)     => {
                    if print {
                        eprintln!("k0, rchess, stockfish = {:?} / {:>4} / ---- / failed", k, v0);
                    }
                },
                (None,Some(v1))         => {
                    if print {
                        eprintln!("k0, rchess, stockfish = {:?} / ---- / {:>4} / failed", k, v1);
                    }
                },
                (Some((_,v0)),Some(v1)) => {
                    if print {
                        if v0 == v1 {
                            eprintln!("k0, rchess, stockfish = {:?} / {:>4} / {:>4}", k, v0, v1);
                        } else {
                            eprintln!("k0, rchess, stockfish = {:?} / {:>4} / {:>4} / failed ({})",
                                k, v0, v1, v0 - v1);
                        }
                    }
                },
                (None,None)             => {
                    panic!()
                },
            }
        }

        // for (k0,(_,v0)) in nodes0.iter() {
        //     if let Some(v1) = nodes1.get(k0) {
        //         if print {
        //             if v0 == v1 {
        //                 eprintln!("k0, rchess, stockfish = {:?} / {:>4} / {:>4}", k0, v0, v1);
        //             } else {
        //                 eprintln!("k0, rchess, stockfish = {:?} / {:>4} / {:>4} / failed ({})",
        //                         k0, v0, v1, v0 - v1);
        //             }
        //         }
        //     } else {
        //         if print {
        //             eprintln!("k0, rchess, stockfish = {:?} / {:>4} / --", k0, v0);
        //         }
        //     }

        //     // assert!(v0 == v1);

        // }

        if print {
            // eprintln!("ns1 = {:?}", ns1);
            if ns0 == ns1 {
                eprintln!("rchess, stockfish = {:>2} / {:>2}", ns0, ns1);
            } else {
                eprintln!("rchess, stockfish = {:>2} / {:>2} / failed ({})",
                        ns0, ns1, ns0 as i64 - ns1 as i64);
            }
        }

        // let words = raw_output.split_whitespace()
        //     .map(|s| s.to_lowercase())
        //     .collect::<HashSet<_>>();
        // println!("Found {} unique words:", words.len());
        // println!("{:#?}", words);

        Ok((done,((ns0, nodes0), (ns1, nodes1))))
    } else {
        let err = String::from_utf8(output.stderr).unwrap();
        // error_chain::bail!("External command failed:\n {}", err)
        panic!("wat")
    }


}



