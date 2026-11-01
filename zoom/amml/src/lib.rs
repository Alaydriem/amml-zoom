//! @alaydriem/zoom as an AMML mod: hooks the game's field of view, look turn, focus loss, dimension change,
//! level exit and client update, and drives zoom-core with them.

#![deny(unsafe_code)]

pub mod detours;
pub mod services;
pub mod traits;
pub mod types;
pub mod zoom_mod;

pub use zoom_mod::ZoomMod;

amml_sdk::amml_mod!(ZoomMod);
