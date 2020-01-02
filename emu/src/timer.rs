use std::time::{Instant, Duration};

pub struct Timer {
    start : Instant,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            start : Instant::now()
        }
    }

    pub fn get(&mut self) -> RunTime {
        let now = Instant::now();

        let r = RunTime::from_duration(now - self.start);

        self.start = now;

        r
    }

}

pub struct RunTime {
    dur : Duration
}


impl RunTime {

    pub fn from_duration(dur : Duration) -> RunTime {
        RunTime {
           dur 
        }
    }

    pub fn nanos(&self) -> f64 {
        f64::from(self.dur.subsec_nanos())
    }

    pub fn secs(&self) -> f64 {
        let nanos = self.nanos();
        nanos / 1_000_000_000.0
    }

    pub fn millis(&self) -> f64 {
        let nanos = self.nanos();
        nanos / 1_000_000.0
    }
    pub fn micros(&self) -> f64 {
        let nanos = self.nanos();
        nanos / 1_000.0
    }
}

