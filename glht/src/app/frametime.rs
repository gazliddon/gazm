use std::time::{Duration, Instant};
#[derive(Copy, Clone)]
pub  struct FrameTime {
    base_time: Instant,
    last_sync: Instant,
    last_dt: Duration,
}

#[allow(dead_code)]
impl FrameTime {
    pub fn from_now() -> Self {
        Self {
            base_time: Instant::now(),
            last_sync: Instant::now(),
            last_dt: Duration::new(0, 0),
        }
    }

    pub fn now_as_duration(&self) -> Duration {
        self.last_sync - self.base_time
    }

    pub fn dt(&self) -> f64 {
        self.last_dt.as_secs_f64()
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let dt = now - self.last_sync;
        self.last_dt = dt;
        self.last_sync = now;
    }
}

impl Default for FrameTime {
    fn default() -> Self {
        FrameTime::from_now()
    }
}

////////////////////////////////////////////////////////////////////////////////
