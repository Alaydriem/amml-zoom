use std::ops::Deref;
use std::time::Duration;

use alaydriem_zoom::services::{ZoomDescriptor, ZoomLink, ZoomSession};
use amm_client::test_support::FakeTable;
use amm_core::structs::event::{KeybindPhase, ModEvent};
use amml_api::testing::FakeGame;

pub const PAST_EASE: Duration = Duration::from_millis(300);
pub const PRESSED: ModEvent = ModEvent::Keybind { action: 0, phase: KeybindPhase::Pressed };
pub const RELEASED: ModEvent = ModEvent::Keybind { action: 0, phase: KeybindPhase::Released };

// a session on a link that is connected to the fake manager of this thread; the game stays alive with it
pub struct Session {
    pub game: FakeGame,
    session: ZoomSession,
}

impl Session {
    // a fake game whose manager tables are provided and announced
    pub fn game() -> FakeGame {
        let game = FakeTable::game();
        FakeTable::provide(&game.ctx()).unwrap();
        FakeTable::ready(&game);
        game
    }

    // a new link of the mod on `game`
    pub fn link(game: &FakeGame) -> ZoomLink {
        ZoomLink::start(&FakeTable::ctx_of(game, ZoomDescriptor::TEXT)).unwrap()
    }

    // the first tick has run, so the link is connected and the settings are read
    pub fn attached() -> Self {
        let game = Self::game();
        let session = ZoomSession::new(game.ctx().log());
        session.attach(Self::link(&game));
        session.tick(true).unwrap();
        Self { game, session }
    }
}

impl Deref for Session {
    type Target = ZoomSession;

    fn deref(&self) -> &ZoomSession {
        &self.session
    }
}
