use serde::Deserialize;

use crate::config::ZoomConfig;

// the whole config.hcl document; the `keys` block belongs to the manager and is not read here
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ZoomSettings {
    pub zoom: ZoomConfig,
}

impl ZoomSettings {
    pub fn into_config(self) -> ZoomConfig {
        self.zoom.sanitized()
    }
}
