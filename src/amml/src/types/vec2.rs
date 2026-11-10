// the game's Vec2: two floats, as `LocalPlayer::_applyTurnDelta` reads them
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn scaled(self, factor: f32) -> Self {
        Self { x: self.x * factor, y: self.y * factor }
    }
}
