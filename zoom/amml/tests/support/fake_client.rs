// a stand-in for the game's ClientInstance: its first qword points to a one-slot vtable whose function answers
// getMouseGrabbed
#[repr(C)]
pub struct FakeClient {
    vtable: &'static [extern "C" fn(usize) -> u8; 1],
}

extern "C" fn grabbed(_client: usize) -> u8 {
    1
}

extern "C" fn released(_client: usize) -> u8 {
    0
}

static GRABBED: [extern "C" fn(usize) -> u8; 1] = [grabbed];
static RELEASED: [extern "C" fn(usize) -> u8; 1] = [released];

impl FakeClient {
    pub const SLOT: u32 = 0;

    pub fn new(mouse_grabbed: bool) -> Self {
        Self { vtable: if mouse_grabbed { &GRABBED } else { &RELEASED } }
    }

    pub fn address(&self) -> usize {
        std::ptr::from_ref(self) as usize
    }
}
