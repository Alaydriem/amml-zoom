// a keybind the mod declares; the index is the keybind's position in descriptor.hcl
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomAction {
    Zoom,
}

impl ZoomAction {
    pub fn from_index(action: u16) -> Option<Self> {
        match action {
            0 => Some(Self::Zoom),
            _ => None,
        }
    }
}
