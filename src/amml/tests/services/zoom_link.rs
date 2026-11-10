use std::path::Path;

use alaydriem_zoom::services::ZoomLink;
use amm_client::test_support::FakeTable;
use amm_core::structs::descriptor::Capability;
use amm_core::structs::lease::AcquireOutcome;
use amm_core::structs::registry::ModIndex;
use zoom_core::config::ZoomConfig;
use zoom_core::structs::LeaseSync;

fn link() -> ZoomLink {
    ZoomLink::new(FakeTable::table(), Path::new(FakeTable::CONFIG_DIR)).unwrap()
}

#[test]
fn config_error_falls_back_to_defaults() {
    let link = link();
    FakeTable::set_config("zoom {");

    assert_eq!(link.settings(), ZoomConfig::default());
}

#[test]
fn sync_camera_does_not_acquire_twice() {
    let mut link = link();

    assert_eq!(link.sync_camera(true), LeaseSync::Acquired);
    assert_eq!(link.sync_camera(true), LeaseSync::Unchanged);

    assert_eq!(FakeTable::with(|state| state.acquired.clone()), [Capability::Camera]);
}

#[test]
fn sync_camera_releases_once() {
    let mut link = link();
    link.sync_camera(true);

    assert_eq!(link.sync_camera(false), LeaseSync::Released);
    assert_eq!(link.sync_camera(false), LeaseSync::Unchanged);

    assert_eq!(FakeTable::with(|state| state.released.clone()), [Capability::Camera]);
}

#[test]
fn denied_camera_is_refused() {
    let mut link = link();
    FakeTable::set_acquire_outcome(Capability::Camera, AcquireOutcome::Denied { holder: ModIndex(1) });

    assert_eq!(link.sync_camera(true), LeaseSync::Refused);
    assert_eq!(link.sync_camera(false), LeaseSync::Unchanged);
}
