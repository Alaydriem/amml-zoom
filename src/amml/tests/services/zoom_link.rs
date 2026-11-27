use alaydriem_zoom::services::ZoomLink;
use amm_client::link::LinkState;
use amm_client::test_support::FakeTable;
use amm_core::structs::descriptor::Capability;
use amm_core::structs::lease::AcquireOutcome;
use amm_core::structs::registry::ModIndex;
use amml_api::testing::FakeGame;
use zoom_core::config::ZoomConfig;
use zoom_core::structs::LeaseSync;

use crate::support::session::Session;

// a link connected to the fake manager; `game` stays alive with it
fn link() -> (FakeGame, ZoomLink) {
    let game = Session::game();
    let link = Session::link(&game);
    (game, link)
}

#[test]
fn the_link_is_connected_when_the_manager_is_provided() {
    let (_game, link) = link();

    assert_eq!(link.state(), LinkState::Connected);
}

#[test]
fn config_error_falls_back_to_defaults() {
    let (_game, link) = link();
    FakeTable::set_config("zoom {");

    assert_eq!(link.settings(), ZoomConfig::default());
}

#[test]
fn sync_camera_does_not_acquire_twice() {
    let (_game, mut link) = link();

    assert_eq!(link.sync_camera(true), LeaseSync::Acquired);
    assert_eq!(link.sync_camera(true), LeaseSync::Unchanged);

    assert_eq!(FakeTable::with(|state| state.acquired.clone()), [Capability::Camera]);
}

#[test]
fn sync_camera_releases_once() {
    let (_game, mut link) = link();
    link.sync_camera(true);

    assert_eq!(link.sync_camera(false), LeaseSync::Released);
    assert_eq!(link.sync_camera(false), LeaseSync::Unchanged);

    assert_eq!(FakeTable::with(|state| state.released.clone()), [Capability::Camera]);
}

#[test]
fn denied_camera_is_refused() {
    let (_game, mut link) = link();
    FakeTable::set_acquire_outcome(Capability::Camera, AcquireOutcome::Denied { holder: ModIndex(1) });

    assert_eq!(link.sync_camera(true), LeaseSync::Refused);
    assert_eq!(link.sync_camera(false), LeaseSync::Unchanged);
}

#[test]
fn dropping_the_link_gives_the_camera_lease_back_before_it_closes() {
    let (_game, mut link) = link();
    link.sync_camera(true);

    drop(link);

    assert_eq!(FakeTable::with(|state| state.released.clone()), [Capability::Camera]);
    let calls = FakeTable::calls();
    let release = calls.iter().position(|call| *call == "release").unwrap();
    let unregister = calls.iter().position(|call| *call == "unregister").unwrap();
    assert!(release < unregister, "{calls:?}");
}

#[test]
fn dropping_a_link_that_holds_nothing_releases_nothing() {
    let (_game, link) = link();

    drop(link);

    assert!(FakeTable::with(|state| state.released.is_empty()));
}
