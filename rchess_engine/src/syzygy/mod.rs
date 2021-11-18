
#[macro_use]
pub mod sz_errors;
pub mod sz_format;

use self::sz_format::*;
use self::sz_errors::*;

use crate::tables::*;
use crate::types::*;

use std::io;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::str::FromStr;

use itertools::Itertools;
use positioned_io::{RandomAccessFile};
use once_cell::sync::OnceCell;
use arrayvec::ArrayVec;

/// A collection of tables.
#[derive(Debug)]
pub struct SyzygyTB {
    wdl: HashMap<Material, (PathBuf, OnceCell<WdlTable<RandomAccessFile>>)>,
    dtz: HashMap<Material, (PathBuf, OnceCell<DtzTable<RandomAccessFile>>)>,
}

impl SyzygyTB {
    pub fn new() -> Self {
        Self {
            wdl: HashMap::with_capacity_and_hasher(145, Default::default()),
            dtz: HashMap::with_capacity_and_hasher(145, Default::default()),
        }
    }

    /// Add all relevant tables from a directory.
    ///
    /// Tables are selected by filename, e.g. `KQvKP.rtbz`. The files are not
    /// actually opened. This happens lazily when probing.
    ///
    /// Note that probing generally requires tables for the specific material
    /// composition, as well as material compositions that are transitively
    /// reachable by captures and promotions. These are sometimes distributed
    /// separately, so make sure to add tables from all relevant directories.
    ///
    /// Returns the number of added table files.
    ///
    /// # Errors
    ///
    /// Returns an error result when:
    ///
    /// * The `path` does not exist.
    /// * `path` is not a directory.
    /// * The process lacks permissions to list the directory.
    pub fn add_directory<P: AsRef<Path>>(&mut self, path: P) -> io::Result<usize> {
        let mut num = 0;

        for entry in std::fs::read_dir(path)? {
            if self.add_file(entry?.path()).is_ok() {
                num += 1;
            }
        }

        Ok(num)
    }

    /// Add a table file.
    ///
    /// The file is not actually opened. This happens lazily when probing.
    ///
    /// # Errors
    ///
    /// Returns an error when no file exists at the given path or the
    /// filename does not indicate that it is a valid table file
    /// (e.g. `KQvKP.rtbz`).
    pub fn add_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let path = path.as_ref();

        if !path.is_file() {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }

        let (stem, ext) = match (path.file_stem().and_then(|s| s.to_str()), path.extension()) {
            (Some(stem), Some(ext)) => (stem, ext),
            _                       => return Err(io::Error::from(io::ErrorKind::InvalidInput)),
        };

        let material = match Material::from_str(stem) {
            Some(material) => material,
            _              => return Err(io::Error::from(io::ErrorKind::InvalidInput)),
        };
        // let material = Material::from_str(stem);

        if material.count() as usize > MAX_PIECES {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }

        if material.count_side(White) < 1 || material.count_side(Black) < 1 {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }

        // if ext == TBW.ext || (!material.has_pawns() && PAWNLESS_TBW.map_or(false, |t| ext == t.ext)) {
        if ext == TBW.ext {
            self.wdl.insert(material, (path.to_path_buf(), OnceCell::new()));
        // } else if ext == TBZ.ext || (!material.has_pawns() && PAWNLESS_TBZ.map_or(false, |t| ext == t.ext)) {
        } else if ext == TBZ.ext {
            self.dtz.insert(material, (path.to_path_buf(), OnceCell::new()));
        } else {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }

        Ok(())
    }

}

impl SyzygyTB {

