
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
    // pub ponder:      bool,
    // pub infinite:    bool,
}

#[derive(Debug,Clone)]
pub struct Timer {
    // should_stop:         Arc<AtomicBool>,
    pub settings:        TimeSettings,
    // pub nodes:           Vec<u32>,
    // times:               Vec<f64>,
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

    // pub fn should_stop(&self) -> bool {
    //     self.should_stop.load(Ordering::Relaxed)
    // }

    // pub fn set_should_stop(&mut self, b: bool) {
    //     self.should_stop.store(b, Ordering::Relaxed);
    // }

    // pub fn update_times(&mut self, side: Color, nodes: u32) {
    //     // self.nodes.push(nodes);
    //     let dt = self.elapsed_f64();
    //     // self.times.push(dt);
    //     self.time_left[side] -= dt;
    // }

    pub fn allocate_time(&self) -> Option<Duration> {
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
            // ponder:       false,
            // infinite:     false,
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



