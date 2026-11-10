use std::path::Path;

use alaydriem_zoom::ZoomMod;
use alaydriem_zoom::services::ZoomRuntime;
use alaydriem_zoom::types::LinkState;
use amm_client::test_support::FakeTable;
use amml_sdk::{AmmlMod, SdkError};

use crate::support::fake_host::FakeHost;

const REQUIRED: [(&str, u64); 3] =
    [(ZoomMod::UPDATE, 0x1000), (ZoomMod::GET_FOV, 0x2000), (ZoomMod::MOUSE_GRABBED, 147)];

fn runtime() -> ZoomRuntime {
    ZoomRuntime::new(Path::new(FakeTable::CONFIG_DIR))
}

#[test]
fn every_entry_installs_every_hook() {
    let mut entries = REQUIRED.to_vec();
    entries.extend([
        (ZoomMod::APPLY_TURN_DELTA, 0x3000),
        (ZoomMod::FOCUS_LOST, 0x4000),
        (ZoomMod::DIMENSION_CHANGE, 0x5000),
        (ZoomMod::LEVEL_EXIT, 0x6000),
    ]);
    let host = FakeHost::new(&entries);
    let runtime = runtime();

    ZoomMod::enable_with(&host, &runtime).unwrap();

    assert_eq!(host.hooked().len(), 6);
    assert!(host.warnings().is_empty());
}

#[test]
fn missing_optional_entries_turn_off_only_their_features() {
    let mut entries = REQUIRED.to_vec();
    entries.push((ZoomMod::FOCUS_LOST, 0x4000));
    let host = FakeHost::new(&entries);
    let runtime = runtime();
    runtime.detach();

    ZoomMod::enable_with(&host, &runtime).unwrap();

    assert_eq!(host.hooked(), [ZoomMod::GET_FOV, ZoomMod::FOCUS_LOST, ZoomMod::UPDATE]);
    let warnings = host.warnings();
    assert_eq!(warnings.len(), 3);
    assert!(warnings[0].contains(ZoomMod::APPLY_TURN_DELTA) && warnings[0].contains("look speed matching"));
    assert_eq!(runtime.link_state(), LinkState::Waiting);
}

#[test]
fn a_missing_required_entry_installs_nothing() {
    let mut entries = REQUIRED.to_vec();
    entries.retain(|(id, _)| *id != ZoomMod::GET_FOV);
    entries.push((ZoomMod::APPLY_TURN_DELTA, 0x3000));
    let host = FakeHost::new(&entries);

    let result = ZoomMod::enable_with(&host, &runtime());

    assert!(matches!(result, Err(SdkError::MissingEntry(id)) if id == ZoomMod::GET_FOV));
    assert!(host.hooked().is_empty());
}

#[test]
fn a_missing_mouse_grab_slot_installs_nothing() {
    let entries: Vec<_> = REQUIRED.iter().copied().filter(|(id, _)| *id != ZoomMod::MOUSE_GRABBED).collect();
    let host = FakeHost::new(&entries);

    assert!(ZoomMod::enable_with(&host, &runtime()).is_err());
    assert!(host.hooked().is_empty());
}

#[test]
fn hooks_past_the_capacity_fail_enable_and_are_removed() {
    let host = FakeHost::new(&REQUIRED);
    let runtime = runtime();
    for id in 0..ZoomRuntime::HOOK_CAPACITY as u64 - 1 {
        assert!(runtime.keep_hook(amml_sdk::HookHandle(100 + id)));
    }

    let result = ZoomMod::enable_with(&host, &runtime);

    assert!(result.is_err());
    assert_eq!(host.hooked().len(), 2);
    assert_eq!(host.unhooked().len(), ZoomRuntime::HOOK_CAPACITY + 1);
    assert!(runtime.take_hooks().is_empty());
}

#[test]
fn the_mod_id_is_the_namespaced_module_string() {
    assert_eq!(ZoomMod::NAMESPACE, "alaydriem");
    assert_eq!(ZoomMod::NAME, "zoom");
    assert_eq!(ZoomMod::id(), ZoomMod::MODULE);
    assert_eq!(ZoomMod::MODULE, "@alaydriem/zoom");
}
