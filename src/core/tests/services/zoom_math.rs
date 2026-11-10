use zoom_core::services::ZoomMath;

#[test]
fn zoomed_fov_endpoints() {
    assert_eq!(ZoomMath::zoomed_fov(70.0, 4.0, 0.0), 70.0);
    assert_eq!(ZoomMath::zoomed_fov(70.0, 4.0, 1.0), 17.5);
    assert!((ZoomMath::zoomed_fov(70.0, 4.0, 0.5) - 35.0).abs() < 1e-4);
}

#[test]
fn blend_fades_sprint_out() {
    assert_eq!(ZoomMath::blended_base(80.0, 70.0, 1.0, true), 70.0);
    assert_eq!(ZoomMath::blended_base(80.0, 70.0, 0.0, true), 80.0);
    assert_eq!(ZoomMath::blended_base(80.0, 70.0, 1.0, false), 80.0);
}

#[test]
fn turn_scale_matches_tangent_ratio() {
    let exact = 8.75_f32.to_radians().tan() / 35.0_f32.to_radians().tan();
    assert!((ZoomMath::turn_scale(70.0, 17.5, 1.0) - exact).abs() < 1e-6);
    assert!((exact - 0.2197).abs() < 1e-3);
    assert_eq!(ZoomMath::turn_scale(70.0, 17.5, 0.0), 1.0);
    let half = ZoomMath::turn_scale(70.0, 17.5, 0.5);
    assert!((half - (1.0 + exact) / 2.0).abs() < 1e-6);
}
