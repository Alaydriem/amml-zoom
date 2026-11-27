use std::sync::atomic::{AtomicBool, Ordering};

use amml_api::Logger;
use zoom_core::structs::ZoomEvent;

// the log lines of a zoom, each once per zoom: the full level, the end, the first scaled look turn and the first
// look turn at the full level. Numbers are
// written with two decimals
pub struct ZoomLog {
    log: Logger,
    linked: AtomicBool,
    full: AtomicBool,
    zooming: AtomicBool,
    look: AtomicBool,
    look_full: AtomicBool,
}

impl ZoomLog {
    pub fn new(log: Logger) -> Self {
        Self {
            log,
            linked: AtomicBool::new(false),
            full: AtomicBool::new(false),
            zooming: AtomicBool::new(false),
            look: AtomicBool::new(false),
            look_full: AtomicBool::new(false),
        }
    }

    // a new link: the lines of the link and of the zoom start over
    pub fn attached(&self) {
        for flag in [&self.linked, &self.full, &self.zooming, &self.look, &self.look_full] {
            flag.store(false, Ordering::Relaxed);
        }
    }

    pub fn linked(&self) {
        if !self.linked.swap(true, Ordering::Relaxed) {
            self.log.info("zoom linked", &[]);
        }
    }

    pub fn started(&self) {
        self.log.info("zoom started", &[]);
    }

    // one `fov` answer at the zoom `level`
    pub fn fov(&self, level: f32, base: f32, steady: f32, result: f32) {
        if level >= 1.0 && !self.full.swap(true, Ordering::Relaxed) {
            self.log.info(
                "zoom full",
                &[("base", &Self::number(base)), ("steady", &Self::number(steady)), ("result", &Self::number(result))],
            );
        }
        if level > 0.0 {
            self.zooming.store(true, Ordering::Relaxed);
        } else if self.zooming.swap(false, Ordering::Relaxed) {
            self.full.store(false, Ordering::Relaxed);
            self.look.store(false, Ordering::Relaxed);
            self.look_full.store(false, Ordering::Relaxed);
            self.log.info("zoom ended", &[]);
        }
    }

    // one look turn at the zoom `level`
    pub fn look(&self, level: f32, scale: f32) {
        if scale < 1.0 && !self.look.swap(true, Ordering::Relaxed) {
            self.log.info("zoom look scaled", &[("scale", &Self::number(scale))]);
        }
        if level >= 1.0 && !self.look_full.swap(true, Ordering::Relaxed) {
            self.log.info("zoom look full", &[("scale", &Self::number(scale))]);
        }
    }

    // the manager is gone: the zoom stays off until the mod is enabled again
    pub fn manager_gone(&self) {
        self.log.info("zoom off", &[("reason", "manager_gone")]);
    }

    pub fn reset(&self, event: ZoomEvent) {
        let reason = match event {
            ZoomEvent::FocusLost => "focus_lost",
            ZoomEvent::WorldLeft => "world_left",
            ZoomEvent::DimensionChanged => "dimension",
        };
        self.log.info("zoom reset", &[("reason", reason)]);
    }

    fn number(value: f32) -> String {
        format!("{value:.2}")
    }
}
