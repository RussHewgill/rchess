
use crate::types::*;
use crate::tables::*;

use std::str::FromStr;
use std::collections::HashMap;
use std::io::Write;
use std::process::{Command,Stdio};

pub fn test_stockfish(fen: &str, n: u64) -> std::io::Result<()> {

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
    eprintln!("g = {:?}", g);

    let (ns, ms) = g.perft(&ts, n);

    let mut nodes0: HashMap<String, i64> = HashMap::new();
    for (m,k) in ms.into_iter() {
        let m = format!("{:?}{:?}", m.sq_from(), m.sq_to()).to_ascii_lowercase();
        nodes0.insert(m, k as i64);
    }

    if output.status.success() {
        let raw_output = String::from_utf8(output.stdout).unwrap();
        let mut out = raw_output.lines();
        out.next().unwrap();
        let mut out: Vec<&str> = out.collect();
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

        for (k0,v0) in nodes0.iter() {
            let v1 = nodes1.get(k0).unwrap();

            if v0 == v1 {
                eprintln!("k0, rchess, stockfish = {:?} / {:>4} / {:>4}", k0, v0, v1);
            } else {
                eprintln!("k0, rchess, stockfish = {:?} / {:>4} / {:>4} / failed ({})",
                          k0, v0, v1, v0 - v1);
            }

            // assert!(v0 == v1);

        }

        // let words = raw_output.split_whitespace()
        //     .map(|s| s.to_lowercase())
        //     .collect::<HashSet<_>>();
        // println!("Found {} unique words:", words.len());
        // println!("{:#?}", words);

        Ok(())
    } else {
        let err = String::from_utf8(output.stderr).unwrap();
        // error_chain::bail!("External command failed:\n {}", err)
        panic!("wat")
    }


}



