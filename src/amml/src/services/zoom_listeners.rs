use std::sync::Arc;

use amml_api::amml_abi::{CameraFov, ClientTick, DimensionChanging, LookDelta, WindowFocus, WorldLeft};
use amml_api::{ApiError, Ctx, Event, HookOrder, Listener, Look, LookDeltaExt, WindowFocusExt};
use zoom_core::structs::ZoomEvent;

use crate::services::ZoomRuntime;

// the loader listeners of the zoom; each only hands its event to the runtime. Dropping them removes them
pub struct ZoomListeners {
    listeners: Vec<Listener>,
    runtime: Arc<ZoomRuntime>,
}

impl ZoomListeners {
    // the tick and the camera field of view are the zoom itself, the rest are optional
    pub fn start(ctx: &Ctx, runtime: Arc<ZoomRuntime>) -> Result<Self, ApiError> {
        let events = ctx.events();
        let mut listeners = Vec::with_capacity(6);
        let ticking = Arc::clone(&runtime);
        listeners.push(events.listen::<ClientTick>(HookOrder::NORMAL, move |_, at| {
            let in_gameplay = at.client().and_then(|client| client.mouse_captured()).unwrap_or(false);
            ticking.tick(in_gameplay);
        })?);
        let seeing = Arc::clone(&runtime);
        listeners.push(events.listen::<CameraFov>(HookOrder::NORMAL, move |event, _| {
            event.result = seeing.fov(event.base, event.steady);
        })?);
        let turning = Arc::clone(&runtime);
        listeners.extend(Self::optional(events.listen::<LookDelta>(HookOrder::NORMAL, move |event, _| {
            if event.is_injected() {
                return;
            }
            let scale = turning.turn_scale();
            let delta = event.delta();
            event.set_delta(Look { yaw: delta.yaw * scale, pitch: delta.pitch * scale });
        }))?);
        let focus = Arc::clone(&runtime);
        listeners.extend(Self::optional(events.listen::<WindowFocus>(HookOrder::NORMAL, move |event, _| {
            if !event.gained() {
                focus.event(ZoomEvent::FocusLost);
            }
        }))?);
        let leaving = Arc::clone(&runtime);
        listeners.extend(Self::optional(
            events.listen::<WorldLeft>(HookOrder::NORMAL, move |_, _| leaving.event(ZoomEvent::WorldLeft)),
        )?);
        let changing = Arc::clone(&runtime);
        listeners.extend(Self::optional(events.listen::<DimensionChanging>(HookOrder::NORMAL, move |_, _| {
            changing.event(ZoomEvent::DimensionChanged);
        }))?);
        Ok(Self { listeners, runtime })
    }

    // the listeners go first, so no listener runs while the runtime lets go of the link
    pub fn stop(self) {
        let Self { listeners, runtime } = self;
        drop(listeners);
        runtime.detach();
    }

    // a listen the loader cannot serve is skipped; any other failure is passed on
    fn optional(listen: Result<Listener, ApiError>) -> Result<Option<Listener>, ApiError> {
        match listen {
            Ok(listener) => Ok(Some(listener)),
            Err(ApiError::NotAvailable) => Ok(None),
            Err(error) => Err(error),
        }
    }
}
