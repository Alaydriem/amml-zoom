// game events that end a zoom
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomEvent {
    FocusLost,
    DimensionChanged,
    WorldLeft,
}