    pub fn probe<'a>(&'a self, ts: &'a Tables, g: &'a Game) -> SyzygyResult<WdlEntry<'a>> {
        if g.all_occupied().count() > MAX_PIECES {
            return Err(SyzygyError::TooManyPieces);
        }
        if g.state.castling.any() {
            return Err(SyzygyError::Castling);
        }

        // Determine the WDL value of this position. This is also a
        // prerequisite for probing DTZ tables. There are two complications:
        //
        // (1) Resolving en passant captures.
        //
        // (2) When a position has a capture that achieves a particular result
        //     (e.g. there is a winning capture), then the position itself
        //     should have at least that value (e.g. it is winning). In this
        //     case the table can store an arbitrary lower value, whichever is
        //     best for compression.
        //
        //     If the best move is zeroing, then we need remember this to avoid
        //     probing the DTZ tables.

        // Resolve captures: Find the best non-ep capture and the best
        // en passant capture.
        let mut best_capture = Wdl::Loss;
        let mut best_ep = Wdl::Loss;

        // let legals = g.search_all(&ts).get_moves_unsafe();
        let legals: Vec<Move> = match g.search_all(&ts) {
            Outcome::Moves(ms)    => ms,
            // Outcome::Checkmate(w) => return Err(SyzygyError::Checkmate(w)),
            // Outcome::Stalemate    => return Err(SyzygyError::Stalemate),
            Outcome::Checkmate(w) => vec![],
            Outcome::Stalemate    => vec![],
        };

        let gs = legals.iter().filter(|m| m.filter_all_captures()).flat_map(|&mv| {
            if let Ok(g2) = g.make_move_unchecked(ts, mv) {
                let mvs = g2.search_all(ts);
                if mvs.is_end() { None } else {
                    Some((mv,g2))
                }
            } else { None }
        });
        for (mv,after) in gs {
            // eprintln!("mv = {:?}\n{:?}", mv, g);
            let v = -self.probe_ab_no_ep(ts, &after, Wdl::Loss, -best_capture)?;
            if v == Wdl::Win {
                return Ok(WdlEntry {
                    tablebase: self,
                    g,
                    wdl: v,
                    state: ProbeState::ZeroingBestMove,
                });
            }
            if mv.filter_en_passant() {
                best_ep = std::cmp::max(best_ep, v);
            } else {
                best_capture = std::cmp::max(best_capture, v);
            }
        }

        // for &mv in legals.iter().filter(|m| m.filter_all_captures()) {
        //     let after = g.make_move_unchecked(ts, mv).unwrap();
        //     let v = -self.probe_ab_no_ep(ts, &after, Wdl::Loss, -best_capture)?;
        //     if v == Wdl::Win {
        //         return Ok(WdlEntry {
        //             tablebase: self,
        //             g,
        //             wdl: v,
        //             state: ProbeState::ZeroingBestMove,
        //         });
        //     }
        //     if mv.filter_en_passant() {
        //         best_ep = std::cmp::max(best_ep, v);
        //     } else {
        //         best_capture = std::cmp::max(best_capture, v);
        //     }
        // }

        // Probe table.
        let v = self.probe_wdl_table(g)?;

        // Now max(v, best_capture) is the WDL value of the position without
        // ep rights. Detect the case were an ep move is stricly better
        // (including blessed losing positions).
        if best_ep > std::cmp::max(v, best_capture) {
            return Ok(WdlEntry {
                tablebase: self,
                g,
                wdl: best_ep,
                state: ProbeState::ZeroingBestMove,
            });
        }

        best_capture = std::cmp::max(best_capture, best_ep);

        // Now max(v, best_capture) is the WDL value of the position,
        // unless the position without ep rights is stalemate (and there are
        // ep moves).
        if best_capture >= v {
            return Ok(WdlEntry {
                tablebase: self,
                g,
                wdl: best_capture,
                state: if best_capture > Wdl::Draw { ProbeState::ZeroingBestMove } else { ProbeState::Normal },
            })
        }

        // If the position would be stalemate without ep captures, then we are
        // forced to play the best en passant move.
        if v == Wdl::Draw && !legals.is_empty() && legals.iter().all(|m| m.filter_en_passant()) {
            return Ok(WdlEntry {
                tablebase: self,
                g,
                wdl: best_ep,
                state: ProbeState::ZeroingBestMove,
            })
        }

        Ok(WdlEntry {
            tablebase: self,
            g,
            wdl: v,
            state: ProbeState::Normal,
        })
    }

    fn probe_ab_no_ep(&self, ts: &Tables, g: &Game, mut alpha: Wdl, beta: Wdl) -> SyzygyResult<Wdl> {
        // Use alpha-beta to recursively resolve captures. This is only called
        // for positions without ep rights.
        assert!(g.state.en_passant.is_none());

        let mvs = g.search_all(ts).get_moves_unsafe();
        let mvs = mvs.into_iter().filter(|m| m.filter_all_captures());

        let gs = mvs.flat_map(|mv| {
            if let Ok(g2) = g.make_move_unchecked(ts, mv) {
                let mvs = g2.search_all(ts);
                if mvs.is_end() { None } else {
                    Some((mv,g2))
                }
            } else { None }
        });
        // eprintln!("gs.count() = {:?}", gs.clone().count());

        for (mv,after) in gs {
            let v = -self.probe_ab_no_ep(ts, &after, -beta, -alpha)?;
            if v >= beta {
                return Ok(v);
            }
            alpha = std::cmp::max(alpha, v);
        }

        // for mv in mvs {
        //     let after = g.make_move_unchecked(ts, mv).unwrap();
        //     let v = -self.probe_ab_no_ep(ts, &after, -beta, -alpha)?;
        //     if v >= beta {
        //         return Ok(v);
        //     }
        //     alpha = std::cmp::max(alpha, v);
        // }

        println!("wat pre probe_ab_no_ep");
        let v = self.probe_wdl_table(g)?;
        Ok(std::cmp::max(alpha, v))
    }

    /// Probe tables for the [`Wdl`] value of a position.
    ///
    /// This indicates if the position is winning, lost or drawn with
    /// or without the 50-move rule, assuming `pos` is reached directly after
    /// a capture or pawn move.
    ///
    /// # Errors
    ///
    /// See [`SyzygyError`] for possible error
    /// conditions.
    pub fn probe_wdl(&self, ts: &Tables, g: &Game) -> SyzygyResult<Wdl> {
        self.probe(ts, g).map(|entry| entry.wdl())
    }

    /// Probe tables for the [`Dtz`] value of a position.
    ///
    /// Min-maxing the DTZ of the available moves guarantees achieving the
    /// optimal outcome under the 50-move rule.
    ///
    /// Requires both WDL and DTZ tables.
    ///
    /// # Errors
    ///
    /// See [`SyzygyError`] for possible error
    /// conditions.
    pub fn probe_dtz(&self, ts: &Tables, g: &Game) -> SyzygyResult<Dtz> {
        self.probe(ts, g).and_then(|entry| entry.dtz(ts))
    }

    fn probe_wdl_table(&self, g: &Game) -> SyzygyResult<Wdl> {
        // Test for KvK.
        if g.get_piece(King) == g.all_occupied() {
            return Ok(Wdl::Draw);
        }

        // Get raw WDL value from the appropriate table.
        let key = g.state.material;
        self.wdl_table(&key)
            .and_then(|table| table.probe_wdl(g).ctx(Metric::Wdl, &key))
    }

    pub fn probe_dtz_table(&self, g: &Game, wdl: DecisiveWdl) -> SyzygyResult<Option<Dtz>> {
        // Get raw DTZ value from the appropriate table.
        let key = g.state.material;
        self.dtz_table(&key)
            .and_then(|table| table.probe_dtz(g, wdl).ctx(Metric::Dtz, &key))
    }

    fn wdl_table(&self, key: &Material) -> SyzygyResult<&WdlTable<RandomAccessFile>> {
        if let Some(&(ref path, ref table)) = self.wdl.get(key).or_else(|| self.wdl.get(&key.clone().into_flipped())) {
            table.get_or_try_init(|| WdlTable::open(path, key)).ctx(Metric::Wdl, key)
        } else {
            Err(SyzygyError::MissingTable {
                metric: Metric::Wdl,
                material: key.clone().into_normalized(),
            })
        }
    }

    fn dtz_table(&self, key: &Material) -> SyzygyResult<&DtzTable<RandomAccessFile>> {
        if let Some(&(ref path, ref table)) = self.dtz.get(key).or_else(|| self.dtz.get(&key.clone().into_flipped())) {
            table.get_or_try_init(|| DtzTable::open(path, key)).ctx(Metric::Dtz, key)
        } else {
            Err(SyzygyError::MissingTable {
                metric: Metric::Dtz,
                material: key.clone().into_normalized(),
            })
        }
    }

    /// Select a DTZ-optimal move.
    ///
    /// Requires both WDL and DTZ tables.
    ///
    /// # Errors
    ///
    /// See [`SyzygyError`] for possible error
    /// conditions.
    pub fn best_move(&self, ts: &Tables, g: &Game) -> SyzygyResult<Option<(Move, Dtz)>> {
        struct WithAfter<S> {
            mv: Move,
            after: S,
        }

        struct WithWdlEntry<'a> {
            mv: Move,
            entry: WdlEntry<'a>,
        }

        struct WithDtz {
            m: Move,
            immediate_loss: bool,
            zeroing: bool,
            dtz: Dtz,
        }

        // Build list of successor positions.

        let with_after = g.search_all(ts).get_moves_unsafe().into_iter().flat_map(|mv| {
            // let mut after = g.clone();
            // after.play_unchecked(&m);
            match g.make_move_unchecked(ts, mv) {
                Ok(after) => {
                    let mvs = after.search_all(&ts);
                    if mvs.is_end() { None } else {
                        Some(WithAfter { mv, after })
                    }
                },
                _         => None,
            }

        }).collect::<ArrayVec<_, 256>>();

        eprintln!("with_after.len() = {:?}", with_after.len());
        // Determine WDL for each move.
        let with_wdl = with_after.iter().map(|e| {
            eprintln!("e.mv = {:?}", e.mv);
            Ok(WithWdlEntry {
                mv: e.mv.clone(),
                entry: self.probe(ts, &e.after)?,
        })}).collect::<SyzygyResult<ArrayVec<_, 256>>>()?;

        // Find best WDL.
        let best_wdl = with_wdl.iter().map(|a| a.entry.wdl).min().unwrap_or(Wdl::Loss);

        // println!("wat 3");
        // Select a DTZ-optimal move among the moves with best WDL.
        itertools::process_results(with_wdl.iter().filter(|a| a.entry.wdl == best_wdl).map(|a| {
            let dtz = a.entry.dtz(ts)?;
            Ok(WithDtz {
                immediate_loss: dtz == Dtz(-1)
                    && a.entry.g.is_checkmate(ts),
                zeroing: a.mv.is_zeroing(),
                m: a.mv.clone(),
                dtz,
            })
        }), |iter| iter.min_by_key(|m| (
            std::cmp::Reverse(m.immediate_loss),
            m.zeroing ^ (m.dtz < Dtz(0)), // zeroing is good/bad if winning/losing
            std::cmp::Reverse(m.dtz),
        )).map(|m| (m.m, m.dtz)))
    }

}

