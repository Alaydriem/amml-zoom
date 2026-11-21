use alaydriem_zoom::ZoomMod;
use amm_client::link::LinkUses;
use amm_client::test_support::FakeTable;
use amm_core::structs::descriptor::Capability;
use amm_core::structs::lease::AcquireOutcome;
use amm_core::structs::registry::ModIndex;
use amml_api::amml_abi::{CameraFov, ClientTick};
use amml_api::{AmmlMod, Area};

use crate::support::zoom_rig::ZoomRig;

const MANIFEST: &str = include_str!("../amml.hcl");

#[test]
fn a_held_key_divides_the_fov_by_the_factor() {
    let rig = ZoomRig::with_manager(true);

    rig.press();

    assert_eq!(rig.fov(70.0, 70.0), 17.5);
    assert!(rig.logged("info zoom linked"));
    assert!(rig.logged("info zoom started"));
    assert!(rig.logged("info zoom full base=70.00 steady=70.00 result=17.50"));
}

#[test]
fn a_release_gives_the_game_fov_back() {
    let rig = ZoomRig::with_manager(true);
    rig.press();
    rig.fov(70.0, 70.0);

    rig.release();

    assert_eq!(rig.fov(70.0, 70.0), 70.0);
    assert!(rig.logged("info zoom ended"));
}

#[test]
fn no_zoom_starts_while_the_mouse_is_free() {
    let rig = ZoomRig::with_manager(false);

    rig.press();

    assert_eq!(rig.fov(70.0, 70.0), 70.0);
    assert!(!rig.logged("info zoom started"));
}

#[test]
fn a_screen_opened_while_held_ends_the_zoom() {
    let rig = ZoomRig::with_manager(true);
    rig.press();
    assert_eq!(rig.fov(70.0, 70.0), 17.5);

    rig.capture(Some(false));
    rig.tick();

    assert_eq!(rig.fov(70.0, 70.0), 70.0);
}

#[test]
fn the_look_delta_is_scaled_while_zoomed_and_injected_turns_are_not() {
    let rig = ZoomRig::with_manager(true);
    rig.press();
    rig.fov(70.0, 70.0);

    let (yaw, pitch) = rig.look(8.0, 4.0, false);
    let injected = rig.look(8.0, 4.0, true);

    assert!(yaw > 0.0 && yaw < 8.0 && pitch > 0.0 && pitch < 4.0);
    assert_eq!(yaw / 8.0, pitch / 4.0);
    assert_eq!(injected, (8.0, 4.0));
    assert!(rig.logged(&format!("info zoom look scaled scale={:.2}", yaw / 8.0)));
    assert!(rig.logged(&format!("info zoom look full scale={:.2}", yaw / 8.0)));
}

#[test]
fn a_focus_loss_resets_the_zoom() {
    let rig = ZoomRig::with_manager(true);
    let resets = [
        ("focus_lost", ZoomRig::focus_lost as fn(&ZoomRig)),
        ("world_left", ZoomRig::world_left),
        ("dimension", ZoomRig::dimension_changing),
    ];

    for (reason, reset) in resets {
        rig.press();
        assert_eq!(rig.fov(70.0, 70.0), 17.5, "{reason}");

        reset(&rig);

        assert_eq!(rig.fov(70.0, 70.0), 70.0, "{reason}");
        assert!(rig.logged(&format!("info zoom reset reason={reason}")), "{reason}");
        rig.release();
    }
}

#[test]
fn manager_gone_while_zoomed_gives_the_game_fov() {
    let rig = ZoomRig::with_manager(true);
    rig.press();
    assert_eq!(rig.fov(70.0, 70.0), 17.5);

    FakeTable::gone(&rig.game);
    rig.tick();

    assert_eq!(rig.fov(70.0, 70.0), 70.0);
    assert!(rig.logged("info zoom off reason=manager_gone"));
    rig.tick();
    assert_eq!(rig.fov(70.0, 70.0), 70.0);
    rig.game.disable::<ZoomMod>();
}

