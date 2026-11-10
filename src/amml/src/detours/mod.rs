// The game functions zoom hooks. Each body hands the raw arguments to ZoomRuntime and calls the original;
// bool arguments and results cross as bytes, so no bool invariant is assumed of the game's registers

use amml_sdk::amml_detour;
use zoom_core::structs::ZoomEvent;

use crate::services::ZoomRuntime;
use crate::types::Vec2;

// bool ClientInstance::update(bool isInitFinished) (26.60: update(bool, bool)): once per frame on the game thread
// 26.60 added a bool third argument (r8). It is carried as a raw u64 so the register reaches the original
// bit for bit; 26.51 ignores r8, so one declaration works on both builds
amml_detour!(pub mod client_update(this: usize, init_finished: u8, extra: u64) -> u8 |original| {
    let result = original(this, init_finished, extra);
    if let Some(runtime) = ZoomRuntime::get() {
        runtime.tick(this);
    }
    result
});

// float LevelRendererPlayer::getFov(float a, bool enableVariableFOV) const: the field of view the camera is
// built from; the zoom replaces it. ZoomRuntime::fov calls the original with the game's own arguments last, so
// on a panic the guard returns that result, the value the game would have had
amml_detour!(pub mod get_fov(this: usize, a: f32, variable: u8) -> f32 |original| {
    match ZoomRuntime::get() {
        Some(runtime) => runtime.fov(variable != 0, |variable| original(this, a, u8::from(variable))),
        None => original(this, a, variable),
    }
});

// void LocalPlayer::_applyTurnDelta(Vec2 const& turnOffset): mouse look reaches the player here
amml_detour!(pub mod apply_turn_delta(this: usize, delta: &Vec2) -> () |original| {
    match ZoomRuntime::get() {
        Some(runtime) => original(this, &runtime.turn(*delta)),
        None => original(this, delta),
    }
});

// void MinecraftGame::onAppFocusLost(): Alt+Tab and similar, where the key-up may never arrive. `this` is the
// AppPlatformListener subobject (MinecraftGame + 0x90), passed on unchanged
amml_detour!(pub mod focus_lost(this: usize) -> () |original| {
    original(this);
    if let Some(runtime) = ZoomRuntime::get() {
        runtime.event(ZoomEvent::FocusLost);
    }
});

// void LevelRendererPlayer::onWillChangeDimension(Player& player). `this` is the PlayerListener subobject
// (LevelRendererPlayer + 0xC90), passed on unchanged
amml_detour!(pub mod dimension_change(this: usize, player: usize) -> () |original| {
    original(this, player);
    if let Some(runtime) = ZoomRuntime::get() {
        runtime.event(ZoomEvent::DimensionChanged);
    }
});

// void ClientInstance::onLevelExit(), the function LeviLamina raises ClientExitLevelEvent from; like that event,
// the zoom hears of it before the original runs. `this` is the GameCallbacks subobject (ClientInstance + 0x88),
// passed on unchanged
amml_detour!(pub mod level_exit(this: usize) -> () |original| {
    if let Some(runtime) = ZoomRuntime::get() {
        runtime.event(ZoomEvent::WorldLeft);
    }
    original(this);
});
