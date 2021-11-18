
mod sz_format;

use self::sz_format::*;

use crate::tables::*;
use crate::types::*;

use itertools::Itertools;


/// WDL, .rtbw: win / draw / loss
/// DTZ, .rtbz: distance to zero
#[derive(Debug,Eq,PartialEq,PartialOrd,Clone)]
// #[derive(Serialize,Deserialize,Debug,Eq,PartialEq,PartialOrd,Clone)]
pub struct SyzygyBase {
    pub wdl:   Vec<SyzWDL>,
    pub dtz:   Vec<SyzDTZ>,
}

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone)]
pub struct SyzWDL {
}

#[derive(Debug,Eq,PartialEq,PartialOrd,Clone)]
pub struct SyzDTZ {
}

/// Probe
impl SyzygyBase {

    pub fn probe(&self, ts: &Tables, g: &Game) -> () {
        unimplemented!()
    }

    pub fn probe_wdl(&self, ts: &Tables, g: &Game) -> () {

        let mvs = g.search_all(ts).get_moves_unsafe();

        unimplemented!()
    }

}

/// Load
impl SyzygyBase {

    // const PCHR: [char; 6] = ['K', 'Q', 'R', 'B', 'N', 'P'];

    // pub fn load_dir(ts: &Tables, dir: &str) -> std::io::Result<Self> {
    pub fn load_dir(dir: &str) -> std::io::Result<Self> {
        let paths = std::fs::read_dir(&dir)?;
        let mut wdl = vec![];
        let mut dtz = vec![];
        for f in paths {
            let f = f?;
            let p = f.path();
            match p.extension().map(|x| x.to_str()).flatten() {
                Some(e@"rtbw") => wdl.push((e.to_owned(),p)),
                Some(e@"rtbz") => dtz.push((e.to_owned(),p)),
                _            => {},
            }
        }

        // XXX: 
        for (ext,p) in wdl.into_iter().take(2) {
        // for (ext,p) in wdl.into_iter() {

            let n = p.file_name().unwrap().to_str().unwrap().replace(&ext, "").replace(".", "");
            // eprintln!("n = {:?}", n);
            let mut buf: Vec<u8> = std::fs::read(p)?;

            // let n2 = Self::normalize_tablename(&n, false);
            // Self::load_table(true, &n, buf);

        }

        unimplemented!()
    }

    // fn normalize_tablename(name: &str, mirror: bool) -> String {
    //     // const PCHR: [char; 6] = ['K', 'Q', 'R', 'B', 'N', 'P'];
    //     const fn f(c: char) -> usize {
    //         match c {
    //             'K' => 0,
    //             'Q' => 1,
    //             'R' => 2,
    //             'B' => 3,
    //             'N' => 4,
    //             'P' => 5,
    //             _   => unimplemented!(),
    //         }
    //     }
    //     let mut xs = name.split("v");
    //     let mut black = xs.next().unwrap();
    //     let mut white = xs.next().unwrap();
    //     let black = black.chars().sorted_by_key(|&c| f(c)).collect::<String>();
    //     let white = white.chars().sorted_by_key(|&c| f(c)).collect::<String>();
    //     let a = black.chars().map(|c| f(c)).collect::<Vec<_>>();
    //     let b = white.chars().map(|c| f(c)).collect::<Vec<_>>();
    //     if mirror ^ ((white.len(), a) < (black.len(), b)) {
    //         vec![black,"v".to_string(),white].join("")
    //     } else {
    //         vec![white,"v".to_string(),black].join("")
    //     }
    // }

    // fn load_table(is_wdl: bool, name: &str, buf: Vec<u8>) {
    //     let n_pcs = name.len() - 1;
    //     // eprintln!("name = {:?}", name);
    //     let key   = Self::normalize_tablename(&name, false);
    //     let key_m = Self::normalize_tablename(&name, true);
    //     let has_pawns = name.contains('P');
    //     let mut xs = name.split("v");
    //     let black = xs.next().unwrap();
    //     let white = xs.next().unwrap();
    //     if has_pawns {
    //         let mut pawns = (black.match_indices('P').count(),white.match_indices('P').count());
    //         if pawns.1 > 0 && (pawns.0 == 0 || pawns.1 < pawns.0) {
    //             std::mem::swap(&mut pawns.0, &mut pawns.1);
    //         }
    //     } else {
    //         let mut k = 0;
    //         for pc in Self::PCHR {
    //             if black.match_indices(pc).count == 1 {
    //                 k += 1;
    //             }
    //             if white.match_indices(pc).count == 1 {
    //                 k += 1;
    //             }
    //         }
    //         if k >= 3 {
    //             // enc_type = 0
    //         }
    //     }
    // }

}


