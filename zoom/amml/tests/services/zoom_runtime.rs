use std::cell::Cell;
use std::path::Path;

use alaydriem_zoom::services::ZoomRuntime;
use alaydriem_zoom::types::{LinkState, Vec2};
use amm_client::test_support::FakeTable;
use amml_sdk::HookHandle;
use zoom_core::structs::ZoomEvent;

use crate::support::fake_client::FakeClient;
use crate::support::session::{PAST_EASE, PRESSED};

const DELTA: Vec2 = Vec2 { x: 8.0, y: -4.0 };

fn attached() -> ZoomRuntime {
    let runtime = ZoomRuntime::new(Path::new(FakeTable::CONFIG_DIR));
    assert_eq!(runtime.attach_with(Ok(FakeTable::table())), LinkState::Attached);
    runtime
}

// the zoom key goes down while the game's client has `mouse_grabbed`, and the ease runs to its end; the client
// is boxed so its address stays the one the runtime keeps
fn zoomed(mouse_grabbed: bool) -> (ZoomRuntime, Box<FakeClient>) {
    let runtime = attached();
    let client = Box::new(FakeClient::new(mouse_grabbed));
    runtime.client().set_mouse_grabbed_slot(FakeClient::SLOT);
    FakeTable::push_event(PRESSED);
    runtime.tick(client.address());
    std::thread::sleep(PAST_EASE);
    (runtime, client)
}

#[test]
fn first_tick_without_the_manager_turns_the_zoom_off() {
    let runtime = ZoomRuntime::new(Path::new(FakeTable::CONFIG_DIR));
    let client = FakeClient::new(true);

    runtime.tick(client.address());

    assert_eq!(runtime.link_state(), LinkState::Off);
    assert_eq!(runtime.client().client(), client.address());
    let calls = Cell::new(0);
    let fov = runtime.fov(true, |_| {
        calls.set(calls.get() + 1);
        70.0
    });
    assert_eq!((fov, calls.get()), (70.0, 1));
    assert_eq!(runtime.turn(DELTA), DELTA);
}

#[test]
fn variable_fov_asks_the_game_for_the_steady_value_too() {
    let runtime = attached();
    let asked = Cell::new(Vec::new());
    let original = |variable: bool| {
        let mut seen = asked.take();
        seen.push(variable);
        asked.set(seen);
        if variable { 80.0 } else { 70.0 }
    };

    assert_eq!(runtime.fov(true, original), 80.0);
    assert_eq!(asked.take(), [false, true]);
    assert_eq!(runtime.fov(false, original), 70.0);
    assert_eq!(asked.take(), [false]);
}

#[test]
fn press_with_the_mouse_grabbed_zooms_the_fov_and_the_turn() {
    let (runtime, _client) = zoomed(true);

    assert_eq!(runtime.fov(false, |_| 70.0), 17.5);
    let turned = runtime.turn(DELTA);
    assert!(turned.x.abs() < DELTA.x.abs() && turned.y.abs() < DELTA.y.abs());
    assert_eq!(turned.x / DELTA.x, turned.y / DELTA.y);
}

#[test]
fn press_with_a_menu_open_does_not_zoom() {
    let (runtime, _client) = zoomed(false);

    assert_eq!(runtime.fov(false, |_| 70.0), 70.0);
    assert_eq!(runtime.turn(DELTA), DELTA);
}

#[test]
fn leaving_the_world_snaps_back() {
    let (runtime, _client) = zoomed(true);

    runtime.event(ZoomEvent::WorldLeft);

    assert_eq!(runtime.fov(false, |_| 70.0), 70.0);
}

#[test]
fn detach_unregisters_and_disables_the_link() {
    let (runtime, _client) = zoomed(true);

    runtime.detach();

    assert_eq!(runtime.link_state(), LinkState::Disabled);
    assert_eq!(FakeTable::with(|state| state.unregistered.len()), 1);
    assert_eq!(runtime.fov(false, |_| 70.0), 70.0);
}

#[test]
fn a_disabled_runtime_stays_disabled_until_enable() {
    let (runtime, client) = zoomed(true);
    runtime.detach();

    runtime.tick(client.address());
    assert_eq!(runtime.attach_with(Ok(FakeTable::table())), LinkState::Disabled);

    assert_eq!(runtime.link_state(), LinkState::Disabled);
    assert_eq!(FakeTable::with(|state| state.unregistered.len()), 2);
    runtime.enable_link();
    assert_eq!(runtime.attach_with(Ok(FakeTable::table())), LinkState::Attached);
}

#[test]
fn hooks_are_bounded_and_taken_newest_first() {
    let runtime = ZoomRuntime::new(Path::new(FakeTable::CONFIG_DIR));
    let count = ZoomRuntime::HOOK_CAPACITY as u64 + 2;

    let kept: Vec<bool> = (0..count).map(|id| runtime.keep_hook(HookHandle(id))).collect();
    let taken = runtime.take_hooks();

    let expected: Vec<HookHandle> = (0..ZoomRuntime::HOOK_CAPACITY as u64).rev().map(HookHandle).collect();
    assert_eq!(taken, expected);
    assert_eq!(kept.iter().filter(|kept| !**kept).count(), 2);
    assert!(runtime.take_hooks().is_empty());
}
