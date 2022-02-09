

use std::collections::HashMap;

use std::io::Write;
use std::fs::OpenOptions;

use serde_json::{json,Value};
use serde::{Deserialize,Serialize};
use derive_new::new;
use once_cell::sync::Lazy;

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Engine {
    pub name:     String,
    pub command:  String,
    pub protocol: String,
    #[serde(default)]
    pub options:  Vec<HashMap<String,Value>>,
}

impl Engine {
    pub fn new(name: String, command: String, options: Vec<(String, Value)>) -> Self {
        let mut maps: Vec<HashMap<String,Value>> = vec![];
        for (k,v) in options.into_iter() {
            let mut map = HashMap::default();
            map.insert("name".to_string(), Value::String(k));
            map.insert("value".to_string(), v);
            maps.push(map);
        }
        Self {
            name,
            command,
            protocol: "uci".to_string(),
            options: maps,
        }
    }
}

impl Engine {
    pub fn new_tuner(id: usize, options: Vec<(String, Value)>) -> Self {
        Self::new(
            format!("rchess_tuning_{}", id),
            "/home/me/code/rust/rchess/target/release/rchess_uci_tuning".to_string(),
            options,
        )
    }
}

pub fn json_test() {

    let path = "engines-test.json";

    // let b = std::fs::read_to_string("engines.json").unwrap();
    let b = std::fs::read_to_string(path).unwrap();

    // let mut json: Vec<HashMap<String, Value>> = serde_json::from_str(&b).unwrap();
    let mut json: Vec<Engine> = serde_json::from_str(&b).unwrap();

    let e0 = &json[0];
    // eprintln!("e0 = {:?}", e0);

    // let opts = e0.options;

    // for map in e0.options.iter() {
    //     eprintln!("map = {:?}", map);
    // }

    // let opts = e0.get("options");

    // eprintln!("opts = {:?}", opts);

    // for x in opts.iter() {
    //     eprintln!("x = {:?}", x);
    // }

    // for x in json.iter() {
    //     // eprintln!("x = {:?}", x);
    //     let n = x.get("name").unwrap();
    //     eprintln!("n = {:?}", n);
    // }

    let engine = Engine::new_tuner(0, vec![("wat".to_string(), json!(true))]);

    json.push(engine);
    let mut file = OpenOptions::new()
        .truncate(true)
        .read(true)
        .create(true)
        .write(true)
        .open(path)
        .unwrap();
    serde_json::to_writer_pretty(file, &json).unwrap();

}



