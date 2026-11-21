use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};

use amm_client::link::LinkState;
use amm_core::structs::event::ModEvent;
use amml_api::Logger;
use zoom_core::config::ZoomConfig;
use zoom_core::services::ZoomController;
use zoom_core::structs::{FovQuery, ManagerReply, ZoomEvent};

use crate::services::{MonotonicClock, ZoomLink, ZoomLog};

// the zoom state for the whole game session, so no listener can outlive it.
// Lock order is link, then controller. The controller lock is never held across a call into the manager.
// Only `attach`, `detach` and `tick` (game thread) take the link lock; the FOV, turn and reset calls read the
// connected state the last tick stored, so a `CameraFov` on the render thread never reaches the manager link
pub struct ZoomSession {
    link: Mutex<Option<ZoomLink>>,
    controller: Mutex<ZoomController>,
    clock: MonotonicClock,
    log: ZoomLog,
    connected: AtomicBool,
    configured: AtomicBool,
}

impl ZoomSession {
    pub fn new(log: Logger) -> Self {
        Self {
            link: Mutex::new(None),
            controller: Mutex::new(ZoomController::new(ZoomConfig::default())),
            clock: MonotonicClock::new(),
            log: ZoomLog::new(log),
            connected: AtomicBool::new(false),
            configured: AtomicBool::new(false),
        }
    }

    // takes the link; a link from an earlier attach is dropped first, so the manager never sees the mod twice.
    // The settings are read on the first tick with the link connected
    pub fn attach(&self, link: ZoomLink) {
        let mut slot = self.link.lock().unwrap_or_else(PoisonError::into_inner);
        drop(slot.take());
        *slot = Some(link);
        self.connected.store(false, Ordering::Relaxed);
        self.configured.store(false, Ordering::Relaxed);
        self.log.attached();
        self.with_controller(|controller, _| controller.resume());
    }

    // the zoom returns to normal and stays off; dropping the link releases the lease and unregisters
    pub fn detach(&self) {
        let mut slot = self.link.lock().unwrap_or_else(PoisonError::into_inner);
        self.connected.store(false, Ordering::Relaxed);
        self.with_controller(|controller, _| controller.suspend());
        drop(slot.take());
    }

    // reads the settings once the link is connected, then the manager's events into the controller, and keeps the
    // Camera lease in line with the zoom; None when a panic poisoned a lock. While the link is not connected only
    // the events are read
    pub fn tick(&self, in_gameplay: bool) -> Option<()> {
        let mut slot = self.link.lock().ok()?;
        let Some(link) = slot.as_mut() else { return Some(()) };
        let connected = link.state() == LinkState::Connected;
        self.connected.store(connected, Ordering::Relaxed);
        if connected && !self.configured.swap(true, Ordering::Relaxed) {
            let config = link.settings();
            self.with_controller(|controller, _| controller.reconfigure(config))?;
            self.log.linked();
        }
        let events = link.drain();
        let reply = self.with_controller(|controller, now_us| {
            events.iter().fold(ManagerReply::Nothing, |reply, event| {
                match self.apply(controller, event, in_gameplay, now_us) {
                    ManagerReply::ReloadSettings => ManagerReply::ReloadSettings,
                    ManagerReply::Nothing => reply,
                }
            })
        })?;
        if !connected {
            return Some(());
        }
        if reply == ManagerReply::ReloadSettings {
            let config = link.settings();
            self.with_controller(|controller, _| controller.reconfigure(config))?;
        }
        let wants_camera = self.with_controller(|controller, now_us| controller.wants_camera(now_us))?;
        let sync = link.sync_camera(wants_camera);
        self.with_controller(|controller, _| controller.lease_synced(sync))?;
        Some(())
    }

    // the field of view the game should use; `normal_fov` while the last tick saw the link not connected or
    // when a panic poisoned a lock
    pub fn fov(&self, normal_fov: f32, steady_fov: f32, in_gameplay: bool) -> f32 {
        if !self.connected.load(Ordering::Relaxed) {
            return normal_fov;
        }
        let query = FovQuery { normal_fov, steady_fov, in_gameplay };
        self.with_controller(|controller, now_us| {
            let result = controller.fov(query, now_us);
            self.log.fov(controller.level(now_us), normal_fov, steady_fov, result);
            result
        })
        .unwrap_or(normal_fov)
    }

    // the factor to multiply the look delta by; 1.0 while the last tick saw the link not connected or when a
    // panic poisoned the lock
    pub fn turn_scale(&self) -> f32 {
        if !self.connected.load(Ordering::Relaxed) {
            return 1.0;
        }
        self.with_controller(|controller, now_us| {
            let scale = controller.turn_scale(now_us);
            self.log.look(controller.level(now_us), scale);
            scale
        })
        .unwrap_or(1.0)
    }

    pub fn event(&self, event: ZoomEvent) {
        if !self.connected.load(Ordering::Relaxed) {
            return;
        }
        self.with_controller(|controller, now_us| {
            controller.event(event, now_us);
            self.log.reset(event);
        });
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

    // one manager event into the controller; the end of the manager suspends the zoom, a press the controller
    // took is logged
    fn apply(&self, controller: &mut ZoomController, event: &ModEvent, in_gameplay: bool, now_us: u64) -> ManagerReply {
        if *event == ModEvent::ManagerGone {
            controller.suspend();
            self.log.manager_gone();
            return ManagerReply::Nothing;
        }
        let held = controller.held();
        let reply = controller.apply(event, in_gameplay, now_us);
        if !held && controller.held() {
            self.log.started();
        }
        reply
    }
}
