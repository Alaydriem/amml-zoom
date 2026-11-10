use std::ffi::c_void;

use amml_sdk::{AmmlMod, Api, ModDirs, SdkError};

use crate::detours::{apply_turn_delta, client_update, dimension_change, focus_lost, get_fov, level_exit};
use crate::services::{AmmlHost, HookInstaller, ZoomRuntime};
use crate::traits::GameHost;
use crate::types::HookSpec;

// the AMML entry point: load makes the runtime, enable installs the hooks, disable removes them and leaves the
// mod manager. The manager link is made on the first ClientInstance::update after enable
pub struct ZoomMod;

impl ZoomMod {
    // the mod id as a static string for the host and logger; a test checks it equals AmmlMod::id()
    pub const MODULE: &'static str = "@alaydriem/zoom";
    pub const UPDATE: &'static str = "ClientInstance.update";
    pub const MOUSE_GRABBED: &'static str = "ClientInstance.getMouseGrabbed";
    pub const GET_FOV: &'static str = "LevelRendererPlayer.getFov";
    pub const DIMENSION_CHANGE: &'static str = "LevelRendererPlayer.onWillChangeDimension";
    pub const APPLY_TURN_DELTA: &'static str = "LocalPlayer.applyTurnDelta";
    pub const FOCUS_LOST: &'static str = "MinecraftGame.onAppFocusLost";
    pub const LEVEL_EXIT: &'static str = "ClientExitLevel.hook";

    // every hook, in install order; the client update and the FOV are the zoom itself, the rest are optional
    pub fn hooks() -> [HookSpec; 6] {
        [
            HookSpec {
                entry: Self::GET_FOV,
                detour: get_fov::detour as *const c_void,
                original: get_fov::GUARD.original(),
                required: true,
                feature: "the zoom",
            },
            HookSpec {
                entry: Self::APPLY_TURN_DELTA,
                detour: apply_turn_delta::detour as *const c_void,
                original: apply_turn_delta::GUARD.original(),
                required: false,
                feature: "look speed matching",
            },
            HookSpec {
                entry: Self::FOCUS_LOST,
                detour: focus_lost::detour as *const c_void,
                original: focus_lost::GUARD.original(),
                required: false,
                feature: "the reset on focus loss",
            },
            HookSpec {
                entry: Self::DIMENSION_CHANGE,
                detour: dimension_change::detour as *const c_void,
                original: dimension_change::GUARD.original(),
                required: false,
                feature: "the reset on dimension change",
            },
            HookSpec {
                entry: Self::LEVEL_EXIT,
                detour: level_exit::detour as *const c_void,
                original: level_exit::GUARD.original(),
                required: false,
                feature: "the reset on leaving the world",
            },
            HookSpec {
                entry: Self::UPDATE,
                detour: client_update::detour as *const c_void,
                original: client_update::GUARD.original(),
                required: true,
                feature: "the zoom",
            },
        ]
    }

    // the body of enable: the getMouseGrabbed slot and the required entries must resolve, then the hooks go in
    // and the next tick reaches the manager
    pub fn enable_with<H: GameHost>(host: &H, runtime: &ZoomRuntime) -> Result<(), SdkError> {
        let slot = host.resolve(Self::MOUSE_GRABBED)?;
        let slot = u32::try_from(slot).map_err(|_| SdkError::Mod(format!("{} is not a slot", Self::MOUSE_GRABBED)))?;
        runtime.client().set_mouse_grabbed_slot(slot);
        HookInstaller::install(host, runtime, &Self::hooks())?;
        runtime.enable_link();
        Ok(())
    }
}

impl AmmlMod for ZoomMod {
    const NAMESPACE: &'static str = "alaydriem";
    const NAME: &'static str = "zoom";

    fn load(api: &Api) -> Result<(), SdkError> {
        let dirs = ModDirs::new(api, Self::MODULE)?;
        ZoomRuntime::install(&dirs.config_dir);
        Ok(())
    }

    fn enable(api: &Api) -> Result<(), SdkError> {
        let runtime = ZoomRuntime::get().ok_or_else(|| SdkError::Mod("load did not run".to_string()))?;
        Self::enable_with(&AmmlHost::new(api, Self::MODULE), runtime)
    }

    fn disable(api: &Api) {
        let Some(runtime) = ZoomRuntime::get() else { return };
        HookInstaller::remove_all(&AmmlHost::new(api, Self::MODULE), runtime);
        runtime.detach();
    }
}