/// Additional probe information from a brief alpha-beta search.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ProbeState {
    /// Normal probe.
    Normal,
    /// Best move is zeroing.
    ZeroingBestMove,
    /// Threatening to force a capture (in antichess variants, where captures
    /// are compulsory).
    Threat,
}

/// WDL entry. Prerequisite for probing DTZ tables.
#[derive(Debug)]
pub struct WdlEntry<'a> {
    tablebase: &'a SyzygyTB,
    g:         &'a Game,
    pub wdl:       Wdl,
    pub state:     ProbeState,
}

impl<'a> WdlEntry<'a> {
    fn wdl(&self) -> Wdl {
        self.wdl
    }

    // fn dtz(&self, ts: &Tables) -> SyzygyResult<Dtz> {
    //     return Ok(Dtz(123456));
    // }

    pub fn dtz(&self, ts: &Tables) -> SyzygyResult<Dtz> {
        let wdl = match self.wdl.decisive() {
            Some(wdl) => wdl,
            None      => return Ok(Dtz(0)),
        };

        if self.state == ProbeState::ZeroingBestMove
            || self.g.get_color(self.g.state.side_to_move) == self.g.get(Pawn, self.g.state.side_to_move) {
                println!("wat 0");
                return Ok(Dtz::before_zeroing(wdl));
        }

        if self.state == ProbeState::Threat && wdl >= DecisiveWdl::CursedWin {
            // The position is a win or a cursed win by a threat move.
            println!("wat 1");
            return Ok(Dtz::before_zeroing(wdl).add_plies(1));
        }

        // If winning, check for a winning pawn move. No need to look at
        // captures again, they were already handled above.
        if wdl >= DecisiveWdl::CursedWin {

            let mut pawn_advances = self.g.search_all(ts).get_moves_unsafe();
            pawn_advances.retain(|m| !m.filter_all_captures() && m.piece() == Some(Pawn));

            for &mv in &pawn_advances {
                // let mut after = self.g.clone();
                // after.play_unchecked(m);
                let after = self.g.make_move_unchecked(ts, mv).unwrap();
                let g = self.g.make_move_unchecked(ts, mv);
                let v = -self.tablebase.probe_wdl(ts, &after)?;
                if v == wdl.into() {
                    println!("wat 2");
                    return Ok(Dtz::before_zeroing(wdl));
                }
            }
        }

        // At this point we know that the best move is not a capture. Probe the
        // table. DTZ tables store only one side to move.
        if let Some(Dtz(dtz)) = self.tablebase.probe_dtz_table(self.g, wdl)? {
            // println!("wat 3");
            return Ok(Dtz::before_zeroing(wdl).add_plies(dtz));
        }

        // We have to probe the other side by doing a 1-ply search.
        let mut moves = self.g.search_all(ts).get_moves_unsafe();
        moves.retain(|m| !m.is_zeroing());

        let mut best = if wdl >= DecisiveWdl::CursedWin {
            None
        } else {
            Some(Dtz::before_zeroing(wdl))
        };

        for &mv in &moves {
            // let mut after = self.g.clone();
            // after.play_unchecked(m);
            let after = self.g.make_move_unchecked(ts, mv).unwrap();

            let v = -self.tablebase.probe_dtz(ts, &after)?;

            if v == Dtz(1) && after.is_checkmate(ts) {
                best = Some(Dtz(1));
            } else if wdl >= DecisiveWdl::CursedWin {
                if v > Dtz(0) && best.map_or(true, |best| v + Dtz(1) < best) {
                    best = Some(v + Dtz(1));
                }
            } else if best.map_or(true, |best| v - Dtz(1) < best) {
                best = Some(v - Dtz(1));
            }
        }

        (|| Ok(u!(best)))().ctx(Metric::Dtz, &self.g.state.material)
    }

}

