

use std::{collections::HashMap, path::Path};

use std::io::Write;
use std::fs::OpenOptions;

use serde_json::{json,Value};
use serde::{Deserialize,Serialize};
use derive_new::new;
use once_cell::sync::Lazy;

#[derive(Debug,Clone)]
pub struct Engine {
    pub name:     String,
    pub command:  String,
    pub options:  HashMap<String,Value>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
struct EngineSer {
    pub name:     String,
    pub command:  String,
    pub protocol: String,
    #[serde(default)]
    options:      Vec<HashMap<String,Value>>,
}

impl Engine {

    pub fn write_option_val(&mut self, opt: &str, val: Value) {
        unimplemented!()
    }

}

/// read, write
impl Engine {

    pub fn write<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let eng_ser = self.convert_to_ser();

        let mut engines = EngineSer::_read_all_from_file(&path)?;

        let replaced = engines.insert(self.name.clone(), eng_ser);

        EngineSer::write_all(&path, engines)?;

        Ok(())
    }

    pub fn read_from_file<P: AsRef<Path>>(name: &str, path: P) -> std::io::Result<Self> {
        let engines = Self::read_all_from_file(path)?;
        if let Some(e) = engines.get(name) {
            return Ok(e.clone());
        }
        panic!("engine {} not found", name);
    }

    pub fn read_all_from_file<P: AsRef<Path>>(path: P) -> std::io::Result<HashMap<String,Self>> {
        let b = std::fs::read_to_string(path)?;
        let json: Vec<EngineSer> = serde_json::from_str(&b).unwrap();
        let mut out = HashMap::default();
        json.into_iter()
            .map(|eng| Self::convert_from_ser(eng))
            .for_each(|e| { out.insert(e.name.clone(), e); });
        Ok(out)
    }

    fn convert_to_ser(&self) -> EngineSer {
        let mut options: Vec<HashMap<String,Value>> = vec![];
        for (k,v) in self.options.iter() {
            let mut map = HashMap::default();
            map.insert("name".to_string(), Value::String(k.to_string()));
            map.insert("value".to_string(), v.clone());
            options.push(map);
        }
        EngineSer {
            name: self.name.clone(),
            command: self.command.clone(),
            protocol: "uci".to_string(),
            options,
        }
    }

    fn convert_from_ser(eng: EngineSer) -> Self {
        let mut options = HashMap::default();
        for map in eng.options.iter() {
            let name = if let Value::String(name) = map.get("name").unwrap() {
                name } else { panic!("convert_from_ser: bad json? {:?}", &eng) };
            let val  = map.get("value").unwrap();
            options.insert(name.clone(), val.clone());
        }
        Self {
            name:     eng.name,
            command:  eng.command,
            options,
        }
    }

}

impl EngineSer {

    fn write_all<P: AsRef<Path>>(path: P, engines: HashMap<String,Self>) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .truncate(true)
            .read(true)
            .create(true)
            .write(true)
            .open(path)
            .unwrap();
        let engines: Vec<Self> = engines.into_iter().map(|x| x.1).collect();
        serde_json::to_writer_pretty(file, &engines)?;
        Ok(())
    }

    fn _read_all_from_file<P: AsRef<Path>>(path: P) -> std::io::Result<HashMap<String,EngineSer>> {
        let b = std::fs::read_to_string(path)?;
        let json: Vec<EngineSer> = serde_json::from_str(&b).unwrap();
        let mut out = HashMap::default();
        json.into_iter().for_each(|e| { out.insert(e.name.clone(), e); });
        Ok(out)
    }
}

/// new
impl Engine {

    pub fn new(name: String, command: String, options: HashMap<String,Value>) -> Self {
        Self {
            name,
            command,
            options,
        }
    }

    pub fn new_tuner(id: usize, options: HashMap<String,Value>) -> Self {
        Self::new(
            format!("rchess_tuning_{}", id),
            "/home/me/code/rust/rchess/target/release/rchess_uci_tuning".to_string(),
            options,
        )
    }
}




