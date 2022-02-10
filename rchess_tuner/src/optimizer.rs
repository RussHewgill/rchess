

use crate::tuner_types::*;
use crate::supervisor::{Supervisor,Tunable, CuteChess};


impl Supervisor {
    pub fn find_optimum(&mut self) {
        debug!("starting find_optimum, param: {}", &self.tunable.opt.name);

        let timecontrol = TimeControl::new_f64(1.0, 0.1);

        let output_label = format!("{}", &self.tunable.opt.name);

        let (elo0,elo1) = (0,50);
        let num_games = 50;

        let cutechess = CuteChess::run_cutechess(
            &self.engine_tuning.name,
            &self.engine_baseline.name,
            timecontrol,
            &self.tunable.opt.name,
            num_games,
            (elo0,elo1),
            0.05);

        unimplemented!()
    }
}