// /// WDL, .rtbw: win / draw / loss
// /// DTZ, .rtbz: distance to zero
// #[derive(Debug,Eq,PartialEq,PartialOrd,Clone)]
// // #[derive(Serialize,Deserialize,Debug,Eq,PartialEq,PartialOrd,Clone)]
// pub struct SyzygyBase {
//     pub wdl:   Vec<SyzWDL>,
//     pub dtz:   Vec<SyzDTZ>,
// }

// #[derive(Debug,Eq,PartialEq,PartialOrd,Clone)]
// pub struct SyzWDL {
// }

// #[derive(Debug,Eq,PartialEq,PartialOrd,Clone)]
// pub struct SyzDTZ {
// }

// /// Probe
// impl SyzygyBase {

//     pub fn probe(&self, ts: &Tables, g: &Game) -> () {
//         unimplemented!()
//     }

//     pub fn probe_wdl(&self, ts: &Tables, g: &Game) -> () {

//         let mvs = g.search_all(ts).get_moves_unsafe();

//         unimplemented!()
//     }

// }

// /// Load
// impl SyzygyBase {

//     // const PCHR: [char; 6] = ['K', 'Q', 'R', 'B', 'N', 'P'];

//     // pub fn load_dir(ts: &Tables, dir: &str) -> std::io::Result<Self> {
//     pub fn load_dir(dir: &str) -> std::io::Result<Self> {
//         let paths = std::fs::read_dir(&dir)?;
//         let mut wdl = vec![];
//         let mut dtz = vec![];
//         for f in paths {
//             let f = f?;
//             let p = f.path();
//             match p.extension().map(|x| x.to_str()).flatten() {
//                 Some(e@"rtbw") => wdl.push((e.to_owned(),p)),
//                 Some(e@"rtbz") => dtz.push((e.to_owned(),p)),
//                 _            => {},
//             }
//         }

