# Final review: alaydriem-zoom

Date: 2026-10-19
Scope: every source file under `zoom/`, `shim/src`, `shim/include`, `e2e/src`, plus `Cargo.toml`, `mise.toml`, `shim/xmake.lua`, `e2e/xmake.lua`, `tooth.json`, `shim/manifest.json`, `README.md`, `docs/findings/benches/baseline.md`. Checked against the spec, the plan, the project `CLAUDE.md`, the progress ledger rulings, and the LeviLamina 26.51.6 headers. I did not open any third-party zoom or client mod source.

I did not run the build or the tests. The executor reports `check`, `test` (45/45), `header-check`, `build-mod` and `e2e` (18/18) as clean, and I found nothing in the code that disagrees with that.

## Strengths

- **The core is small and correct.** `ZoomTransition` makes the level a pure function of time (`zoom/core/src/structs/ease/transition.rs:26-36`). The ease is frame-rate independent by construction, and a direction change starts from `level(now)`, so the value never jumps (`zoom_controller.rs:101-104`). The duration scales with the distance left, so a reversal halfway takes half the time.
- **Out-of-order time cannot happen at the controller (review focus 4).** `LzHandle::with_controller` reads the clock *inside* the mutex (`zoom/ffi/src/services/handle.rs:21-24`), so the times the controller sees always go up. `level_at` also uses `saturating_sub`, so an earlier time gives `from` (test `level_before_start_is_from`).
- **The FOV maths cannot produce NaN or 0 (review focus 3).** `is_valid_fov` rejects non-finite, `<= 0` and `>= 180` and returns the input unchanged (`zoom_controller.rs:68-70, 106-108`). An invalid `steady_fov` is replaced by `normal_fov`. With a valid base in (0, 180) and `zoom_factor` clamped to [1, 32], `base / factor^level` is always finite and positive.
- **Key released in a menu (review focus 1) works.** When chat opens, the next `getFov` sees `in_gameplay = false` and releases (`zoom_controller.rs:71-73`). The key-up in chat finds `held == false` and does nothing. Closing chat does nothing. A key-up is also honored outside gameplay (`zoom_controller.rs:45`), so the hold clears even if no frame was drawn while the menu was open. Tested: `release_while_out_of_gameplay_clears_hold`, `fov_leaving_gameplay_eases_out`, e2e `leavingGameplay`.
- **Zoom before the first frame (review focus 5) is safe.** `last_base_fov` starts at `DEFAULT_BASE_FOV = 70.0` (`zoom_controller.rs:21, 32`). Tested by `turn_scale_before_first_fov_uses_default_base`.
- **NaN config values cannot reach the FOV (review focus 2, part).** `clamp_float` maps NaN to the default and clamps the rest (`config/zoom.rs:75-81`). JSON cannot hold a NaN literal, and an overflowing literal such as `1e39` becomes `inf` and is then clamped to 32.
- **Calling the original `getFov` twice is safe.** The header declares `LevelRendererPlayer::getFov(float, bool) const` (header line 383) next to `mFov` / `mOFov`, so it is a const interpolation with no side effects. The second call happens only when `enableVariableFOV` is true. The extra cost is negligible.
- **The hook targets match the headers.** `LevelRendererPlayer::getFov` (383), `LevelRendererPlayer::$onWillChangeDimension` (464), `LocalPlayer::_applyTurnDelta(Vec2 const&)` (435), `MinecraftGame::$onAppFocusLost` (1854), `ClientInstance::getMouseGrabbed` (716), `ll::service::getClientInstance`, `KeyInputEvent::keyCode()/isDown()`, `ClientExitLevelEvent`. `Vec2` derives from `floatN2`, so it has `.x` and `.y`.
- **The spec anti-goals are respected in the code.** No `MouseInputEvent` listener, so the scroll wheel is untouched. `KeyInputEvent` is never cancelled. Nothing writes game options. The only file the mod writes is its own default `config.json`.
- **The FFI follows the rules.** Every entry point is a thin delegator inside `LzBoundary::guard` / `guard_value`. Every `unsafe` block has a `// SAFETY:` comment on the line before it. `#![forbid(unsafe_code)]` is set in core and bench. The panic fallbacks are tested. The poisoned-lock fallbacks (`normal_fov`, `1.0`) are the safe values.
- **The project rules are kept.** There is one type per file, `mod.rs` re-exports, no `unwrap`/`expect` in non-test code (also denied by workspace clippy lints), the tests mirror `src/` through one `integration` target, there are no zero-value tests, and comments are on the line before and never name the plan or spec. The shim stays thin: it holds no state, times nothing, and only forwards events and Rust's results.
- **Bench numbers are recorded** (`docs/findings/benches/baseline.md`): FOV query 10.9 ns mid-ease, 3.2 ns settled, turn scale 14.8 ns. The hot paths do not allocate.

