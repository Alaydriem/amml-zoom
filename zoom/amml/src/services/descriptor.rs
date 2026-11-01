// the descriptor the manager registers the mod from; it names the claim, the zoom keybind and the settings schema
pub struct ZoomDescriptor;

impl ZoomDescriptor {
    pub const TEXT: &'static str = include_str!("../../descriptor.hcl");
}
