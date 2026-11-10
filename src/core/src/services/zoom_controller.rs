use amm_core::structs::descriptor::Capability;
use amm_core::structs::event::{KeybindPhase, ModEvent};

use crate::config::ZoomConfig;
use crate::services::ZoomMath;
use crate::structs::{EasingCurve, FovQuery, LeaseSync, ManagerReply, ZoomAction, ZoomEvent, ZoomTransition};

// the zoom state: whether the key is held and the ease of the zoom level between 0 (normal) and 1 (zoomed);
// while suspended the manager has taken the camera or disabled the mod, so the zoom stays normal and a press
// is ignored until the lease is given back, Resumed or Enabled
pub struct ZoomController {
    config: ZoomConfig,
    curve: EasingCurve,
    zoom_in_us: u64,
    zoom_out_us: u64,
    held: bool,
    suspended: bool,
    transition: ZoomTransition,
    last_base_fov: f32,
    fov_seen: bool,
    turn_seen: bool,
}

impl ZoomController {
    pub const DEFAULT_BASE_FOV: f32 = 70.0;
    const MAX_FOV: f32 = 180.0;

    pub fn new(config: ZoomConfig) -> Self {
        Self {
            curve: config.curve(),
            zoom_in_us: u64::from(config.zoom_in_ms) * 1000,
            zoom_out_us: u64::from(config.zoom_out_ms) * 1000,
            config,
            held: false,
            suspended: false,
            transition: ZoomTransition::settled(0.0),
            last_base_fov: Self::DEFAULT_BASE_FOV,
            fov_seen: false,
            turn_seen: false,
        }
    }

    pub fn press(&mut self, in_gameplay: bool, now_us: u64) {
        if !self.held && in_gameplay && !self.suspended {
            self.held = true;
            self.ease_to(1.0, self.zoom_in_us, now_us);
        }
    }

    pub fn release(&mut self, now_us: u64) {
        if self.held {
            self.held = false;
            self.ease_to(0.0, self.zoom_out_us, now_us);
        }
    }

    pub fn suspend(&mut self) {
        self.suspended = true;
        self.snap_to_normal();
    }

    pub fn resume(&mut self) {
        self.suspended = false;
    }

    // a settings change: the new values apply from the next press
    pub fn reconfigure(&mut self, config: ZoomConfig) {
        *self = Self {
            suspended: self.suspended,
            last_base_fov: self.last_base_fov,
            fov_seen: self.fov_seen,
            turn_seen: self.turn_seen,
            ..Self::new(config)
        };
    }

    // a released lease ends a suspension: the manager forgets a Shared holder that gives its lease back and
    // sends no Resumed, so the next press asks for the lease again. A refused lease ends the zoom
    pub fn lease_synced(&mut self, sync: LeaseSync) {
        match sync {
            LeaseSync::Released => self.suspended = false,
            LeaseSync::Refused => self.snap_to_normal(),
            LeaseSync::Unchanged | LeaseSync::Acquired => {}
        }
    }

    // true while the zoom changes the view, so the mod must hold the Camera lease
    pub fn wants_camera(&self, now_us: u64) -> bool {
        self.held || self.level(now_us) > 0.0
    }

    pub fn apply(&mut self, event: &ModEvent, in_gameplay: bool, now_us: u64) -> ManagerReply {
        match *event {
            ModEvent::Keybind { action, phase } => match (ZoomAction::from_index(action), phase) {
                (Some(ZoomAction::Zoom), KeybindPhase::Pressed) => self.press(in_gameplay, now_us),
                (Some(ZoomAction::Zoom), KeybindPhase::Released) => self.release(now_us),
                _ => {}
            },
            ModEvent::Suspended { capability: Capability::Camera, .. } | ModEvent::Disabled => self.suspend(),
            ModEvent::Revoked { capability: Capability::Camera, .. } => self.snap_to_normal(),
            ModEvent::Resumed(Capability::Camera)
            | ModEvent::CapabilityAvailable(Capability::Camera)
            | ModEvent::Enabled => self.resume(),
            ModEvent::SettingsChanged { .. } => return ManagerReply::ReloadSettings,
            _ => {}
        }
        ManagerReply::Nothing
    }

    pub fn event(&mut self, event: ZoomEvent, now_us: u64) {
        self.held = false;
        match event {
            ZoomEvent::FocusLost | ZoomEvent::WorldLeft => self.snap_to_normal(),
            ZoomEvent::DimensionChanged => self.ease_to(0.0, self.zoom_out_us, now_us),
        }
    }

    pub fn level(&self, now_us: u64) -> f32 {
        self.transition.level_at(now_us)
    }

    // an invalid base FOV from the game is returned unchanged and changes nothing
    pub fn fov(&mut self, query: FovQuery, now_us: u64) -> f32 {
        if !self.fov_seen {
            self.fov_seen = true;
            curia::info!("getFov hook active");
        }
        if !Self::is_valid_fov(query.normal_fov) {
            return query.normal_fov;
        }
        if !query.in_gameplay && self.held {
            self.release(now_us);
        }
        let steady_fov = if Self::is_valid_fov(query.steady_fov) { query.steady_fov } else { query.normal_fov };
        let level = self.level(now_us);
        let base =
            ZoomMath::blended_base(query.normal_fov, steady_fov, level, self.config.fade_variable_fov_while_zoomed);
        self.last_base_fov = base;
        ZoomMath::zoomed_fov(base, self.config.zoom_factor, level)
    }

    pub fn turn_scale(&mut self, now_us: u64) -> f32 {
        if !self.turn_seen {
            self.turn_seen = true;
            curia::info!("turn delta hook active");
        }
        let level = self.level(now_us);
        if level <= 0.0 {
            return 1.0;
        }
        let zoomed = ZoomMath::zoomed_fov(self.last_base_fov, self.config.zoom_factor, level);
        ZoomMath::turn_scale(self.last_base_fov, zoomed, self.config.sensitivity_match_strength)
    }

    fn snap_to_normal(&mut self) {
        self.held = false;
        self.transition = ZoomTransition::settled(0.0);
    }

    // starts from the current level, so a change of direction never jumps
    fn ease_to(&mut self, target: f32, full_duration_us: u64, now_us: u64) {
        let from = self.level(now_us);
        self.transition = ZoomTransition::new(from, target, now_us, full_duration_us, self.curve);
    }

    fn is_valid_fov(fov: f32) -> bool {
        fov.is_finite() && fov > 0.0 && fov < Self::MAX_FOV
    }
}
