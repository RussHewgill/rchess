
use crate::types::*;

pub use std::sync::{Arc, atomic::AtomicBool};
use std::time::{Duration};

#[derive(Debug,PartialEq,PartialOrd,Clone,Copy)]
pub struct TimeSettings {
    clock_time:      [Duration; 2],
    increment:       [Duration; 2],
    pub ponder:      bool,
    pub infinite:    bool,
}

#[derive(Debug,Clone)]
pub struct Timer {
    pub should_stop:     Arc<AtomicBool>,
    pub settings:        TimeSettings,
    // pub nodes:           Vec<u16>,
}

impl Timer {
    pub fn new(should_stop: Arc<AtomicBool>, settings: TimeSettings) -> Self {
        Self {
            should_stop,
            settings,
            // nodes: 
        }
    }
}

impl TimeSettings {
    pub fn new(clock_time: Duration, increment: Duration) -> Self {
        Self {
            clock_time:   [clock_time, clock_time],
            increment:    [increment, increment],
            ponder:       false,
            infinite:     false,
        }
    }
}



