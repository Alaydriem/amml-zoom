use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Mutex, OnceLock, PoisonError};

use amm_client::link::ApiTable;
use amm_core::errors::LinkError;
use amml_sdk::{HookHandle, ModRuntime};
use zoom_core::structs::ZoomEvent;

use crate::services::{GameClient, ZoomSession};
use crate::types::{LinkState, Vec2};

// what the hooks share: the zoom session, the game client, the manager link state and the installed hooks.
// The mod manager is reached on the first ClientInstance::update after enable, when every mod of the host
// has loaded; `hooks` holds at most HOOK_CAPACITY handles
pub struct ZoomRuntime {
    session: ZoomSession,
    client: GameClient,
    config_dir: PathBuf,
    link_state: AtomicU8,
    hooks: Mutex<Vec<HookHandle>>,
}

static RUNTIME: OnceLock<ZoomRuntime> = OnceLock::new();

impl ZoomRuntime {
    pub const HOOK_CAPACITY: usize = 8;

    pub fn new(config_dir: &Path) -> Self {
        Self {
            session: ZoomSession::new(),
            client: GameClient::new(),
            config_dir: config_dir.to_path_buf(),
            link_state: AtomicU8::new(LinkState::Waiting.to_u8()),
            hooks: Mutex::new(Vec::with_capacity(Self::HOOK_CAPACITY)),
        }
    }

    // the runtime of this DLL; the first call wins
    pub fn install(config_dir: &Path) -> &'static ZoomRuntime {
        RUNTIME.get_or_init(|| Self::new(config_dir))
    }

    pub fn get() -> Option<&'static ZoomRuntime> {
        RUNTIME.get()
    }

    pub fn session(&self) -> &ZoomSession {
        &self.session
    }

    pub fn client(&self) -> &GameClient {
        &self.client
    }

    pub fn link_state(&self) -> LinkState {
        LinkState::from_u8(self.link_state.load(Ordering::SeqCst))
    }

    // enable: the next tick reaches the manager again
    pub fn enable_link(&self) {
        self.link_state.store(LinkState::Waiting.to_u8(), Ordering::SeqCst);
    }

    // once per ClientInstance::update on the game thread, after the original: keeps the client, reaches the
    // manager on the first call after enable, then hands the manager's events to the zoom
    pub fn tick(&self, client: usize) {
        self.client.capture(client);
        if self.link_state() == LinkState::Waiting {
            self.attach_with(ApiTable::resolve());
        }
        if self.link_state() == LinkState::Attached {
            let _ = self.session.tick(self.client.in_gameplay());
        }
    }

    // registers with the manager the table belongs to; on failure the zoom stays off and one line is logged.
    // Only a Waiting runtime takes the result: if disable ran meanwhile, a new link is dropped again
    pub fn attach_with(&self, table: Result<ApiTable, LinkError>) -> LinkState {
        let state = match table.and_then(|table| self.session.attach(table, &self.config_dir)) {
            Ok(()) => LinkState::Attached,
            Err(error) => {
                Self::log_link_error(&error);
                LinkState::Off
            }
        };
        let waiting = LinkState::Waiting.to_u8();
        match self.link_state.compare_exchange(waiting, state.to_u8(), Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => state,
            Err(current) => {
                if state == LinkState::Attached {
                    self.session.detach();
                }
                LinkState::from_u8(current)
            }
        }
    }

    // disable: the zoom returns to normal and unregisters; no tick reaches the manager until the next enable
    pub fn detach(&self) {
        self.link_state.store(LinkState::Disabled.to_u8(), Ordering::SeqCst);
        self.session.detach();
    }

    // the field of view for one getFov call; `original` runs the game's getFov with the given
    // enableVariableFOV. Without the manager link the game's value is returned unchanged. The steady value is
    // asked for first, so the last call of `original` is always the one with the game's own arguments: the
    // detour's panic fallback returns the last original result
    pub fn fov(&self, variable: bool, original: impl Fn(bool) -> f32) -> f32 {
        if self.link_state() != LinkState::Attached {
            return original(variable);
        }
        let steady = if variable { Some(original(false)) } else { None };
        let normal = original(variable);
        self.session.fov(normal, steady.unwrap_or(normal), self.client.in_gameplay())
    }

    // the look delta scaled to match the zoom; unchanged without the manager link
    pub fn turn(&self, delta: Vec2) -> Vec2 {
        if self.link_state() != LinkState::Attached {
            return delta;
        }
        delta.scaled(self.session.turn_scale())
    }

    pub fn event(&self, event: ZoomEvent) {
        if self.link_state() == LinkState::Attached {
            self.session.event(event);
        }
    }

    // keeps an installed hook; false, keeping nothing, when HOOK_CAPACITY hooks are already kept
    pub fn keep_hook(&self, hook: HookHandle) -> bool {
        let mut hooks = self.hooks.lock().unwrap_or_else(PoisonError::into_inner);
        if hooks.len() >= Self::HOOK_CAPACITY {
            return false;
        }
        hooks.push(hook);
        true
    }

    // the installed hooks, newest first, leaving none behind
    pub fn take_hooks(&self) -> Vec<HookHandle> {
        let mut hooks = self.hooks.lock().unwrap_or_else(PoisonError::into_inner);
        let mut taken = std::mem::take(&mut *hooks);
        taken.reverse();
        taken
    }

    fn log_link_error(error: &LinkError) {
        let Some(runtime) = ModRuntime::get() else { return };
        match error {
            LinkError::ManagerMissing | LinkError::AbiMismatch { .. } => {
                runtime.logger().error("@alaydriem/zoom needs @amml/modmanager (ABI 2.x); zoom is off");
            }
            other => {
                runtime.logger().error(&format!("@amml/modmanager refused @alaydriem/zoom ({other}); zoom is off"));
            }
        }
    }
}
