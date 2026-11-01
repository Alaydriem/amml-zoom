use std::f32::consts::FRAC_PI_2;

// maps linear progress in [0, 1] to eased progress in [0, 1]; every curve starts at 0 and ends at 1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EasingCurve {
    Linear,
    EaseOutSine,
    EaseOutCubic,
    EaseOutQuint,
    EaseOutExpo,
    EaseInOutCubic,
}

impl EasingCurve {
    pub const DEFAULT_NAME: &'static str = "easeInOutCubic";

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "linear" => Some(Self::Linear),
            "easeOutSine" => Some(Self::EaseOutSine),
            "easeOutCubic" => Some(Self::EaseOutCubic),
            "easeOutQuint" => Some(Self::EaseOutQuint),
            "easeOutExpo" => Some(Self::EaseOutExpo),
            "easeInOutCubic" => Some(Self::EaseInOutCubic),
            _ => None,
        }
    }

    // t outside [0, 1] is clamped; NaN counts as the start
    pub fn apply(self, t: f32) -> f32 {
        if t.is_nan() || t <= 0.0 {
            return 0.0;
        }
        if t >= 1.0 {
            return 1.0;
        }
        match self {
            Self::Linear => t,
            Self::EaseOutSine => (t * FRAC_PI_2).sin(),
            Self::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
            Self::EaseOutQuint => 1.0 - (1.0 - t).powi(5),
            Self::EaseOutExpo => 1.0 - 2.0_f32.powf(-10.0 * t),
            Self::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
        }
    }
}
