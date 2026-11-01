use alaydriem_zoom::services::GameClient;

use crate::support::fake_client::FakeClient;

#[test]
fn no_client_is_not_gameplay() {
    let client = GameClient::new();
    client.set_mouse_grabbed_slot(FakeClient::SLOT);

    assert!(!client.in_gameplay());
}

#[test]
fn no_slot_is_not_gameplay() {
    let fake = FakeClient::new(true);
    let client = GameClient::new();
    client.capture(fake.address());

    assert!(!client.in_gameplay());
}

#[test]
fn reads_the_mouse_grab_through_the_vtable_slot() {
    let grabbed = FakeClient::new(true);
    let released = FakeClient::new(false);
    let first = GameClient::new();
    let second = GameClient::new();
    first.set_mouse_grabbed_slot(FakeClient::SLOT);
    second.set_mouse_grabbed_slot(FakeClient::SLOT);

    first.capture(grabbed.address());
    second.capture(released.address());

    assert!(first.in_gameplay());
    assert!(!second.in_gameplay());
}

#[test]
fn keeps_the_first_client() {
    let primary = FakeClient::new(true);
    let split_screen = FakeClient::new(false);
    let client = GameClient::new();
    client.set_mouse_grabbed_slot(FakeClient::SLOT);

    client.capture(0);
    client.capture(primary.address());
    client.capture(split_screen.address());

    assert_eq!(client.client(), primary.address());
    assert!(client.in_gameplay());
}
