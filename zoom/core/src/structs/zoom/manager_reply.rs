// what the FFI layer must do after the controller handled a manager event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagerReply {
    Nothing,
    ReloadSettings,
}
