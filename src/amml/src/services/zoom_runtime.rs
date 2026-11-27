use std::sync::atomic::{AtomicBool, Ordering};

use amml_api::Logger;
use zoom_core::structs::ZoomEvent;

use crate::services::{ZoomLink, ZoomSession};

// what the listeners share: the zoom session and whether the game has the mouse captured (read on each client
// tick, so a field of view call sees the state of the last tick)
pub struct ZoomRuntime {
    session: ZoomSession,
    in_gameplay: AtomicBool,
}

impl ZoomRuntime {
    pub fn new(log: Logger) -> Self {
        Self { session: ZoomSession::new(log), in_gameplay: AtomicBool::new(false) }
    }

    // enable: the zoom works with this link until `detach`
    pub fn attach(&self, link: ZoomLink) {
        self.session.attach(link);
    }

    // disable: the zoom returns to normal, the Camera lease is given back and the link closes
    pub fn detach(&self) {
        self.session.detach();
        self.in_gameplay.store(false, Ordering::Relaxed);
    }

    // once per client tick on the game thread
    pub fn tick(&self, in_gameplay: bool) {
        self.in_gameplay.store(in_gameplay, Ordering::Relaxed);
        let _ = self.session.tick(in_gameplay);
    }

    // the field of view for one camera call; `base` unchanged while the manager is not connected
    pub fn fov(&self, base: f32, steady: f32) -> f32 {
        self.session.fov(base, steady, self.in_gameplay.load(Ordering::Relaxed))
    }

    // the look delta factor; 1.0 while the manager is not connected
    pub fn turn_scale(&self) -> f32 {
        self.session.turn_scale()
    }

    pub fn event(&self, event: ZoomEvent) {
        self.session.event(event);
    }
}
