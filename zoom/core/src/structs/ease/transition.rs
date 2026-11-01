use crate::structs::EasingCurve;

// one ease of the zoom level from `from` to `to`; the level is a pure function of time
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZoomTransition {
    from: f32,
    to: f32,
    start_us: u64,
    // the full duration scaled by the distance travelled, so a reversal halfway takes half the time
    duration_us: u64,
    curve: EasingCurve,
}

impl ZoomTransition {
    pub fn new(from: f32, to: f32, start_us: u64, full_duration_us: u64, curve: EasingCurve) -> Self {
        let distance = f64::from((to - from).abs().min(1.0));
        let duration_us = (full_duration_us as f64 * distance).round() as u64;
        Self { from, to, start_us, duration_us, curve }
    }

    pub fn settled(level: f32) -> Self {
        Self { from: level, to: level, start_us: 0, duration_us: 0, curve: EasingCurve::Linear }
    }

    // a time before the start gives `from`
    pub fn level_at(&self, now_us: u64) -> f32 {
        if self.duration_us == 0 {
            return self.to;
        }
        let elapsed = now_us.saturating_sub(self.start_us);
        if elapsed >= self.duration_us {
            return self.to;
        }
        let t = (elapsed as f64 / self.duration_us as f64) as f32;
        self.from + (self.to - self.from) * self.curve.apply(t)
    }

    pub fn target(&self) -> f32 {
        self.to
    }
}
