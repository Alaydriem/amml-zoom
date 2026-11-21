use alaydriem_zoom::ZoomMod;
use amm_client::test_support::FakeTable;
use amm_core::structs::event::{KeybindPhase, ModEvent};
use amml_api::Area;
use amml_api::amml_abi::{CameraFov, ClientTick, DimensionChanging, LookDelta, WindowFocus, WorldLeft};
use amml_api::testing::{FakeClient, FakeGame};

// the zoom mod enabled on a fake game that serves every event of its `USES`, with no easing time
pub struct ZoomRig {
    pub game: FakeGame,
}

impl ZoomRig {
    // the mod manager is provided and the mouse is captured when `captured`
    pub fn with_manager(captured: bool) -> Self {
        Self::build(true, captured)
    }

    // the mod manager is not installed
    pub fn without_manager() -> Self {
        Self::build(false, true)
    }

    fn build(manager: bool, captured: bool) -> Self {
        let game = FakeTable::game();
        game.offer::<ClientTick>();
        game.offer::<CameraFov>();
        game.offer::<LookDelta>();
        game.offer::<WindowFocus>();
        game.offer::<WorldLeft>();
        game.offer::<DimensionChanging>();
        game.offer_area(Area::CLIENT_MOUSE);
        if manager {
            FakeTable::provide(&game.ctx()).unwrap();
        }
        FakeTable::set_config("zoom {\n  zoom_in_ms = 0\n  zoom_out_ms = 0\n}\n");
        game.load::<ZoomMod>().unwrap();
        game.enable::<ZoomMod>().unwrap();
        game.open_gate();
        let rig = Self { game };
        rig.capture(Some(captured));
        rig
    }

    // what the game reports for the mouse from now on
    pub fn capture(&self, captured: Option<bool>) {
        self.game.set_client(FakeClient { mouse_captured: captured, ..FakeClient::default() });
    }

    pub fn tick(&self) {
        self.game.raise_with_client(&mut ClientTick { header: FakeGame::header::<ClientTick>() });
    }

    pub fn press(&self) {
        self.key(KeybindPhase::Pressed);
    }

    pub fn release(&self) {
        self.key(KeybindPhase::Released);
    }

    // the field of view a `CameraFov` call leaves
    pub fn fov(&self, base: f32, steady: f32) -> f32 {
        let mut event = CameraFov { header: FakeGame::header::<CameraFov>(), base, steady, result: base, _reserved: 0 };
        self.game.raise(&mut event);
        event.result
    }

    // the delta a `LookDelta` leaves; `injected` marks the turn as one the loader injected
    pub fn look(&self, yaw: f32, pitch: f32, injected: bool) -> (f32, f32) {
        let mut header = FakeGame::header::<LookDelta>();
        header.injected = u8::from(injected);
        let mut event = LookDelta { header, yaw, pitch };
        self.game.raise(&mut event);
        (event.yaw, event.pitch)
    }

    pub fn focus_lost(&self) {
        let mut event =
            WindowFocus { header: FakeGame::header::<WindowFocus>(), gained: 0, replayed: 0, _reserved: [0; 2] };
        self.game.raise(&mut event);
    }

    pub fn world_left(&self) {
        self.game.raise(&mut WorldLeft { header: FakeGame::header::<WorldLeft>(), epoch: 0 });
    }

    pub fn dimension_changing(&self) {
        self.game.raise(&mut DimensionChanging { header: FakeGame::header::<DimensionChanging>() });
    }

    // true when the log holds `line` (level, message and fields as the fake writes them)
    pub fn logged(&self, line: &str) -> bool {
        self.game.logs().iter().any(|logged| logged == line)
    }

    fn key(&self, phase: KeybindPhase) {
        FakeTable::push_event(ModEvent::Keybind { action: 0, phase });
        self.tick();
    }
}
