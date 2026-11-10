use zoom_core::config::{ZoomConfig, ZoomSettings};
use zoom_core::structs::EasingCurve;

fn read(text: &str) -> ZoomConfig {
    hcl::from_str::<ZoomSettings>(text).unwrap().into_config()
}

#[test]
fn empty_document_gives_defaults() {
    assert_eq!(read(""), ZoomConfig::default());
}

#[test]
fn document_values_are_read() {
    let config = read("zoom {\n  zoom_factor = 8.0\n  easing = \"linear\"\n}\nkeys {\n  zoom = \"Ctrl+Z\"\n}\n");

    assert_eq!(config.zoom_factor, 8.0);
    assert_eq!(config.curve(), EasingCurve::Linear);
    assert_eq!(config.zoom_in_ms, ZoomConfig::default().zoom_in_ms);
    assert_eq!(config.sensitivity_match_strength, ZoomConfig::default().sensitivity_match_strength);
}

#[test]
fn out_of_range_value_is_clamped() {
    assert_eq!(read("zoom {\n  zoom_factor = 99.0\n}\n").zoom_factor, 32.0);
}
