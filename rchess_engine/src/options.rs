
use crate::{explore::Explorer, tables::SParams};

use std::{str::FromStr, collections::HashMap};


// pub enum EngOptType {
//     Check,
//     Spin,
//     Combo,
//     Button,
//     String,
// }

// #[derive(Debug,Default,Clone)]
#[derive(Clone)]
pub struct EngineOption {
    name:     &'static str,
    default:  Option<u64>,
    min:      Option<u64>,
    max:      Option<u64>,
    // step:     Option<u64>,
    // func:     Option<Box<dyn FnMut(SParams, u64)>>,
    func:     fn(&mut SParams, u64),
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

        let val = u64::from_str(val).unwrap();

        if let Some(opt) = self.options.get(name) {
            (opt.func)(&mut self.search_params, val);
            self.sync_threads();
        } else {
            panic!();
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

/// new, get, insert
impl EngineOptions {

    pub fn get(&self, name: &str) -> Option<&EngineOption> {
        self.opts.get(name)
    }

    pub fn insert(&mut self, opt: EngineOption) {
        self.opts.insert(opt.name.to_string(), opt);
    }

    pub fn new() -> Self {
        let mut out = Self {
            opts: HashMap::default(),
        };

        out.insert(EngineOption {
            name:    "lmr_reduction",
            // default: Some(6),
            default: Some(3),
            min:     Some(2),
            // max:     Some(10),
            max:     Some(5),
            func:    opt_lmr_reduction,
            // func:    Some(Box::new(|sp, val| {
            //     sp.lmr_reduction = val as i16;
            // })),
        });

        out
    }
}

fn opt_lmr_reduction(sp: &mut SParams, val: u64) {
    sp.lmr_reduction = val as i16;
}