## Issues

### Critical

None.

### Important

**I-1. One value of the wrong type throws away the whole config, including `keyCode`.**
- Where: `zoom/core/src/config/loader.rs:32-35`, with `zoom/core/src/config/zoom.rs:6-8` (`u32` durations, `#[serde(default)]` at struct level only).
- What is wrong: serde fails the whole parse on one bad field. For example `"zoomInMs": -100`, `"zoomInMs": 300.5`, `"keyCode": "C"` or `"easing": 3` each makes `load` take the `Err` branch (`loader.rs:25-28`) and return `ZoomConfig::default()`. All the other values the user set are lost, for example a custom `keyCode` or `zoomFactor`. The plan's review focus 2 names "negative durations" as an input that must "load with sane values". It does load sane values, but only by resetting everything, and no test covers it.
- Why it matters: spec 7.5 and `CLAUDE.md` section 15 say "unknown or bad values fall back to defaults ... with a warning", which a user reads as per value. A user who mistypes one duration finds that their zoom key has quietly gone back to tilde. The single warning shows a serde line/column message, not which setting was reset. The README says "If the file cannot be read, the mod uses the defaults", which does not tell the user that a readable file with one bad number counts as "cannot be read".
- Fix: deserialize per field. Either parse into `serde_json::Value` and take each key with its own fallback and `warn!`, or put `#[serde(deserialize_with = ...)]` on each field with a lenient deserializer that falls back to the default and warns (the `serde(default)` free-function exception in `CLAUDE.md` section 6 covers the small default functions; the lenient deserializer can be an associated fn on a struct). Accept negative or fractional durations and clamp them (deserialize as `f64`, then clamp to `[0, MAX_DURATION_MS]` and round). Add loader tests: `{"keyCode": 67, "zoomInMs": -5}` gives key 67 and `zoomInMs` 250 (or 0) with the file kept, and `{"keyCode": 67, "easing": 3}` gives key 67 and `easeOutCubic`.

### Minor

**M-1. The ease starts at full speed, and a reversal flips speed at once.**
- Where: `zoom/core/src/structs/ease/curve.rs:40` (default `EaseOutCubic`), `zoom_controller.rs:101-104`.
- What is wrong: ease-out cubic has its highest speed at t = 0 (slope 3). With the defaults (250 ms, 4x, log-space), the first frame after a press moves the FOV 70° to about 54° at 60 FPS (level 0.187) and 70° to about 44° at 30 FPS (level 0.348). On a reversal the new transition starts at the same full speed in the other direction, so the velocity goes from about +7.7/s to -12/s (in level units) in one frame. Spec 7.2 asks for ease-out ("starts quickly"), so this matches the request. But spec 7.2 also asks the builder to avoid "an abrupt jump in speed where practical", and spec 4 names "a visible snap ... at the start" as an anti-goal.
- Why it matters: comfort is the most important quality of the mod. The large first frame at 30 FPS is the part most likely to read as a snap.
- Fix: no code change before in-game tuning. During hand-off 2 (spec 11, checks 2, 3 and 9), compare `easeOutCubic` with `easeOutSine` (start slope about 1.57) and `easeInOutCubic`. If the reversal feels like a bounce, a velocity-matched reversal is practical: carry the current slope into the new transition, using a cubic Hermite segment or a critically damped spring. This stays a pure function of time, so frame-rate independence is kept.

