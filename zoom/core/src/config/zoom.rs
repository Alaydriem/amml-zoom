use serde::Deserialize;

use crate::structs::EasingCurve;

// the `zoom` block of config.hcl; the manager checks every value against descriptor.hcl before the mod reads it
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct ZoomConfig {
    // the field of view is divided by this when fully zoomed
    pub zoom_factor: f32,
    pub zoom_in_ms: u32,
    pub zoom_out_ms: u32,
    pub easing: String,
    // 1.0 slows mouse look to exactly match the zoom, 0.0 leaves it unchanged
    pub sensitivity_match_strength: f32,
    // fades out the sprint and effect FOV changes while zoomed
    pub fade_variable_fov_while_zoomed: bool,
}

impl Default for ZoomConfig {
    fn default() -> Self {
        Self {
            zoom_factor: 4.0,
            zoom_in_ms: 250,
            zoom_out_ms: 250,
            easing: EasingCurve::DEFAULT_NAME.to_string(),
            sensitivity_match_strength: 1.0,
            fade_variable_fov_while_zoomed: true,
        }
    }
}

impl ZoomConfig {
    pub const MIN_ZOOM_FACTOR: f32 = 1.0;
    pub const MAX_ZOOM_FACTOR: f32 = 32.0;
    pub const MAX_DURATION_MS: u32 = 2000;

    // clamps every value into its working range, logging each field it had to change
    pub fn sanitized(self) -> Self {
        let defaults = Self::default();
        let zoom_factor = Self::clamp_float(
            "zoomFactor",
            self.zoom_factor,
            Self::MIN_ZOOM_FACTOR,
            Self::MAX_ZOOM_FACTOR,
            defaults.zoom_factor,
        );
        let sensitivity_match_strength = Self::clamp_float(
            "sensitivityMatchStrength",
            self.sensitivity_match_strength,
            0.0,
            1.0,
            defaults.sensitivity_match_strength,
        );
        let zoom_in_ms = Self::clamp_duration("zoomInMs", self.zoom_in_ms);
        let zoom_out_ms = Self::clamp_duration("zoomOutMs", self.zoom_out_ms);
        let easing = if EasingCurve::from_name(&self.easing).is_some() {
            self.easing
        } else {
            curia::warn!("config easing is unknown; using the default", { value: self.easing, used: defaults.easing.clone() });
            defaults.easing
        };
        Self { zoom_factor, sensitivity_match_strength, zoom_in_ms, zoom_out_ms, easing, ..self }
    }

    pub fn curve(&self) -> EasingCurve {
        EasingCurve::from_name(&self.easing).unwrap_or(EasingCurve::EaseInOutCubic)
    }

    fn clamp_float(name: &str, value: f32, min: f32, max: f32, fallback: f32) -> f32 {
        let clamped = if value.is_nan() { fallback } else { value.clamp(min, max) };
        if clamped != value {
            curia::warn!("config value is out of range; using the clamped value", { field: name, value: value, used: clamped });
        }
        clamped
    }

    fn clamp_duration(name: &str, value: u32) -> u32 {
        let clamped = value.min(Self::MAX_DURATION_MS);
        if clamped != value {
            curia::warn!("config value is out of range; using the clamped value", { field: name, value: value, used: clamped });
        }
        clamped
    }
}
