

use std::collections::HashMap;

use serde_json::Value;
use serde::{Deserialize,Serialize};
use derive_new::new;

#[derive(Debug,Clone,Serialize,Deserialize,new)]
pub struct Engine {
    name:     String,
    command:  String,
    options:  Vec<(String,Value)>,
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

    let b = std::fs::read_to_string("engines.json").unwrap();

    // let mut json: Vec<HashMap<String, Value>> = serde_json::from_str(&b).unwrap();
    let mut json: Vec<Engine> = serde_json::from_str(&b).unwrap();

    let e0 = &json[0];

    eprintln!("e0 = {:?}", e0);

    // for x in json.iter() {
    //     // eprintln!("x = {:?}", x);
    //     let n = x.get("name").unwrap();
    //     eprintln!("n = {:?}", n);
    // }

    // let engine = Engine::new_tuner(0, vec![]);

    // json.push()

}



