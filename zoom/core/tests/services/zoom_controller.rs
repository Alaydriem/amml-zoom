use amm_core::structs::descriptor::Capability;
use amm_core::structs::event::{KeybindPhase, ModEvent};
use amm_core::structs::registry::ModIndex;
use zoom_core::config::ZoomConfig;
use zoom_core::services::ZoomMath;
use zoom_core::structs::{FovQuery, LeaseSync, ManagerReply, ZoomEvent};

use crate::support::controllers::{FULL, controller, zoomed_at};

fn query(normal_fov: f32, in_gameplay: bool) -> FovQuery {
    FovQuery { normal_fov, steady_fov: normal_fov, in_gameplay }
}

#[test]
fn press_eases_in() {
    let mut zoom = controller();
    zoom.press(true, 0);
    assert_eq!(zoom.level(0), 0.0);
    let mid = zoom.level(100_000);
    assert!(mid > 0.0 && mid < 1.0);
    assert_eq!(zoom.level(FULL), 1.0);
}

#[test]
fn release_eases_out() {
    let mut zoom = zoomed_at(300_000);
    zoom.release(300_000);
    assert!(zoom.level(400_000) < 1.0);
    assert_eq!(zoom.level(550_000), 0.0);
}

#[test]
fn reverse_mid_ease_has_no_jump() {
    let mut zoom = controller();
    zoom.press(true, 0);
    let before = zoom.level(100_000);
    zoom.release(100_000);
    assert_eq!(zoom.level(100_000).to_bits(), before.to_bits());
}

#[test]
fn key_repeat_does_not_restart() {
    let mut zoom = controller();
    zoom.press(true, 0);
    let before = zoom.level(200_000);
    zoom.press(true, 200_000);
    assert_eq!(zoom.level(200_000).to_bits(), before.to_bits());
    assert_eq!(zoom.level(FULL), 1.0);
}

#[test]
fn release_while_out_of_gameplay_clears_hold() {
    let mut zoom = zoomed_at(300_000);
    zoom.release(300_000);
    assert_eq!(zoom.level(600_000), 0.0);
    zoom.press(true, 700_000);
    assert_eq!(zoom.level(700_000 + FULL), 1.0);
}

#[test]
fn focus_lost_snaps() {
    let mut zoom = zoomed_at(300_000);
    zoom.event(ZoomEvent::FocusLost, 300_000);
    assert_eq!(zoom.level(300_000), 0.0);
}

#[test]
fn world_left_snaps() {
    let mut zoom = zoomed_at(300_000);
    zoom.event(ZoomEvent::WorldLeft, 300_000);
    assert_eq!(zoom.level(300_000), 0.0);
}

#[test]
fn dimension_change_eases_out() {
    let mut zoom = zoomed_at(300_000);
    zoom.event(ZoomEvent::DimensionChanged, 300_000);
    let mid = zoom.level(350_000);
    assert!(mid > 0.0 && mid < 1.0);
    assert_eq!(zoom.level(550_000), 0.0);
}

#[test]
fn fov_zooms_with_level() {
    let mut zoom = controller();
    zoom.press(true, 0);
    assert_eq!(zoom.fov(query(70.0, true), FULL), 17.5);
}

#[test]
fn fov_passes_through_invalid_base() {
    let mut zoom = controller();
    zoom.press(true, 0);
    for value in [0.0, -5.0, 200.0] {
        assert_eq!(zoom.fov(query(value, true), FULL), value);
    }
    assert!(zoom.fov(query(f32::NAN, true), FULL).is_nan());
    assert_eq!(zoom.level(FULL), 1.0);
}

#[test]
fn fov_leaving_gameplay_eases_out() {
    let mut zoom = zoomed_at(300_000);
    let _ = zoom.fov(query(70.0, false), 300_000);
    assert_eq!(zoom.level(550_000), 0.0);
}

#[test]
fn fov_fades_sprint_while_zoomed() {
    let mut zoom = controller();
    zoom.press(true, 0);
    let sprinting = FovQuery { normal_fov: 80.0, steady_fov: 70.0, in_gameplay: true };
    assert_eq!(zoom.fov(sprinting, 0), 80.0);
    assert_eq!(zoom.fov(sprinting, FULL), 17.5);
}

#[test]
fn turn_scale_before_first_fov_uses_default_base() {
    let mut zoom = controller();
    zoom.press(true, 0);
    let scale = zoom.turn_scale(FULL);
    assert!(scale.is_finite());
    assert!((scale - ZoomMath::turn_scale(70.0, 17.5, 1.0)).abs() < 1e-6);
}

#[test]
fn turn_scale_is_one_when_not_zoomed() {
    let mut zoom = controller();
    assert_eq!(zoom.turn_scale(0), 1.0);
}

#[test]
fn turn_scale_eases_with_zoom() {
    let mut zoom = controller();
    zoom.press(true, 0);
    let mid = zoom.turn_scale(100_000);
    let full = zoom.turn_scale(FULL);
    assert!(mid < 1.0 && mid > full);
}

#[test]
fn default_ease_starts_gently() {
    let mut zoom = controller();
    zoom.press(true, 0);
    let first_frame_at_60_fps = zoom.level(16_667);
    assert!(first_frame_at_60_fps < 0.01, "level after one frame is {first_frame_at_60_fps}");
}

const PRESSED: ModEvent = ModEvent::Keybind { action: 0, phase: KeybindPhase::Pressed };
const RELEASED: ModEvent = ModEvent::Keybind { action: 0, phase: KeybindPhase::Released };
const SUSPENDED: ModEvent = ModEvent::Suspended { capability: Capability::Camera, by: ModIndex(1) };

