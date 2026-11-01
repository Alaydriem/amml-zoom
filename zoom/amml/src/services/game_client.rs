use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

// the game's primary ClientInstance and the vtable slot of IClientInstance::getMouseGrabbed.
// The client is the first `this` seen by ClientInstance::update: the primary client is created at start-up
// and lives until the game closes, while split-screen clients come and go
pub struct GameClient {
    client: AtomicUsize,
    mouse_grabbed_slot: AtomicU32,
}

impl GameClient {
    const NO_SLOT: u32 = u32::MAX;

    pub const fn new() -> Self {
        Self { client: AtomicUsize::new(0), mouse_grabbed_slot: AtomicU32::new(Self::NO_SLOT) }
    }

    pub fn set_mouse_grabbed_slot(&self, slot: u32) {
        self.mouse_grabbed_slot.store(slot, Ordering::Release);
    }

    // keeps the first non-null client; later calls change nothing
    pub fn capture(&self, client: usize) {
        if client != 0 {
            let _ = self.client.compare_exchange(0, client, Ordering::AcqRel, Ordering::Acquire);
        }
    }

    pub fn client(&self) -> usize {
        self.client.load(Ordering::Acquire)
    }

    // the client exists and has grabbed the mouse, so no menu, chat or other screen is open;
    // false before the first update or without the slot
    pub fn in_gameplay(&self) -> bool {
        let client = self.client();
        let slot = self.mouse_grabbed_slot.load(Ordering::Acquire);
        if client == 0 || slot == Self::NO_SLOT {
            return false;
        }
        let Ok(slot) = usize::try_from(slot) else { return false };
        // SAFETY: `client` is the ClientInstance* the game passed to ClientInstance::update, and the primary
        // client lives until the game closes. Its first qword is the IClientInstance vtable, and `slot` is the
        // derived ClientInstance.getMouseGrabbed entry, which the recipe range check keeps inside that vtable.
        // The function there is `bool getMouseGrabbed() const`, read as a byte so no bool invariant is assumed.
        #[allow(unsafe_code)]
        let grabbed = unsafe {
            let vtable = *(client as *const *const usize);
            let function = *vtable.add(slot);
            let get: extern "C" fn(usize) -> u8 = std::mem::transmute(function);
            get(client)
        };
        grabbed != 0
    }
}

impl Default for GameClient {
    fn default() -> Self {
        Self::new()
    }
}
