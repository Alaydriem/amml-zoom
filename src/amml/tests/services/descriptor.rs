use alaydriem_zoom::services::ZoomDescriptor;
use amm_core::services::DescriptorChecker;
use amm_core::services::settings::SchemaChecker;
use amm_core::structs::descriptor::{Capability, Chord, ClaimMode, Descriptor, KeyScope};
use amm_core::structs::settings::HclDocument;
use zoom_core::config::{ZoomConfig, ZoomSettings};

fn descriptor() -> Descriptor {
    DescriptorChecker::new(128, 256).parse(ZoomDescriptor::TEXT).unwrap()
}

#[test]
fn descriptor_passes_the_manager_checker() {
    let descriptor = descriptor();

    assert_eq!(descriptor.namespace, "alaydriem");
    assert_eq!(descriptor.name, "zoom");
    assert_eq!(descriptor.id(), "@alaydriem/zoom");
    let claims: Vec<_> = descriptor.claims.iter().map(|claim| (claim.capability, claim.mode)).collect();
    assert_eq!(claims, [(Capability::Camera, ClaimMode::Shared)]);
    let keybinds: Vec<_> = descriptor
        .keybinds
        .iter()
        .map(|keybind| (keybind.action.as_str(), keybind.capability, keybind.scope, keybind.default_chord))
        .collect();
    let grave = Chord { key: 0xC0, ctrl: false, shift: false, alt: false, f3: false };
    assert_eq!(keybinds, [("zoom", Capability::Camera, KeyScope::Global, grave)]);
}

#[test]
fn schema_defaults_match_core_defaults() {
    let checked = SchemaChecker::check(&descriptor().settings, HclDocument::empty());

    let settings = hcl::from_str::<ZoomSettings>(&checked.document.text()).unwrap();

    assert_eq!(settings.into_config(), ZoomConfig::default());
}
