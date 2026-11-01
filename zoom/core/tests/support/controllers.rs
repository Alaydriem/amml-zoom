use zoom_core::config::ZoomConfig;
use zoom_core::services::ZoomController;

pub const FULL: u64 = 250_000;

pub fn controller() -> ZoomController {
    ZoomController::new(ZoomConfig::default())
}

pub fn zoomed_at(now_us: u64) -> ZoomController {
    let mut controller = controller();
    controller.press(true, 0);
    assert_eq!(controller.level(now_us), 1.0);
    controller
}