#[test]
fn pressed_keybind_zooms() {
    let mut zoom = controller();
    assert_eq!(zoom.apply(&PRESSED, true, 0), ManagerReply::Nothing);
    assert_eq!(zoom.level(FULL), 1.0);
}

#[test]
fn released_keybind_eases_out() {
    let mut zoom = controller();
    zoom.apply(&PRESSED, true, 0);
    zoom.apply(&RELEASED, true, FULL);
    assert_eq!(zoom.level(2 * FULL), 0.0);
}

#[test]
fn press_outside_gameplay_is_ignored() {
    let mut zoom = controller();
    zoom.apply(&PRESSED, false, 0);
    assert_eq!(zoom.level(FULL), 0.0);
    assert!(!zoom.wants_camera(FULL));
}

#[test]
fn unknown_action_is_ignored() {
    let mut zoom = controller();
    let other = ModEvent::Keybind { action: 1, phase: KeybindPhase::Pressed };
    assert_eq!(zoom.apply(&other, true, 0), ManagerReply::Nothing);
    assert_eq!(zoom.level(FULL), 0.0);
}

#[test]
fn repeat_does_not_restart_ease() {
    let mut zoom = controller();
    zoom.apply(&PRESSED, true, 0);
    let before = zoom.level(FULL / 2);
    zoom.apply(&ModEvent::Keybind { action: 0, phase: KeybindPhase::Repeat }, true, FULL / 2);
    assert_eq!(zoom.level(FULL / 2).to_bits(), before.to_bits());
}

#[test]
fn suspend_snaps_and_blocks_press() {
    let mut zoom = zoomed_at(FULL);
    zoom.apply(&SUSPENDED, true, FULL);
    assert_eq!(zoom.level(FULL), 0.0);
    zoom.apply(&PRESSED, true, FULL);
    assert_eq!(zoom.level(3 * FULL), 0.0);
    zoom.apply(&ModEvent::Resumed(Capability::Camera), true, 3 * FULL);
    zoom.apply(&PRESSED, true, 3 * FULL);
    assert_eq!(zoom.level(4 * FULL), 1.0);
}

#[test]
fn disable_suspends() {
    let mut zoom = zoomed_at(FULL);
    zoom.apply(&ModEvent::Disabled, true, FULL);
    assert_eq!(zoom.level(FULL), 0.0);
    zoom.apply(&PRESSED, true, FULL);
    assert_eq!(zoom.level(2 * FULL), 0.0);
}

#[test]
fn revoke_snaps_without_blocking_the_next_press() {
    let mut zoom = zoomed_at(FULL);
    zoom.apply(&ModEvent::Revoked { capability: Capability::Camera, by: ModIndex(1) }, true, FULL);
    assert_eq!(zoom.level(FULL), 0.0);
    zoom.apply(&PRESSED, true, FULL);
    assert_eq!(zoom.level(2 * FULL), 1.0);
}

#[test]
fn released_lease_clears_suspension() {
    let mut zoom = zoomed_at(FULL);
    zoom.apply(&SUSPENDED, true, FULL);

    zoom.lease_synced(LeaseSync::Released);

    zoom.press(true, FULL);
    assert_eq!(zoom.level(2 * FULL), 1.0);
}

#[test]
fn refused_lease_snaps_without_blocking_the_next_press() {
    let mut zoom = zoomed_at(FULL);

    zoom.lease_synced(LeaseSync::Refused);

    assert_eq!(zoom.level(FULL), 0.0);
    zoom.press(true, FULL);
    assert_eq!(zoom.level(2 * FULL), 1.0);
}

#[test]
fn capability_available_and_enabled_resume() {
    for event in [ModEvent::CapabilityAvailable(Capability::Camera), ModEvent::Enabled] {
        let mut zoom = controller();
        zoom.suspend();
        zoom.apply(&event, true, 0);
        zoom.apply(&PRESSED, true, 0);
        assert_eq!(zoom.level(FULL), 1.0);
    }
}

#[test]
fn suspend_of_other_capability_is_ignored() {
    let mut zoom = zoomed_at(FULL);
    zoom.apply(&ModEvent::Suspended { capability: Capability::Hud, by: ModIndex(1) }, true, FULL);
    assert_eq!(zoom.level(FULL), 1.0);
}

#[test]
fn settings_changed_asks_for_reload() {
    let mut zoom = controller();
    assert_eq!(zoom.apply(&ModEvent::SettingsChanged { paths: 1 }, true, 0), ManagerReply::ReloadSettings);
}

#[test]
fn reconfigure_resets_and_keeps_suspension() {
    let mut zoom = controller();
    zoom.suspend();
    zoom.reconfigure(ZoomConfig::default());
    zoom.press(true, 0);
    assert_eq!(zoom.level(FULL), 0.0);

    let mut zoom = zoomed_at(FULL);
    zoom.reconfigure(ZoomConfig { zoom_factor: 2.0, ..ZoomConfig::default() });
    assert_eq!(zoom.level(FULL), 0.0);
    zoom.press(true, FULL);
    assert_eq!(zoom.fov(query(70.0, true), 2 * FULL), 35.0);
}

#[test]
fn wants_camera_until_eased_out() {
    let mut zoom = controller();
    zoom.press(true, 0);
    zoom.release(FULL);
    assert!(zoom.wants_camera(FULL + 1));
    assert!(!zoom.wants_camera(2 * FULL + 1));
}