#[test]
fn the_fov_follows_the_link_state_the_last_tick_stored() {
    let rig = ZoomRig::with_manager(true);
    rig.press();
    assert_eq!(rig.fov(70.0, 70.0), 17.5);

    FakeTable::gone(&rig.game);
    let before_the_tick = rig.fov(70.0, 70.0);
    rig.tick();

    assert_eq!(before_the_tick, 17.5);
    assert_eq!(rig.fov(70.0, 70.0), 70.0);
    rig.game.disable::<ZoomMod>();
}

#[test]
fn a_refused_required_listen_is_logged_and_fails_enable() {
    let game = FakeTable::game();
    game.offer::<ClientTick>();
    game.offer::<CameraFov>();
    game.offer_area(Area::CLIENT_MOUSE);
    game.refuse_listen::<CameraFov>();
    FakeTable::provide(&game.ctx()).unwrap();
    game.load::<ZoomMod>().unwrap();

    assert!(game.enable::<ZoomMod>().is_err());
    let logs = game.logs();
    assert!(logs.iter().any(|line| line.starts_with("error zoom off error=")), "{logs:?}");
}

#[test]
fn without_the_manager_the_fov_is_unchanged() {
    let rig = ZoomRig::without_manager();

    for _ in 0..10 {
        rig.press();
        rig.tick();
    }

    assert_eq!(rig.fov(70.0, 70.0), 70.0);
    assert_eq!(rig.look(8.0, 4.0, false), (8.0, 4.0));
}

#[test]
fn disable_while_zoomed_leaves_the_game_fov_and_releases_the_lease() {
    let rig = ZoomRig::with_manager(true);
    rig.press();
    assert_eq!(rig.fov(70.0, 70.0), 17.5);

    rig.game.disable::<ZoomMod>();

    assert_eq!(rig.fov(70.0, 70.0), 70.0);
    let calls = FakeTable::calls();
    let release = calls.iter().position(|call| *call == "release").unwrap();
    let unregister = calls.iter().position(|call| *call == "unregister").unwrap();
    assert!(release < unregister, "{calls:?}");
}

#[test]
fn a_denied_camera_lease_leaves_the_game_fov() {
    FakeTable::set_acquire_outcome(Capability::Camera, AcquireOutcome::Denied { holder: ModIndex(1) });
    let rig = ZoomRig::with_manager(true);

    rig.press();

    assert_eq!(rig.fov(70.0, 70.0), 70.0);
}

#[test]
fn the_manifest_names_the_mod_and_lists_no_entries() {
    assert!(MANIFEST.contains("namespace    = \"alaydriem\""));
    assert!(MANIFEST.contains("name         = \"zoom\""));
    assert!(MANIFEST.contains("api_version  = 3"));
    assert!(!MANIFEST.contains("entries"));
}

#[test]
fn link_uses_are_complete() {
    assert_eq!(LinkUses::check(&ZoomMod::USES), Ok(()));
}

#[test]
fn the_manifest_uses_block_is_the_uses_of_the_mod() {
    let manifest = MANIFEST.replace("\r\n", "\n");

    assert_eq!(manifest.matches(&ZoomMod::USES.to_hcl()).count(), 1);
    assert_eq!(ZoomMod::USES.write_into(&manifest), manifest);
}

#[test]
fn the_manifest_writer_keeps_a_crlf_manifest_and_adds_no_second_block() {
    let crlf = MANIFEST.replace("\r\n", "\n").replace('\n', "\r\n");

    let once = ZoomMod::USES.write_into(&crlf);
    let twice = ZoomMod::USES.write_into(&once);

    assert_eq!(once, crlf);
    assert_eq!(twice, crlf);
    assert_eq!(twice.matches("\nuses {\r\n").count(), 1);
}
