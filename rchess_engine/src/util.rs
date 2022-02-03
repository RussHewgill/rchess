
use crate::movegen::MoveGen;
use crate::types::*;
use crate::tables::*;

use std::str::FromStr;
use std::collections::{HashMap,HashSet};
use std::io::Write;
use std::process::{Command,Stdio};


pub fn pretty_print_si(x: i64) -> String {
    if x.abs() > 1_000_000 {
        format!("{:.1}M", x as f64 / 1_000_000.)
    } else if x > 1000 {
        format!("{:.1}k", x as f64 / 1000.)
    } else {
        format!("{}", x)
    }
}

// pub fn pretty_print_grid<T: std::fmt::Debug + std::fmt::Display>(grid: &[T; 64], padding: usize) {
// pub fn pretty_print_grid<T: std::fmt::Debug + std::fmt::Display>(grid: &[T; 64]) -> std::io::Result<()> {
pub fn pretty_print_grid(grid: &[Score; 64]) -> std::io::Result<()> {
    use std::io::{self, Write};
    use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    for y0 in 0..8 {
        let y = 7-y0;
        let mut line = String::new();

        // line.push_str(&format!("{}  ", y + 1));

        stdout.set_color(ColorSpec::new().set_fg(None))?;
        write!(stdout, "{}  ", y + 1)?;

        for x in 0..8 {

            // let ch: char = match self.get_at(Coord::new(x,y)) {
            //     Some((c,p)) => p.print(c),
            //     None        => {
            //         let c = square_color(Coord::new(x,y));
            //         c.print()
            //     },
            // };

            let s = grid[Coord::new(x,y)];

            // ((*s).clamp(-127,127) + 127) as u8
            let x = (s.clamp(-127, 127) - 127).abs() as u8;

            // stdout.set_color(ColorSpec::new().set_fg(Some(map_color_score(*s))))?;
            // stdout.set_color(ColorSpec::new().set_fg(Some(map_color(((*s).clamp(-127,127) + 127) as u8))))?;
            stdout.set_color(ColorSpec::new().set_fg(Some(map_color(x))))?;
            stdout.flush()?;

            // stdout.write()
            write!(stdout, "{:>3} ", s)?;

            // line.push_str(&format!("{:>3}", s));
            // line.push(' ');
        }
        // f.write_str(&format!("{}\n", line))?;
        println!("{}", line);
    }
    let mut line = String::new();
    stdout.set_color(ColorSpec::new().set_fg(None))?;
    line.push_str(&format!("     "));
    let cs = vec!['A','B','C','D','E','F','G','H'];
    for x in 0..8 {
        line.push_str(&format!("{}   ", cs[x]));
    }
    // f.write_str(&format!("{}\n", line))?;
    println!("{}", line);

    Ok(())
}

pub fn map_color_score(score: Score) -> termcolor::Color {
    let score = scale_score_to_i8(score);

    let x = (score as i16 + 127) as u8;

    // let x = (score.clamp(-127, 127) - 127).abs() as u8;

    map_color(x)
}

pub fn map_color(x: u8) -> termcolor::Color {
    let x = x / 6;

    let i = x as f64;
    let r = (f64::sin(0.024 * i + 0.0) * 127.0 + 128.0).round();
    let g = (f64::sin(0.024 * i + 2.0) * 127.0 + 128.0).round();
    let b = (f64::sin(0.024 * i + 4.0) * 127.0 + 128.0).round();
    let r = r as u8;
    let g = g as u8;
    let b = b as u8;
    termcolor::Color::Rgb(r,g,b)
}

pub fn read_epd_no_bm(path: &str) -> std::io::Result<Vec<String>> {
    let file = std::fs::read_to_string(path)?;
    let lines = file.lines();
    let mut out = vec![];

    let lines = lines.filter(|x| {
        !x.starts_with("#")
    });

    for line in lines.into_iter() {
        out.push(line.to_owned());
    }

    Ok(out)
}

