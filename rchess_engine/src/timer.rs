
use crate::types::*;

pub use self::old::*;

pub use std::sync::{
    Arc,
    atomic::{AtomicBool,Ordering},
};
use std::time::{Instant,Duration};

// #[derive(Debug,PartialEq,PartialOrd,Clone,Copy)]
// pub struct TimeRemaining {
//     remaining:     [Duration; 2],
//     increment:     [Duration; 2],
//     moves_to_go:   Option<u32>,
// }

// impl TimeRemaining {
//     pub fn new(time: Duration, inc: Duration, mtg: Option<u32>) -> Self {
//         Self {
//             remaining:    [time, time],
//             increment:    [inc, inc],
//             moves_to_go:  mtg,
//         }
//     }
//     pub fn get_rem_inc(&self, side: Color) -> (Duration,Duration) {
//         (self.remaining[side],self.increment[side])
//     }

//     pub fn get_rem_inc_f64(&self, side: Color) -> (f64,f64) {
//         let (t, inc) = self.get_rem_inc(side);
//         (t.as_secs_f64(), inc.as_secs_f64())
//     }

// }

// #[derive(Debug,PartialEq,PartialOrd,Clone,Copy)]
// pub struct TimeManager {
//     start_time:      Instant,
//     depth:           Depth,
//     hard_limit:      Duration,
//     allotted_time:   Duration,
//     must_play:       bool,
// }

// impl TimeManager {
//     pub fn init(side: Color, ply: Depth, time: &TimeRemaining) -> Self {

//         let (t, inc) = time.get_rem_inc_f64(side);
//         let mtg = time.moves_to_go.unwrap_or(40) as f64;

//         let time_avail = t + inc * (mtg - 1.0);

//         let hard_limit = Duration::from_secs_f64(10.0);

//         let allotted_time = Duration::from_secs_f64(time_avail / mtg);

//         Self {
//             start_time:      Instant::now(),
//             depth:           0,
//             hard_limit,
//             allotted_time,
//             must_play:       false,
//         }
//     }
// }

mod old {
    use crate::types::*;

    pub use std::sync::{
        Arc,
        atomic::{AtomicBool,Ordering},
    };
    use std::time::{Instant,Duration};

    pub type Seconds = f64;

    #[derive(Debug,PartialEq,PartialOrd,Clone,Copy)]
    pub struct TimeSettings {
        clock_time:      [Seconds; 2],
        pub increment:   [Seconds; 2],
        pub safety:      Seconds,
        pub ponder:      bool,
        pub infinite:    bool,
    }

    #[derive(Debug,Clone)]
    pub struct Timer {
        pub settings:        TimeSettings,
        // pub nodes:           Vec<u32>,
        // times:               Vec<f64>,
        pub moves_to_go:     Option<u32>,
        pub time_left:       [Seconds; 2],
        pub init:            Instant,
    }

    impl Timer {
        // pub fn new(side: Color, should_stop: Arc<AtomicBool>, settings: TimeSettings) -> Self {
        pub fn new(settings: TimeSettings) -> Self {
            Self {
                // should_stop,
                settings,
                // nodes:         vec![],
                // times:         vec![],
                moves_to_go:   None,
                time_left:     settings.clock_time,
                init:          Instant::now(),
            }
        }

        pub fn reset(&mut self) {
            // self.set_should_stop(false);
            self.time_left = self.settings.clock_time;
            self.init = Instant::now();
        }

        pub fn elapsed(&self) -> Duration {
            self.init.elapsed()
        }
        pub fn elapsed_f64(&self) -> Seconds {
            self.init.elapsed().as_secs_f64()
        }

        pub fn allocate_time(&self, side: Color, ply: Depth) -> Duration {
            let t   = self.time_left[side];
            let inc = self.settings.increment[side];

            let mtg = self.moves_to_go.unwrap_or(50).min(50) as f64;

            let time_left = t + inc * (mtg - 1.0);
            assert!(time_left > 0.0);

            let (opt,max) = if let Some(_) = self.moves_to_go {
                let opt = f64::min(
                    0.0084 + (ply as f64 + 3.0).sqrt() * 0.0042,
                    0.2 * t / time_left
                );
                let max = f64::min(7.0, 4.0 + ply as f64 / 12.0);
                (opt,max)
            } else {
                let opt = f64::min(
                    (0.88 + ply as f64 / 116.4) / mtg,
                    0.88 * t / time_left
                );
                let max = f64::min(6.3, 1.5 + 0.11 * mtg);

                (opt,max)
            };

            unimplemented!()
        }

        pub fn estimate_remaining(&self, side: Color, mtg: u32, ideal: bool) -> Duration {

            unimplemented!()
        }

        pub fn should_search(&self, side: Color, depth: Depth) -> bool {
            unimplemented!()
        }

        // fn should_search2(&self, side: Color, depth: Depth) -> bool {
        //     if depth <= 2 { return true; }
        //     // let d = depth as usize;

        //     // let n0 = self.nodes[d-1];
        //     // let n1 = self.nodes[d-2];
        //     // let k = n0 as f64 / n1 as f64;
        //     // eprintln!("k = {:?}", k);
        //     // let rate = (n0 - n1) as f64 / self.times[d-1];
        //     // // eprintln!("n0 = {:?}", n0);
        //     // // eprintln!("n1 = {:?}", n1);
        //     // eprintln!("rate = {:?} nodes / s", rate);
        //     // // let rate = self.nodes[d-1];
        //     // if depth <= 3 { return true; }

        //     let alloc_time = (1.0 - self.settings.safety) * self.settings.clock_time[side] / 40.0
        //         + self.settings.increment[side];

        //     debug!("alloc_time = {:?}", alloc_time);

        //     self.time_left[side] > alloc_time

        //     // false

        // }

        #[cfg(feature = "nope")]
        /// https://github.com/Johnson-A/Crabby/blob/master/src/timer.rs
        fn should_search3(&self, side: Color, depth: Depth) -> bool {
            if depth <= 2 { return true; }

            // let d = (depth - 1) as usize;
            let d = depth as usize;
            let estimate = self.times[d - 1] * self.nodes[d-1] as f64 / self.nodes[d-2] as f64;


            let alloc_time = (1.0 - self.settings.safety) * self.settings.clock_time[side] / 40.0
                + self.settings.increment[side];

            // eprintln!("estimate = {:?}", estimate);
            // eprintln!("alloc_time = {:?}", alloc_time);
            // eprintln!("times[d-1], estimate * 0.3 = {:?}, {:?}", self.times[d-1], estimate * 0.3);
            // eprintln!("elapsed = {:?}", self.elapsed_f64());

            // !self.should_stop() && (self.time_left[side] > 0.0)

            (alloc_time - self.times[d-1] > estimate * 0.3)
                || (alloc_time / 1.5 > self.elapsed_f64())

        }

    }

    impl TimeSettings {

        pub fn new_f64(clock_time: f64, increment: f64) -> Self {
            Self {
                clock_time:   [clock_time; 2],
                increment:    [increment; 2],
                safety:       0.1,
                ponder:       false,
                infinite:     false,
            }
        }

        // pub fn new(clock_time: f64, increment: Duration) -> Self {
        //     Self {
        //         clock_time:   [clock_time; 2],
        //         increment:    [increment; 2],
        //         ponder:       false,
        //         infinite:     false,
        //     }
        // }

    }


}


