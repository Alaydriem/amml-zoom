use alaydriem_zoom::services::ZoomRuntime;
use amm_client::test_support::FakeTable;
use amml_api::testing::FakeGame;
use zoom_core::structs::ZoomEvent;

use crate::support::session::{PAST_EASE, PRESSED, Session};

// a runtime on a link connected to the fake manager; `game` stays alive with it
fn attached() -> (FakeGame, ZoomRuntime) {
    let game = Session::game();
    let runtime = ZoomRuntime::new(game.ctx().log());
    runtime.attach(Session::link(&game));
    runtime.tick(true);
    (game, runtime)
}

// the zoom key goes down while the game has the mouse captured, and the ease runs to its end
fn zoomed() -> (FakeGame, ZoomRuntime) {
    let (game, runtime) = attached();
    FakeTable::push_event(PRESSED);
    runtime.tick(true);
    std::thread::sleep(PAST_EASE);
    (game, runtime)
}

#[test]
fn without_the_manager_the_fov_and_the_turn_are_unchanged() {
    let game = FakeTable::game();
    let runtime = ZoomRuntime::new(game.ctx().log());
    runtime.attach(Session::link(&game));

    runtime.tick(true);

    assert_eq!(runtime.fov(70.0, 70.0), 70.0);
    assert_eq!(runtime.turn_scale(), 1.0);
}

#[test]
fn press_with_the_mouse_captured_zooms_the_fov_and_the_turn() {
    let (_game, runtime) = zoomed();

    assert_eq!(runtime.fov(70.0, 70.0), 17.5);
    assert!(runtime.turn_scale() < 1.0);
}

#[test]
fn press_with_a_menu_open_does_not_zoom() {
    let (_game, runtime) = attached();

    FakeTable::push_event(PRESSED);
    runtime.tick(false);
    std::thread::sleep(PAST_EASE);

    assert_eq!(runtime.fov(70.0, 70.0), 70.0);
    assert_eq!(runtime.turn_scale(), 1.0);
}

#[test]
fn a_menu_opened_on_a_tick_ends_the_zoom() {
    let (_game, runtime) = zoomed();

    runtime.tick(false);
    runtime.fov(70.0, 70.0);
    std::thread::sleep(PAST_EASE);

    assert_eq!(runtime.fov(70.0, 70.0), 70.0);
}

#[test]
fn leaving_the_world_snaps_back() {
    let (_game, runtime) = zoomed();

    runtime.event(ZoomEvent::WorldLeft);

    assert_eq!(runtime.fov(70.0, 70.0), 70.0);
}

#[test]
fn detach_unregisters_and_the_fov_is_unchanged() {
    let (_game, runtime) = zoomed();

    runtime.detach();

    assert_eq!(FakeTable::with(|state| state.unregistered.len()), 1);
    assert_eq!(runtime.fov(70.0, 70.0), 70.0);
    assert_eq!(runtime.turn_scale(), 1.0);
}

#[test]
fn a_detached_runtime_zooms_again_with_a_new_link() {
    let (game, runtime) = zoomed();
    runtime.detach();

    runtime.attach(Session::link(&game));
    runtime.tick(true);
    FakeTable::push_event(PRESSED);
    runtime.tick(true);
    std::thread::sleep(PAST_EASE);

    assert_eq!(runtime.fov(70.0, 70.0), 17.5);
}
