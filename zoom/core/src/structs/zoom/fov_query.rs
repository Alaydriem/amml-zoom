// one getFov call from the game, in degrees
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FovQuery {
    // with the sprint and effect changes
    pub normal_fov: f32,
    // without the sprint and effect changes
    pub steady_fov: f32,
    // the mouse is grabbed by the game and no menu or chat is open
    pub in_gameplay: bool,
}