pub fn read_epd(path: &str) -> std::io::Result<Vec<(String, Vec<String>)>> {
// pub fn read_epd(path: &str) -> std::io::Result<Vec<(String, String)>> {
    let file = std::fs::read_to_string(path)?;
    let lines = file.lines();
    let mut out = vec![];

    let lines = lines.filter(|x| {
        !x.starts_with("#")
    });

    // let mut lines = lines.collect::<Vec<&str>>();
    // lines.truncate(1);

    for line in lines.into_iter() {
        let mut line = line.split("bm").collect::<Vec<&str>>();

        let mv = line[1].to_string();
        let mut mv = mv.split(";");
        let mv = mv.next().unwrap();
        let mut mv = mv.split(" ").map(|s| s.to_string());
        mv.next();
        let mv = mv.collect::<Vec<String>>();

        // out.push((line[0].to_string(), mv.join(", ")));
        out.push((line[0].to_string(), mv));

        // out.push((line[0].to_string(), line[1].to_string()));

        // let mut line = line.split("/").collect::<Vec<&str>>();
        // let last = line.pop().unwrap();
        // let last = last.split(" ").collect::<Vec<&str>>();
        // // eprintln!("last = {:?}", last);
        // line.push(last[0].clone());
        // let line = line.join("/");
        // // eprintln!("line = {:?}", line);
        // let line = vec![line, last[1].to_string(), last[2].to_string(), last[3].to_string()];
        // let line = line.join(" ");
        // eprintln!("line = {:?}", line);
        // let last = &last[4..last.len()];
        // eprintln!("last = {:?}", last);

        // let last = &last[1..last.len()];
        // eprintln!("last = {:?}", last);
        // eprintln!("line.len() = {:?}", line.len());
        // let fen: String = line[0..8].join("/");
        // eprintln!("fen = {:?}", fen);
    }

    // unimplemented!()
    Ok(out)
}

pub fn read_ccr_onehour(path: &str) -> std::io::Result<Vec<(String, Vec<String>)>> {
    let file = std::fs::read_to_string(path)?;
    let lines = file.lines();
    let mut out = vec![];

    // let mut lines = lines.collect::<Vec<&str>>();
    // lines.truncate(5);

    for line in lines.into_iter() {
        let line = line.split("id").collect::<Vec<&str>>();
        let fen = line[0];
        let ms = &line[1..line.len()];
        // let m = "".to_string();
        // let m = ms.concat().to_string();
        let m: String = ms.concat();
        let mut m = m.split("; ").collect::<Vec<&str>>();
        m.reverse();
        // m.truncate(m.len() - 1);
        m.pop();
        // let m = &m[1..m.len()-1];
        // let m = m.concat();

        // eprintln!("m = {:?}", m);
        // let m = "";
        let m = m.into_iter().map(|s| {
            s.to_string()
                .replace(";","")
                .replace("am","")
                .replace("bm","")
                .replace(" ","")
                .replace("+","")
        }).collect();

        out.push((fen.to_string(),m))
    }

    Ok(out)
}

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

// pub fn find_move_error(
//     ts:        &Tables,
//     fen:       &str,
//     depth:     u64,
//     last_move: Option<Move>,
// ) -> std::io::Result<Option<(Move,String)>> {

//     let (_, ((ns0,nodes0),(ns1,nodes1))) = test_stockfish(&ts, &fen, depth, false)?;

//     // No errors found
//     if ns0 == ns1 {
//         panic!("find_move_error: No errors");
//         // return Ok(None);
//     }

//     // moves in one but not both
//     let diff: HashSet<String> = {
//         let d0: HashSet<String> = nodes0.keys().cloned().collect();
//         let d1: HashSet<String> = nodes1.keys().cloned().collect();
//         let diff0: HashSet<String> = d0.difference(&d1).cloned().collect();
//         let diff1: HashSet<String> = d1.difference(&d0).cloned().collect();
//         // eprintln!("diff0 = {:?}", diff0);
//         // eprintln!("diff1 = {:?}", diff1);
//         diff0.union(&diff1).cloned().collect()
//     };

