use std::sync::{Arc, Mutex, PoisonError};

use amml_api::amml_abi::{
    CameraFov, ClientTick, DimensionChanging, InjectedInputReleased, InputLeaseLost, InputLeaseResumed,
    InputLeaseSuspended, LookDelta, ModsLoaded, ProviderGone, ProviderReady, ScreenshotTaken, ScreenshotWritten,
    WindowFocus, WorldLeft,
};
use amml_api::{AmmlMod, ApiError, Area, Ctx, DisableCtx, Uses};

use crate::services::{ZoomLink, ZoomListeners, ZoomRuntime};

// what the enabled mod owns. `disable` takes it, which drops the listeners and closes the link
static RUNNING: Mutex<Option<ZoomListeners>> = Mutex::new(None);

// the AMML entry point: enable starts the manager link and the listeners, disable removes them and leaves the
// mod manager
pub struct ZoomMod;

impl AmmlMod for ZoomMod {
    const NAMESPACE: &'static str = "alaydriem";
    const NAME: &'static str = "zoom";
    const USES: Uses = amml_api::amml_uses! {
        feature "zoom" {
            required ClientTick;
            required CameraFov;
            required ProviderReady;
            required ProviderGone;
            required area Area::CLIENT_MOUSE;
            // The link listens to these when the loader serves them
            optional ModsLoaded;
            optional InputLeaseSuspended;
            optional InputLeaseResumed;
            optional InputLeaseLost;
            optional InjectedInputReleased;
            optional ScreenshotTaken;
            optional ScreenshotWritten;
            optional area Area::INPUT_LEASE;
        }
        feature "look" {
            optional LookDelta;
        }
        feature "reset" {
            optional WindowFocus;
            optional WorldLeft;
            optional DimensionChanging;
        }
    };

    fn enable(ctx: &Ctx) -> Result<(), ApiError> {
        let log = ctx.log();
        let link = ZoomLink::start(ctx).map_err(|error| {
            log.error("zoom off", &[("error", &error.to_string())]);
            ApiError::Refused
        })?;
        let runtime = Arc::new(ZoomRuntime::new(log));
        runtime.attach(link);
        let listeners = ZoomListeners::start(ctx, runtime).inspect_err(|error| {
            log.error("zoom off", &[("error", &error.to_string())]);
        })?;
        *RUNNING.lock().unwrap_or_else(PoisonError::into_inner) = Some(listeners);
        Ok(())
    }

    fn disable(_ctx: &Ctx, _how: &DisableCtx) {
        let running = RUNNING.lock().unwrap_or_else(PoisonError::into_inner).take();
        if let Some(listeners) = running {
            listeners.stop();
        }
    }
}
