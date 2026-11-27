use amm_client::link::{LinkState, ManagerLink};
use amm_core::errors::LinkError;
use amm_core::structs::descriptor::Capability;
use amm_core::structs::event::ModEvent;
use amm_core::structs::lease::AcquireOutcome;
use amml_api::Ctx;
use zoom_core::config::{ZoomConfig, ZoomSettings};
use zoom_core::structs::LeaseSync;

use crate::services::ZoomDescriptor;

// the registration with the mod manager, the events read this tick and whether the Camera lease is held;
// `events` is cleared at the start of every drain; the link itself keeps a bounded number of unread events.
// Dropping the link gives the Camera lease back, then closes the link
pub struct ZoomLink {
    link: ManagerLink,
    events: Vec<ModEvent>,
    holding_camera: bool,
}

impl ZoomLink {
    // the link waits for the manager's tables; `state` says when it is connected
    pub fn start(ctx: &Ctx) -> Result<Self, LinkError> {
        let link = ManagerLink::start_without_frame(ctx, ZoomDescriptor::TEXT)?;
        Ok(Self { link, events: Vec::new(), holding_camera: false })
    }

    pub fn state(&self) -> LinkState {
        self.link.state()
    }

    pub fn settings(&self) -> ZoomConfig {
        match self.link.config::<ZoomSettings>() {
            Ok(settings) => settings.into_config(),
            Err(error) => {
                curia::warn!("config could not be read; using the defaults", { error: error.to_string() });
                ZoomConfig::default()
            }
        }
    }

    // reads every queued manager event
    pub fn drain(&mut self) -> &[ModEvent] {
        self.events.clear();
        self.events.extend(self.link.events());
        for event in &self.events {
            if matches!(
                event,
                ModEvent::Revoked { capability: Capability::Camera, .. } | ModEvent::Disabled | ModEvent::ManagerGone
            ) {
                self.holding_camera = false;
            }
        }
        &self.events
    }

    // takes or gives back the Camera lease to match `wants`
    pub fn sync_camera(&mut self, wants: bool) -> LeaseSync {
        if wants == self.holding_camera {
            return LeaseSync::Unchanged;
        }
        if !wants {
            self.link.release(Capability::Camera);
            self.holding_camera = false;
            return LeaseSync::Released;
        }
        match self.link.acquire(Capability::Camera) {
            Ok(AcquireOutcome::Granted | AcquireOutcome::Preempted) => {
                self.holding_camera = true;
                LeaseSync::Acquired
            }
            Ok(AcquireOutcome::Denied { .. }) => LeaseSync::Refused,
            Err(error) => {
                curia::warn!("camera lease request failed", { error: error.to_string() });
                LeaseSync::Refused
            }
        }
    }
}

impl Drop for ZoomLink {
    fn drop(&mut self) {
        if self.holding_camera {
            self.link.release(Capability::Camera);
        }
    }
}
