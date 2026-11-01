use std::ffi::c_void;
use std::sync::atomic::{AtomicU64, Ordering};

use alaydriem_zoom::detours::client_update;

static ORIGINAL_EXTRA: AtomicU64 = AtomicU64::new(0);

extern "C" fn game_update(_this: usize, init_finished: u8, extra: u64) -> u8 {
    ORIGINAL_EXTRA.store(extra, Ordering::SeqCst);
    init_finished
}

#[test]
fn the_client_update_detour_passes_the_third_argument_to_the_original_unchanged() {
    client_update::GUARD.original().store(game_update as *mut c_void, Ordering::SeqCst);

    assert_eq!(client_update::detour(0x1000, 1, 0xDEAD_BEEF_0000_0001), 1);

    assert_eq!(ORIGINAL_EXTRA.load(Ordering::SeqCst), 0xDEAD_BEEF_0000_0001);
}
