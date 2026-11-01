use std::path::Path;

use alaydriem_zoom::services::ZoomSession;
use amm_client::test_support::FakeTable;
use amm_core::structs::descriptor::Capability;
use amm_core::structs::event::{KeybindPhase, ModEvent};
use amm_core::structs::lease::AcquireOutcome;
use amm_core::structs::registry::ModIndex;
use zoom_core::structs::ZoomEvent;

use crate::support::session::{PAST_EASE, PRESSED, RELEASED, Session};

fn fov(session: &ZoomSession) -> f32 {
    session.fov(70.0, 70.0, true)
}

fn send(session: &ZoomSession, event: ModEvent) {
    FakeTable::push_event(event);
    session.tick(true).unwrap();
}

#[test]
fn pressed_keybind_zooms_after_tick() {
    let session = Session::attached();

    send(&session, PRESSED);
    std::thread::sleep(PAST_EASE);

    assert_eq!(fov(&session), 17.5);
    assert!(session.turn_scale() < 1.0);
}

#[test]
fn release_returns_to_normal() {
    let session = Session::attached();
    send(&session, PRESSED);
    std::thread::sleep(PAST_EASE);

    send(&session, RELEASED);
    std::thread::sleep(PAST_EASE);

    assert_eq!(fov(&session), 70.0);
    assert_eq!(session.turn_scale(), 1.0);
}

#[test]
fn press_outside_gameplay_does_not_zoom() {
    let session = Session::attached();

    FakeTable::push_event(PRESSED);
    session.tick(false).unwrap();
    std::thread::sleep(PAST_EASE);

    assert_eq!(fov(&session), 70.0);
    assert!(FakeTable::with(|state| state.acquired.is_empty()));
}

#[test]
fn focus_lost_snaps_back() {
    let session = Session::attached();
    send(&session, PRESSED);
    std::thread::sleep(PAST_EASE);
    assert_eq!(fov(&session), 17.5);

    session.event(ZoomEvent::FocusLost);

    assert_eq!(fov(&session), 70.0);
}

#[test]
fn focus_lost_after_release_stays_normal() {
    let session = Session::attached();
    send(&session, PRESSED);
    std::thread::sleep(PAST_EASE);
    send(&session, RELEASED);

    session.event(ZoomEvent::FocusLost);

    assert_eq!(fov(&session), 70.0);
    std::thread::sleep(PAST_EASE);
    assert_eq!(fov(&session), 70.0);
}

#[test]
fn settings_changed_reloads_and_resets() {
    let session = Session::attached();
    send(&session, PRESSED);
    std::thread::sleep(PAST_EASE);
    FakeTable::set_config("zoom {\n  zoom_factor = 2.0\n}\n");

    send(&session, ModEvent::SettingsChanged { paths: 1 });

    assert_eq!(fov(&session), 70.0);
    send(&session, PRESSED);
    std::thread::sleep(PAST_EASE);
    assert_eq!(fov(&session), 35.0);
}

#[test]
fn camera_is_acquired_on_press_and_released_after_ease() {
    let session = Session::attached();

    send(&session, PRESSED);
    assert_eq!(FakeTable::with(|state| state.acquired.clone()), [Capability::Camera]);
    std::thread::sleep(PAST_EASE);
    send(&session, RELEASED);
    assert!(FakeTable::with(|state| state.released.is_empty()));
    std::thread::sleep(PAST_EASE);
    session.tick(true).unwrap();

    assert_eq!(FakeTable::with(|state| state.released.clone()), [Capability::Camera]);
}

#[test]
fn denied_camera_does_not_zoom() {
    let session = Session::attached();
    FakeTable::set_acquire_outcome(Capability::Camera, AcquireOutcome::Denied { holder: ModIndex(1) });

    send(&session, PRESSED);
    std::thread::sleep(PAST_EASE);

    assert_eq!(fov(&session), 70.0);
}

#[test]
fn suspended_camera_snaps_back() {
    let session = Session::attached();
    send(&session, PRESSED);
    std::thread::sleep(PAST_EASE);

    send(&session, ModEvent::Suspended { capability: Capability::Camera, by: ModIndex(1) });

    assert_eq!(fov(&session), 70.0);
}

// the manager forgets a Shared holder that gives back a suspended lease, so no Resumed ever follows
#[test]
fn suspended_lease_given_back_zooms_again() {
    let session = Session::attached();
    send(&session, PRESSED);
    std::thread::sleep(PAST_EASE);

    send(&session, ModEvent::Suspended { capability: Capability::Camera, by: ModIndex(1) });
    assert_eq!(FakeTable::with(|state| state.released.clone()), [Capability::Camera]);
    send(&session, RELEASED);
    send(&session, PRESSED);
    std::thread::sleep(PAST_EASE);

    assert_eq!(fov(&session), 17.5);
    assert_eq!(FakeTable::with(|state| state.acquired.len()), 2);
}

#[test]
fn denied_then_granted_zooms() {
    let session = Session::attached();
    FakeTable::set_acquire_outcome(Capability::Camera, AcquireOutcome::Denied { holder: ModIndex(1) });
    send(&session, PRESSED);
    send(&session, RELEASED);
    FakeTable::set_acquire_outcome(Capability::Camera, AcquireOutcome::Granted);

    send(&session, PRESSED);
    std::thread::sleep(PAST_EASE);

    assert_eq!(fov(&session), 17.5);
}

#[test]
fn settings_change_does_not_log_hook_active_again() {
    let session = Session::attached();
    fov(&session);

    send(&session, ModEvent::SettingsChanged { paths: 1 });
    fov(&session);

    let lines = FakeTable::with(|state| state.logs.iter().filter(|line| line.message == "getFov hook active").count());
    assert_eq!(lines, 1);
}

#[test]
fn revoked_camera_is_not_released_again() {
    let session = Session::attached();
    send(&session, PRESSED);

    send(&session, ModEvent::Revoked { capability: Capability::Camera, by: ModIndex(1) });

    assert_eq!(fov(&session), 70.0);
    assert!(FakeTable::with(|state| state.released.is_empty()));
}

#[test]
fn detach_then_attach_zooms_again() {
    let session = Session::attached();
    send(&session, PRESSED);
    std::thread::sleep(PAST_EASE);

    session.detach();

    assert_eq!(fov(&session), 70.0);
    assert_eq!(FakeTable::with(|state| state.unregistered.len()), 1);
    session.attach(FakeTable::table(), Path::new(FakeTable::CONFIG_DIR)).unwrap();
    send(&session, PRESSED);
    std::thread::sleep(PAST_EASE);
    assert_eq!(fov(&session), 17.5);
}

#[test]
fn more_than_256_events_are_kept_for_the_next_tick() {
    let session = Session::attached();
    for _ in 0..300 {
        FakeTable::push_event(ModEvent::Keybind { action: 0, phase: KeybindPhase::Repeat });
    }
    FakeTable::push_event(PRESSED);

    session.tick(true).unwrap();
    assert!(FakeTable::with(|state| state.acquired.is_empty()));
    session.tick(true).unwrap();

    std::thread::sleep(PAST_EASE);
    assert_eq!(fov(&session), 17.5);
}

#[test]
fn tick_without_link_is_ok() {
    let session = ZoomSession::new();

    assert_eq!(session.tick(true), Some(()));
}

#[test]
fn register_has_no_frame() {
    let _session = Session::attached();

    assert!(FakeTable::with(|state| state.registered_without_frame()));
}