//     // if wrong moves exist or if legal moves are missing, return that FEN
//     if !diff.is_empty() {
//         return Ok(Some((last_move.unwrap(), fen.to_string())));
//     } else {

//         for (k0,(m0,v0)) in nodes0.iter() {
//             let v1 = nodes1.get(k0).unwrap();

//             // perft after move finds error
//             if v0 != v1 {
//                 let mut g = Game::from_fen(&ts, &fen).unwrap();
//                 let _ = g.recalc_gameinfo_mut(&ts);
//                 // eprintln!("g0 = {:?}", g);

//                 let mut g = g.make_move_unchecked(&ts, *m0).unwrap();
//                 let _ = g.recalc_gameinfo_mut(&ts);
//                 // eprintln!("g1 = {:?}", g);
//                 let fen2 = g.to_fen();

//                 return find_move_error(&ts, &fen2, depth - 1, Some(*m0))
//             }

//         }

//         unimplemented!()
//     }
// }

#[derive(Debug,Default,PartialEq,PartialOrd,Clone)]
pub struct StockfishEval {
    pub total_classic:    f64,
    pub total_nn:         f64,
    pub material_mg:      [[f64; 6]; 2],
    pub material_eg:      [[f64; 6]; 2],
}

pub fn stockfish_eval(
    fen:    &str,
    print:  bool,
) -> std::io::Result<(String, StockfishEval)> {
    use regex::Regex;
    let mut eval = StockfishEval::default();

    let mut child = Command::new("stockfish")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    child.stdin
        .as_mut()
        .unwrap()
        .write_all(format!("ucinewgame\nposition fen {}\n", fen).as_bytes())?;

    child.stdin
        .as_mut()
        .unwrap()
        .write_all("eval\n".as_bytes())?;

    let output = child.wait_with_output()?;
    let output = String::from_utf8(output.stdout).unwrap();

    let output = output.lines().collect::<Vec<_>>();

    let output_mat = &output[4..23];

    let output_total = &output[output.len()-4 .. output.len()-1];
    // eprintln!("{}", output_total.join("\n"));

    // let p = Regex::new(r"(-?\d+\.\d+)").unwrap();
    let reg = Regex::new(r"(-?\d+\.\d+)").unwrap();

    let tc = reg.find(output_total[0]).unwrap();
    let tn = reg.find(output_total[1]).unwrap();

    eval.total_classic = f64::from_str(tc.as_str()).unwrap();
    eval.total_nn = f64::from_str(tn.as_str()).unwrap();

    let output_str = output_mat.join("\n");
    if print { println!("{}", output_str) }

    let pawns   = output_mat[6];
    let rooks   = output_mat[9];
    let knights = output_mat[7];
    let bishops = output_mat[8];
    let queens  = output_mat[10];
    // eprintln!("{}", queens);

    let mut ps = reg.find_iter(pawns).collect::<Vec<_>>();
    eval.material_mg[White][Pawn.index()] = f64::from_str(ps[0].as_str()).unwrap();
    eval.material_eg[White][Pawn.index()] = f64::from_str(ps[1].as_str()).unwrap();
    eval.material_mg[Black][Pawn.index()] = f64::from_str(ps[2].as_str()).unwrap();
    eval.material_eg[Black][Pawn.index()] = f64::from_str(ps[3].as_str()).unwrap();

    let mut ns = reg.find_iter(knights).collect::<Vec<_>>();
    eval.material_mg[White][Knight.index()] = f64::from_str(ns[0].as_str()).unwrap();
    eval.material_eg[White][Knight.index()] = f64::from_str(ns[1].as_str()).unwrap();
    eval.material_mg[Black][Knight.index()] = f64::from_str(ns[2].as_str()).unwrap();
    eval.material_eg[Black][Knight.index()] = f64::from_str(ns[3].as_str()).unwrap();

    let mut rs = reg.find_iter(rooks).collect::<Vec<_>>();
    eval.material_mg[White][Rook.index()] = f64::from_str(rs[0].as_str()).unwrap();
    eval.material_eg[White][Rook.index()] = f64::from_str(rs[1].as_str()).unwrap();
    eval.material_mg[Black][Rook.index()] = f64::from_str(rs[2].as_str()).unwrap();
    eval.material_eg[Black][Rook.index()] = f64::from_str(rs[3].as_str()).unwrap();

    let mut bs = reg.find_iter(bishops).collect::<Vec<_>>();
    eval.material_mg[White][Bishop.index()] = f64::from_str(bs[0].as_str()).unwrap();
    eval.material_eg[White][Bishop.index()] = f64::from_str(bs[1].as_str()).unwrap();
    eval.material_mg[Black][Bishop.index()] = f64::from_str(bs[2].as_str()).unwrap();
    eval.material_eg[Black][Bishop.index()] = f64::from_str(bs[3].as_str()).unwrap();

    let mut qs = reg.find_iter(queens).collect::<Vec<_>>();
    eval.material_mg[White][Queen.index()] = f64::from_str(qs[0].as_str()).unwrap();
    eval.material_eg[White][Queen.index()] = f64::from_str(qs[1].as_str()).unwrap();
    eval.material_mg[Black][Queen.index()] = f64::from_str(qs[2].as_str()).unwrap();
    eval.material_eg[Black][Queen.index()] = f64::from_str(qs[3].as_str()).unwrap();

    Ok((output_str, eval))
    // unimplemented!()
}


