use std::time::Instant;

// microseconds since the handle was created
pub struct MonotonicClock {
    origin: Instant,
}

impl MonotonicClock {
    pub fn new() -> Self {
        Self { origin: Instant::now() }
    }

    pub fn now_us(&self) -> u64 {
        u64::try_from(self.origin.elapsed().as_micros()).unwrap_or(u64::MAX)
    }
}

impl Default for MonotonicClock {
    fn default() -> Self {
        Self::new()
    }
}
