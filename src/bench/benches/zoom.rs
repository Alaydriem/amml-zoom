use std::hint::black_box;

use amm_core::structs::event::{KeybindPhase, ModEvent};
use criterion::{Criterion, criterion_group, criterion_main};
use zoom_core::config::ZoomConfig;
use zoom_core::services::ZoomController;
use zoom_core::structs::FovQuery;

const MID_EASE_US: u64 = 100_000;
const SETTLED_US: u64 = 1_000_000;
const PRESSED: ModEvent = ModEvent::Keybind { action: 0, phase: KeybindPhase::Pressed };
const QUERY: FovQuery = FovQuery { normal_fov: 80.0, steady_fov: 70.0, in_gameplay: true };

fn fov_query_mid_ease(c: &mut Criterion) {
    let mut controller = ZoomController::new(ZoomConfig::default());
    controller.press(true, 0);
    c.bench_function("fov_query_mid_ease", |b| b.iter(|| controller.fov(black_box(QUERY), black_box(MID_EASE_US))));
}

fn fov_query_settled(c: &mut Criterion) {
    let mut controller = ZoomController::new(ZoomConfig::default());
    controller.press(true, 0);
    c.bench_function("fov_query_settled", |b| b.iter(|| controller.fov(black_box(QUERY), black_box(SETTLED_US))));
}

fn turn_scale_mid_ease(c: &mut Criterion) {
    let mut controller = ZoomController::new(ZoomConfig::default());
    controller.press(true, 0);
    c.bench_function("turn_scale_mid_ease", |b| b.iter(|| controller.turn_scale(black_box(MID_EASE_US))));
}

// a press on a settled, held controller: the per-tick event path without the ease restarting
fn apply_keybind_pressed(c: &mut Criterion) {
    let mut controller = ZoomController::new(ZoomConfig::default());
    controller.press(true, 0);
    c.bench_function("apply_keybind_pressed", |b| {
        b.iter(|| controller.apply(black_box(&PRESSED), black_box(true), black_box(SETTLED_US)))
    });
}

criterion_group!(benches, fov_query_mid_ease, fov_query_settled, turn_scale_mid_ease, apply_keybind_pressed);
criterion_main!(benches);