/// (_, ((ns0,nodes0),(ns1,nodes1)))
/// ns0    = total nodes found
/// nodes0 = HashMap<Move String, (Move, nodes after Move)>
pub fn test_stockfish(
    ts:         &Tables,
    fen:        &str,
    n:          u64,
    print:      bool,
    print_all:  bool,
) -> std::io::Result<((f64,f64),((u64,HashMap<String,(Move,i64)>),(u64,HashMap<String,i64>)))> {

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

    let now     = std::time::Instant::now();
    let output  = child.wait_with_output()?;
    let done_sf = now.elapsed().as_secs_f64();
    // let done_sf = 0.;

    let mut g = Game::from_fen(ts, fen).unwrap();

    let now = std::time::Instant::now();

    // let (ns0, ms) = g.perft(&ts, n);
    let (ns0,ms) = MoveGen::perft(ts, &g, n as Depth);

    let done = now.elapsed().as_secs_f64();
    // println!("perft done in {} seconds.", now.elapsed().as_secs_f64());

    let mut nodes0: HashMap<String, (Move,i64)> = HashMap::new();
    for (m,k) in ms.into_iter() {
        match m {
            Move::Promotion { .. } | Move::PromotionCapture { .. } => {
                // let new_piece = m.piece().unwrap();
                let new_piece = match m {
                    Move::Promotion { new_piece, .. }  => new_piece,
                    Move::PromotionCapture { pcs, .. } => pcs.first(),
                    _                                  => unimplemented!(),
                };
                let c = match new_piece {
                    Queen  => 'q',
                    Knight => 'n',
                    Rook   => 'r',
                    Bishop => 'b',
                    _      => panic!("Bad promotion: mv = {:?}", m),
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
                    if print_all {
                        eprintln!("k0, rchess, stockfish = {:?} / {:>4} / ---- / failed", k, v0);
                    }
                },
                (None,Some(v1))         => {
                    if print_all {
                        eprintln!("k0, rchess, stockfish = {:?} / ---- / {:>4} / failed", k, v1);
                    }
                },
                (Some((_,v0)),Some(v1)) => {
                    if print_all {
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

        Ok(((done,done_sf),((ns0, nodes0), (ns1, nodes1))))
    } else {
        let err = String::from_utf8(output.stderr).unwrap();
        // error_chain::bail!("External command failed:\n {}", err)
        panic!("wat")
    }


}



