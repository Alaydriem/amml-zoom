use std::path::Path;

use amm_client::link::{ApiTable, EventDrain, ManagerLink};
use amm_core::errors::LinkError;
use amm_core::structs::descriptor::Capability;
use amm_core::structs::event::ModEvent;
use amm_core::structs::lease::AcquireOutcome;
use zoom_core::config::{ZoomConfig, ZoomSettings};
use zoom_core::structs::LeaseSync;

use crate::services::ZoomDescriptor;

// the registration with the mod manager, the events read this tick and whether the Camera lease is held;
// `events` holds at most MAX_EVENTS_PER_TICK and is cleared at the start of every drain
pub struct ZoomLink {
    link: ManagerLink,
    events: Vec<ModEvent>,
    holding_camera: bool,
}

impl ZoomLink {
    pub const MAX_DRAINS_PER_TICK: usize = 4;
    pub const MAX_EVENTS_PER_TICK: usize = Self::MAX_DRAINS_PER_TICK * EventDrain::CAPACITY;

    pub fn new(table: ApiTable, config_dir: &Path) -> Result<Self, LinkError> {
        let link = ManagerLink::without_frame(table, ZoomDescriptor::TEXT, config_dir)?;
        Ok(Self { link, events: Vec::with_capacity(Self::MAX_EVENTS_PER_TICK), holding_camera: false })
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

    // reads the queued manager events, at most MAX_EVENTS_PER_TICK; the rest wait for the next tick
    pub fn drain(&mut self) -> &[ModEvent] {
        self.events.clear();
        for _ in 0..Self::MAX_DRAINS_PER_TICK {
            let before = self.events.len();
            self.events.extend(self.link.events());
            if self.events.len() - before < EventDrain::CAPACITY {
                break;
            }
        }
        for event in &self.events {
            if matches!(event, ModEvent::Revoked { capability: Capability::Camera, .. } | ModEvent::Disabled) {
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
