use zoom_core::config::ZoomConfig;
use zoom_core::structs::EasingCurve;

fn with_factor(zoom_factor: f32) -> ZoomConfig {
    ZoomConfig { zoom_factor, ..ZoomConfig::default() }
}

#[test]
fn sanitize_clamps_zoom_factor() {
    assert_eq!(with_factor(0.0).sanitized().zoom_factor, 1.0);
    assert_eq!(with_factor(-3.0).sanitized().zoom_factor, 1.0);
    assert_eq!(with_factor(1e9).sanitized().zoom_factor, 32.0);
    assert_eq!(with_factor(f32::NAN).sanitized().zoom_factor, 4.0);
}

#[test]
fn sanitize_clamps_durations() {
    let config = ZoomConfig { zoom_in_ms: 99_999, zoom_out_ms: 99_999, ..ZoomConfig::default() }.sanitized();
    assert_eq!(config.zoom_in_ms, 2000);
    assert_eq!(config.zoom_out_ms, 2000);
}

#[test]
fn sanitize_clamps_strength() {
    let strength = |value| ZoomConfig { sensitivity_match_strength: value, ..ZoomConfig::default() }.sanitized();
    assert_eq!(strength(-1.0).sensitivity_match_strength, 0.0);
    assert_eq!(strength(2.0).sensitivity_match_strength, 1.0);
    assert_eq!(strength(f32::NAN).sensitivity_match_strength, 1.0);
}

#[test]
fn sanitize_replaces_unknown_easing() {
    let config = ZoomConfig { easing: "bouncy".to_string(), ..ZoomConfig::default() }.sanitized();
    assert_eq!(config.easing, "easeInOutCubic");
    assert_eq!(config.curve(), EasingCurve::EaseInOutCubic);
}

#[test]
fn known_easing_name_selects_curve() {
    let config = ZoomConfig { easing: "easeOutExpo".to_string(), ..ZoomConfig::default() }.sanitized();
    assert_eq!(config.curve(), EasingCurve::EaseOutExpo);
}
