//! @alaydriem/zoom as an AMML mod: listens to the loader's client tick, camera field of view, look turn, focus,
//! level exit and dimension change events, and drives zoom-core with them.

#![deny(unsafe_code)]

#[forbid(unsafe_code)]
pub mod services;
#[forbid(unsafe_code)]
pub mod zoom_mod;

pub use zoom_mod::ZoomMod;

amml_api::amml_mod!(ZoomMod);