**M-2. `lz_destroy` can free the handle while a hook on another thread is still using it.**
- Where: `shim/src/bridge/RustBridge.cpp:43-46`, `shim/src/mod/ZoomMod.cpp:37-46`, `zoom/ffi/src/api/lifecycle.rs:25-35`.
- What is wrong: `disable()` unhooks and then destroys. Unhooking stops *new* calls, but a render-thread `FovHook` that already loaded `g_handle` can still be inside `lz_fov`, holding the mutex, when `Box::from_raw` drops it. That is a use-after-free. The window is tens of nanoseconds per frame, so the chance per `/clill reactivate` is very small (about 1e-4 or less). The sibling `levi-freecam` has the same pattern.
- Why it matters: if it happens, `/clill reactivate`, the documented way to reload the config, crashes the game.
- Fix (choose one): (a) keep one handle for the life of the process and add `lz_reload(handle, path)` that swaps the controller under the lock; or (b) in the shim, count in-flight calls with an atomic around each hook body and have `destroy()` wait until the count is zero after the exchange (a bounded spin, `CLAUDE.md` section 18).

**M-3. A poisoned lock logs an error on every call, forever.**
- Where: `zoom/ffi/src/services/handle.rs:26-29`.
- What is wrong: after a panic inside a core call, every `lz_fov` (one or more per frame) and every `lz_turn_scale` logs `"zoom controller lock is poisoned"`. That is hundreds of log lines per second until reactivate. The zoom is also off until then. No core path can panic today, so this is defensive only.
- Fix: log once, using an `AtomicBool` on the handle, or recover with `PoisonError::into_inner` and reset the controller to settled 0.

**M-4. Hook install results are ignored.**
- Where: `shim/src/mod/ZoomMod.cpp:28-31`.
- What is wrong: the return of each `Hook::hook()` is discarded. A hook that fails to resolve gives no log line. Its absence can only be seen from the missing `"... hook active"` line, which also does not appear if the hook resolved but the game never calls it.
- Why it matters: hand-off 1 and the Task 8 decision depend on telling "did not resolve" apart from "did not fire".
- Fix: log an error per hook that fails to install. This is a log line, not a decision, so the shim stays thin.

**M-5. `lz_create` with an empty path returns a handle, but its comment says it returns null.**
- Where: `zoom/ffi/src/services/boundary.rs:57-59`, `zoom/ffi/src/api/lifecycle.rs:9`.
- What is wrong: `text` maps `len == 0` to `Some("")`, so `ConfigLoader::load("")` runs, fails to write, logs a warning and returns a handle with the defaults. The comment says null is returned only for a null or non-UTF-8 path.
- Fix: return null for an empty path in `lz_create`, or document the empty-path behavior. Add the case to `tests/api/lifecycle.rs`.

**M-6. `LzEvent` is taken by value from C.**
- Where: `zoom/ffi/src/api/input.rs:17`.
- What is wrong: a value outside 0..=2 from C is undefined behavior in Rust before the guard runs. The shim passes only the header constants, so nothing triggers it today.
- Fix (optional): take `u32` and convert with a `TryFrom`, returning `InvalidArgument` for an unknown value.

