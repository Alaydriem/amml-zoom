use std::path::Path;
use std::time::Duration;

use alaydriem_zoom::services::ZoomSession;
use amm_client::test_support::FakeTable;
use amm_core::structs::event::{KeybindPhase, ModEvent};

pub const PAST_EASE: Duration = Duration::from_millis(300);
pub const PRESSED: ModEvent = ModEvent::Keybind { action: 0, phase: KeybindPhase::Pressed };
pub const RELEASED: ModEvent = ModEvent::Keybind { action: 0, phase: KeybindPhase::Released };

// a session attached to the fake manager of this thread
pub struct Session;

impl Session {
    pub fn attached() -> ZoomSession {
        let session = ZoomSession::new();
        session.attach(FakeTable::table(), Path::new(FakeTable::CONFIG_DIR)).unwrap();
        session
    }
}
