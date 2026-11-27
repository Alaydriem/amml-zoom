use std::panic::AssertUnwindSafe;

use alaydriem_zoom::services::ZoomSession;
use amm_client::test_support::FakeTable;
use amm_core::structs::descriptor::Capability;
use amm_core::structs::event::{KeybindPhase, ModEvent};
use amm_core::structs::lease::AcquireOutcome;
use amm_core::structs::registry::ModIndex;
use amml_api::testing::FakeGame;
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
fn the_settings_are_read_on_the_first_connected_tick() {
    let game = Session::game();
    FakeTable::set_config("zoom {\n  zoom_factor = 2.0\n}\n");
    let session = ZoomSession::new(game.ctx().log());
    session.attach(Session::link(&game));

    session.tick(true).unwrap();
    send(&session, PRESSED);
    std::thread::sleep(PAST_EASE);

    assert_eq!(fov(&session), 35.0);
}

#[test]
fn the_link_is_logged_once() {
    let session = Session::attached();

    session.tick(true).unwrap();
    session.tick(true).unwrap();

    let linked = session.game.logs().iter().filter(|line| *line == "info zoom linked").count();
    assert_eq!(linked, 1);
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
fn revoked_camera_is_not_released_again() {
    let session = Session::attached();
    send(&session, PRESSED);

    send(&session, ModEvent::Revoked { capability: Capability::Camera, by: ModIndex(1) });

    assert_eq!(fov(&session), 70.0);
    assert!(FakeTable::with(|state| state.released.is_empty()));
}

#[test]
fn the_end_of_the_manager_gives_the_game_values_back() {
    let session = Session::attached();
    send(&session, PRESSED);
    std::thread::sleep(PAST_EASE);
    assert_eq!(fov(&session), 17.5);

    FakeTable::gone(&session.game);
    session.tick(true).unwrap();

    assert_eq!(fov(&session), 70.0);
    assert_eq!(session.turn_scale(), 1.0);
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
    session.attach(Session::link(&session.game));
    session.tick(true).unwrap();
    send(&session, PRESSED);
    std::thread::sleep(PAST_EASE);
    assert_eq!(fov(&session), 17.5);
}

#[test]
fn a_press_behind_a_flood_of_events_is_read_in_the_same_tick() {
    let session = Session::attached();
    for _ in 0..300 {
        FakeTable::push_event(ModEvent::Keybind { action: 0, phase: KeybindPhase::Repeat });
    }
    FakeTable::push_event(PRESSED);

    session.tick(true).unwrap();

    assert_eq!(FakeTable::with(|state| state.acquired.clone()), [Capability::Camera]);
    std::thread::sleep(PAST_EASE);
    assert_eq!(fov(&session), 17.5);
}

#[test]
fn tick_without_link_is_ok() {
    let game = FakeGame::new();
    let session = ZoomSession::new(game.ctx().log());

    assert_eq!(session.tick(true), Some(()));
    assert_eq!(fov(&session), 70.0);
}

#[test]
fn a_poisoned_controller_gives_the_game_values_back() {
    let session = Session::attached();
    send(&session, PRESSED);
    std::thread::sleep(PAST_EASE);
    let _ = std::panic::catch_unwind(AssertUnwindSafe(|| session.with_controller(|_, _| panic!("poisoned"))));

    assert_eq!(fov(&session), 70.0);
    assert_eq!(session.turn_scale(), 1.0);
    assert_eq!(session.tick(true), None);
}