**M-7. Small rule nits.**
- `zoom/ffi/src/services/boundary.rs:49-53`: `LzBoundary::output` is unused (YAGNI). Remove it until an out-parameter exists (Task 8 would add one).
- `shim/src/mod/ZoomMod.cpp:21`: "the Rust core is created here, not in load, so /clill reactivate re-reads the config" records a design decision, which `CLAUDE.md` section 20 says not to store as a comment. It is useful, so the alternative is to keep it and accept the nit.
- `zoom/bench/benches/zoom.rs:13` `pressed()` and `zoom/core/tests/support/controllers.rs` are free helper functions. `CLAUDE.md` section 6 allows bench functions, not helpers. The siblings do the same in tests, so this is consistent with the author's practice. It is a nit only for the bench.
- `README.md` says the FOV "never snaps", but `zoomInMs`/`zoomOutMs` accept 0, which snaps by the user's choice. Change the README to "never snaps with a non-zero duration", or set the minimum duration to 1 ms.

**M-8. The e2e harness covers single-threaded flows only.**
- Where: `e2e/src/main.cpp`.
- What is wrong: all 18 checks call from one thread. The real shim calls `lz_fov` from the render thread and `lz_key` / `lz_turn_scale` / `lz_event` from other threads. The harness also does not check the inputs that the review focus names: key-up with `in_gameplay = 0`, a 0 or NaN base FOV through the C ABI, or `lz_turn_scale` before the first `lz_fov`.
- Fix: add one scenario with a thread calling `lz_fov` in a loop while the main thread presses, releases and sends `FOCUS_LOST`. Assert that every return value is finite and in (0, 70]. Add the three review-focus inputs as one-line checks.

## Declined to judge

- **Is `getFov` also used for the held-item (hand) render?** The headers do not show the callers. If it is, the hand is magnified 4x as well. Plan hand-off 1, item 3, already checks this in game.
- **Does `_applyTurnDelta` fire for mouse look on 26.51.1?** The header exists and freecam hooks the same symbol, but whether it fires needs a running game. This is hand-off 1, item 1, with Task 8 as the fallback.
- **Does `$onAppFocusLost` fire on Alt+Tab?** This needs a running game. The `getMouseGrabbed` gate also releases the zoom when the pause screen opens, so a stuck zoom needs both to fail.
- **Gate is `getMouseGrabbed` only, not also `isShowingMenu`.** Spec 7.1 names both. Mouse-grabbed is a sound proxy for "no non-HUD screen", and freecam's report says the semantics of `isShowingMenu` are not verified. In-game check 6 decides.
- **Ease feel and default timings (250 ms, 4x, easeOutCubic).** The spec gives these to the user in hand-off 2. M-1 gives the numbers to watch.
- **Spyglass, third-person and controller look while zoomed.** The builder-check report lists these as in-game edge cases.
- **Is `KeyInputEvent::keyCode()` a Win32 VK code on 26.51.1, and does it repeat?** The header does not say. The core ignores repeat either way. In-game check 1 shows whether tilde works.
- **Non-US layouts, where VK 192 is not the tilde key.** The key can be changed in the config. This is a README note at most.
- **The shim multiplies the turn delta by Rust's scale (`TurnDeltaHook.h:25-29`).** The plan says to do this. It is "pass Rust's result back", not a decision.
- **Atomic orderings in `RustBridge.cpp` (release/acquire with no comment naming the pair).** `CLAUDE.md` section 17 is written for Rust, and freecam uses the same code.
- **Trailing `} // namespace` comments in C++.** These come from clang-format and match the siblings.
- **Clean-room compliance.** Nothing in the code suggests a third-party source, and I cannot prove a negative.
- **The ledger rulings** (tests run once at the end; no git repo). These are user directives. They had no effect I could see on the result.

## Verdict

**Ready for in-game hand-off 1.** There are no Critical issues. The five review-focus inputs are handled correctly, and four of them have direct tests. The one Important issue (I-1: one bad config value resets every setting) is a small, contained fix in `ConfigLoader` / `ZoomConfig` and should be done before release, not before the first in-game check. Take M-1 into hand-off 2 as a tuning item. Fix M-2 before release if `/clill reactivate` will be the normal reload path. The other Minor items are optional.

Task Completed
