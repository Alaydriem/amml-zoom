use std::path::Path;
use std::sync::{Mutex, PoisonError};

use amm_client::link::ApiTable;
use amm_core::errors::LinkError;
use zoom_core::config::ZoomConfig;
use zoom_core::services::ZoomController;
use zoom_core::structs::{FovQuery, ManagerReply, ZoomEvent};

use crate::services::{MonotonicClock, ZoomLink};

// the zoom state for the whole game session, so no hook can outlive it.
// Lock order is link, then controller. The controller lock is never held across a call into the manager,
// so the FOV and turn queries never wait on it
pub struct ZoomSession {
    link: Mutex<Option<ZoomLink>>,
    controller: Mutex<ZoomController>,
    clock: MonotonicClock,
}

impl ZoomSession {
    pub fn new() -> Self {
        Self {
            link: Mutex::new(None),
            controller: Mutex::new(ZoomController::new(ZoomConfig::default())),
            clock: MonotonicClock::new(),
        }
    }

    // registers with the manager; a link from an earlier attach is dropped first, so the manager never sees
    // the mod twice
    pub fn attach(&self, table: ApiTable, config_dir: &Path) -> Result<(), LinkError> {
        let mut slot = self.link.lock().unwrap_or_else(PoisonError::into_inner);
        drop(slot.take());
        let link = ZoomLink::new(table, config_dir)?;
        let config = link.settings();
        *slot = Some(link);
        self.with_controller(|controller, _| {
            controller.reconfigure(config);
            controller.resume();
        });
        curia::info!("@alaydriem/zoom attached to the mod manager");
        Ok(())
    }

    // the zoom returns to normal and stays off; dropping the link releases the lease and unregisters
    pub fn detach(&self) {
        let mut slot = self.link.lock().unwrap_or_else(PoisonError::into_inner);
        self.with_controller(|controller, _| controller.suspend());
        drop(slot.take());
    }

    // reads the manager's events into the controller and keeps the Camera lease in line with the zoom;
    // None when a panic poisoned a lock
    pub fn tick(&self, in_gameplay: bool) -> Option<()> {
        let mut slot = self.link.lock().ok()?;
        let Some(link) = slot.as_mut() else { return Some(()) };
        let events = link.drain();
        let reply = self.with_controller(|controller, now_us| {
            events.iter().fold(ManagerReply::Nothing, |reply, event| {
                match controller.apply(event, in_gameplay, now_us) {
                    ManagerReply::ReloadSettings => ManagerReply::ReloadSettings,
                    ManagerReply::Nothing => reply,
                }
            })
        })?;
        if reply == ManagerReply::ReloadSettings {
            let config = link.settings();
            self.with_controller(|controller, _| controller.reconfigure(config))?;
        }
        let wants_camera = self.with_controller(|controller, now_us| controller.wants_camera(now_us))?;
        let sync = link.sync_camera(wants_camera);
        self.with_controller(|controller, _| controller.lease_synced(sync))?;
        Some(())
    }

    // the field of view the game should use; `normal_fov` when a panic poisoned the lock
    pub fn fov(&self, normal_fov: f32, steady_fov: f32, in_gameplay: bool) -> f32 {
        let query = FovQuery { normal_fov, steady_fov, in_gameplay };
        self.with_controller(|controller, now_us| controller.fov(query, now_us)).unwrap_or(normal_fov)
    }

    // the factor to multiply the look delta by; 1.0 when a panic poisoned the lock
    pub fn turn_scale(&self) -> f32 {
        self.with_controller(|controller, now_us| controller.turn_scale(now_us)).unwrap_or(1.0)
    }

    pub fn event(&self, event: ZoomEvent) {
        self.with_controller(|controller, now_us| controller.event(event, now_us));
    }

    // the time is read inside the lock, so calls reach the controller in time order;
    // None when a panic poisoned the lock
    pub fn with_controller<R>(&self, body: impl FnOnce(&mut ZoomController, u64) -> R) -> Option<R> {
        match self.controller.lock() {
            Ok(mut controller) => {
                let now_us = self.clock.now_us();
                Some(body(&mut controller, now_us))
            }
            Err(_) => {
                curia::error!("zoom controller lock is poisoned");
                None
            }
        }
    }
}

impl Default for ZoomSession {
    fn default() -> Self {
        Self::new()
    }
}
