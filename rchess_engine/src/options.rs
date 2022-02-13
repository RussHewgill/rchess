
use crate::{explore::{Explorer, ExConfig}, tables::SParams};

use std::{str::FromStr, collections::HashMap};
use log::debug;


// pub enum EngOptType {
//     Check,
//     Spin,
//     Combo,
//     Button,
//     String,
// }

#[derive(Clone)]
pub struct EngineOption {
    name:     &'static str,
    default:  Option<i64>,
    min:      Option<i64>,
    max:      Option<i64>,
    func:     fn(&mut SParams, &mut ExConfig, i64),
}

impl std::fmt::Debug for EngineOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)?;
        Ok(())
    }
}

/// print
impl EngineOption {
    pub fn print(&self) {
        print!("option name {}",
                 self.name
        );
        print!(" type spin");
        if let Some(d) = self.default { print!(" default {}", d); }
        if let Some(m) = self.min { print!(" min {}", m); }
        if let Some(m) = self.max { print!(" max {}", m); }

        println!();
    }
}

#[derive(Debug,Clone)]
pub struct EngineOptions {
    // opts:     Vec<EngineOption>,
    opts:     HashMap<String, EngineOption>,
}

/// set
impl Explorer {
    pub fn set_option(&mut self, name: &str, val: &str) {

        let val = i64::from_str(val).unwrap();

        if let Some(opt) = self.options.get(name) {
            (opt.func)(&mut self.search_params, &mut self.cfg, val);
            self.sync_threads();
        } else {
            debug!("no option: {:?} = {:?}", name, val);
        }
    }
}

/// print
impl EngineOptions {
    pub fn print(&self) {
        for (_,opt) in self.opts.iter() {
            opt.print();
        }
    }
}

/// get, insert
impl EngineOptions {
    pub fn get(&self, name: &str) -> Option<&EngineOption> {
        self.opts.get(name)
    }
    pub fn insert(&mut self, opt: EngineOption) {
        self.opts.insert(opt.name.to_string(), opt);
    }
}

/// new
impl EngineOptions {
    pub fn new() -> Self {
        let mut out = Self {
            opts: HashMap::default(),
        };

        out.insert(EngineOption {
            name:    "num_threads",
            default: Some(0),
            min:     Some(0),
            max:     Some(num_cpus::get() as i64),
            func:    opt_num_threads,
        });

        out.insert(EngineOption {
            name:    "lmr_reduction",
            default: Some(3),
            min:     Some(2),
            max:     Some(5),
            func:    opt_lmr_reduction,
        });

        out
    }
}

fn opt_num_threads(sp: &mut SParams, cfg: &mut ExConfig, val: i64) {
    if val <= 0 {
        cfg.num_threads = None;
    } else {
        cfg.num_threads = Some(val as u16);
    }
}

fn opt_lmr_reduction(sp: &mut SParams, cfg: &mut ExConfig, val: i64) {
    sp.lmr_reduction = val as i16;
}






