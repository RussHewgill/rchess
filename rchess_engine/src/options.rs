



// pub enum EngOptType {
//     Check,
//     Spin,
//     Combo,
//     Button,
//     String,
// }

#[derive(Debug,Default,Clone)]
pub struct EngineOption {
    name:     &'static str,
    default:  Option<String>,
    min:      Option<String>,
    max:      Option<String>,
}

#[derive(Debug,Clone)]
pub struct EngineOptions {
    opts:     Vec<EngineOption>,
}

impl EngineOptions {
    pub fn new() -> Self {
        Self {
            opts: vec![],
        }
    }
}







