

use crate::tuner_types::*;
use crate::supervisor::{Supervisor,Tunable};


impl Supervisor {
    pub fn find_optimum(&mut self) {
        debug!("starting find_optimum, param: {}", &self.tunable.opt.name);

        let output_label = format!("", &self.tunable.opt.name);

        let (elo0,elo1) = (0,50);
        let num_games = 50;



        unimplemented!()
    }
}



