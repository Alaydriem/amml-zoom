use zoom_core::structs::{EasingCurve, ZoomTransition};

#[test]
fn settled_level_is_constant() {
    let transition = ZoomTransition::settled(0.0);
    assert_eq!(transition.level_at(0), 0.0);
    assert_eq!(transition.level_at(9_000_000), 0.0);
}

#[test]
fn transition_reaches_target_at_end() {
    let transition = ZoomTransition::new(0.0, 1.0, 1000, 250_000, EasingCurve::EaseOutCubic);
    assert_eq!(transition.level_at(1000), 0.0);
    assert_eq!(transition.level_at(251_000), 1.0);
    assert_eq!(transition.target(), 1.0);
}

#[test]
fn level_before_start_is_from() {
    let transition = ZoomTransition::new(0.0, 1.0, 1000, 250_000, EasingCurve::EaseOutCubic);
    assert_eq!(transition.level_at(500), 0.0);
}

#[test]
fn reversal_duration_scales_with_distance() {
    let transition = ZoomTransition::new(0.5, 0.0, 0, 250_000, EasingCurve::Linear);
    assert_eq!(transition.level_at(125_000), 0.0);
    assert!((transition.level_at(62_500) - 0.25).abs() < 1e-6);
}

#[test]
fn zero_duration_snaps() {
    let transition = ZoomTransition::new(0.2, 1.0, 0, 0, EasingCurve::EaseOutCubic);
    assert_eq!(transition.level_at(0), 1.0);
}

#[test]
fn same_time_gives_same_level() {
    let transition = ZoomTransition::new(0.0, 1.0, 0, 250_000, EasingCurve::EaseOutCubic);
    let once = transition.level_at(100_000);
    let _ = transition.level_at(100_000);
    let twice = transition.level_at(100_000);
    let mut fifty = 0.0;
    for step in 0..50 {
        let _ = transition.level_at(step * 2000);
        fifty = transition.level_at(100_000);
    }
    assert_eq!(once.to_bits(), twice.to_bits());
    assert_eq!(once.to_bits(), fifty.to_bits());
}
