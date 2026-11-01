use zoom_core::structs::EasingCurve;

const CURVES: [EasingCurve; 6] = [
    EasingCurve::Linear,
    EasingCurve::EaseOutSine,
    EasingCurve::EaseOutCubic,
    EasingCurve::EaseOutQuint,
    EasingCurve::EaseOutExpo,
    EasingCurve::EaseInOutCubic,
];

#[test]
fn curve_endpoints() {
    for curve in CURVES {
        assert!(curve.apply(0.0).abs() < 1e-6, "{curve:?} at 0");
        assert!((curve.apply(1.0) - 1.0).abs() < 1e-6, "{curve:?} at 1");
        assert_eq!(curve.apply(-1.0), 0.0, "{curve:?} below 0");
        assert_eq!(curve.apply(2.0), 1.0, "{curve:?} above 1");
    }
}

#[test]
fn curve_is_monotonic() {
    for curve in CURVES {
        let mut previous = curve.apply(0.0);
        for step in 1..=1000 {
            let value = curve.apply(step as f32 / 1000.0);
            assert!(value >= previous, "{curve:?} decreases at step {step}");
            previous = value;
        }
    }
}