//         // XXX: 
//         for (ext,p) in wdl.into_iter().take(2) {
//         // for (ext,p) in wdl.into_iter() {

//             let n = p.file_name().unwrap().to_str().unwrap().replace(&ext, "").replace(".", "");
//             // eprintln!("n = {:?}", n);
//             let mut buf: Vec<u8> = std::fs::read(p)?;

//             // let n2 = Self::normalize_tablename(&n, false);
//             // Self::load_table(true, &n, buf);

//         }

//         unimplemented!()
//     }

//     // fn normalize_tablename(name: &str, mirror: bool) -> String {
//     //     // const PCHR: [char; 6] = ['K', 'Q', 'R', 'B', 'N', 'P'];
//     //     const fn f(c: char) -> usize {
//     //         match c {
//     //             'K' => 0,
//     //             'Q' => 1,
//     //             'R' => 2,
//     //             'B' => 3,
//     //             'N' => 4,
//     //             'P' => 5,
//     //             _   => unimplemented!(),
//     //         }
//     //     }
//     //     let mut xs = name.split("v");
//     //     let mut black = xs.next().unwrap();
//     //     let mut white = xs.next().unwrap();
//     //     let black = black.chars().sorted_by_key(|&c| f(c)).collect::<String>();
//     //     let white = white.chars().sorted_by_key(|&c| f(c)).collect::<String>();
//     //     let a = black.chars().map(|c| f(c)).collect::<Vec<_>>();
//     //     let b = white.chars().map(|c| f(c)).collect::<Vec<_>>();
//     //     if mirror ^ ((white.len(), a) < (black.len(), b)) {
//     //         vec![black,"v".to_string(),white].join("")
//     //     } else {
//     //         vec![white,"v".to_string(),black].join("")
//     //     }
//     // }

//     // fn load_table(is_wdl: bool, name: &str, buf: Vec<u8>) {
//     //     let n_pcs = name.len() - 1;
//     //     // eprintln!("name = {:?}", name);
//     //     let key   = Self::normalize_tablename(&name, false);
//     //     let key_m = Self::normalize_tablename(&name, true);
//     //     let has_pawns = name.contains('P');
//     //     let mut xs = name.split("v");
//     //     let black = xs.next().unwrap();
//     //     let white = xs.next().unwrap();
//     //     if has_pawns {
//     //         let mut pawns = (black.match_indices('P').count(),white.match_indices('P').count());
//     //         if pawns.1 > 0 && (pawns.0 == 0 || pawns.1 < pawns.0) {
//     //             std::mem::swap(&mut pawns.0, &mut pawns.1);
//     //         }
//     //     } else {
//     //         let mut k = 0;
//     //         for pc in Self::PCHR {
//     //             if black.match_indices(pc).count == 1 {
//     //                 k += 1;
//     //             }
//     //             if white.match_indices(pc).count == 1 {
//     //                 k += 1;
//     //             }
//     //         }
//     //         if k >= 3 {
//     //             // enc_type = 0
//     //         }
//     //     }
//     // }

// }


