# Levi Zoom: design

Date: 2026-10-19
Status: draft, waiting for user approval

## 1. Intent

Java Edition has mods that zoom the camera while you hold a key. This project is a **clean-room LeviLamina client mod** for Minecraft Bedrock on Windows that does the same thing.

The most important quality is **comfort**. The zoom must never be a sudden FOV change. It must glide in and out so smoothly that it feels natural.

## 2. Goals

1. Hold a key to zoom in. Release it to zoom out.
2. The change between normal and zoomed view is eased and very comfortable.
3. While zoomed, mouse look slows down to match the zoom.
4. The key, the zoom amount, and the ease timing can be set in a config file, and the file can be reloaded without a game restart.
5. It works in any world and on any server. It is client-only. The server does not need to know about it.

## 3. Non-goals

- No toggle mode. Hold only.
- No scroll-wheel zoom adjustment.
- No in-game Controls menu binding (the config file is enough for now).
- No in-game settings screen.
- No hiding of the hand, HUD, or other visual changes.
- No support for the Microsoft Store game build or for game versions other than the target (section 5).
- No server-side build.

## 4. Anti-goals (a failure even if it works)

- A visible snap, jerk, or jump in FOV at any point: at the start, at the end, or when the key changes direction mid-ease.
- An ease that feels different at 30 FPS and at 240 FPS.
- **Any change to scroll-wheel behavior.** The wheel must work exactly as in vanilla, zoomed or not.
- A zoom that gets stuck on (for example, after Alt+Tab).
- Any change to the user's saved game settings (FOV, sensitivity).
- Code copied from, or written while reading, third-party zoom or client mods (section 6).
- A break of any rule in the project `CLAUDE.md` (section 5).

## 5. Hard constraints (set by the user)

- **Language and shape:** a **Rust** project with a **thin C++ shim**, the same shape as the user's `../levi-minimap` and `../levi-freecam`.
- **Project rules:** the project `CLAUDE.md` (copied from `../levi-minimap/CLAUDE.md`) is NON-NEGOTIABLE. It covers, among other things:
  - all logic in Rust core, `#![forbid(unsafe_code)]`, no game or OS dependencies in core
  - `unsafe` only in the FFI crate, thin `extern "C"` delegators inside a boundary guard with `catch_unwind`, a `// SAFETY:` comment on each `unsafe` block, cbindgen-generated header that is never hand-edited
  - the shim does not decide anything (section 4 of CLAUDE.md)
  - one struct per file, sub-packages by domain, `mod.rs` re-exports
  - services and managers, constructor injection, `new()` / `new_shared()`, no builders, no free functions, no trait objects (enum dispatch)
  - `thiserror` domain errors, a `#[repr(C)]` status enum at the boundary, no `unwrap`/`expect` outside tests
  - bounded memory, no allocation per frame on the hot path, a Criterion bench for each hot path, performance claims backed by measurements
  - `log` crate forwarded to the LeviLamina logger, no `println!`
  - tests in a crate-root `tests/` tree that mirrors `src/`, one aggregator target, public-API behavior tests only, no zero-value tests, `cargo nextest`
  - all builds and tests run through `mise run <task>`
  - comments on the line before, never inline, never referencing plan or spec steps
- **Platform:** Windows 10/11 x64, Minecraft Bedrock client **26.51.1**, installed through LeviLauncher.
- **Loader:** LeviLamina **26.51.x** client build (latest at time of writing: v26.51.7). Pin to match the sibling projects.
- **Toolchain:** pinned like the sibling projects: Rust 1.98.0 (edition 2024) in `rust-toolchain.toml`, `x86_64-pc-windows-msvc`; shim built by xmake with clang-cl and MSVC 2022, from the LeviLamina mod template conventions.
- **No commits.** The user reviews all changes before anything is committed. Checkpoints are "run the tests", not commits.

## 6. Clean-room rule

The builder must **not** read, copy, or adapt code from third-party zoom or client mods. That includes Zoomidy, LaminaView, ZoomIntoBedrock, Flarial, Latite, VidereLonge, Bedrock-Zoom, FastMiner, and similar projects.

Allowed sources:

- The user's own projects: **`../levi-minimap`** and **`../levi-freecam`**. These are the main examples for project layout, mise tasks, FFI boundary, shim structure, hooks, config loading, logging, and install.
- First-party LiteLDev / LeviMC material: LeviLamina source, public headers and docs, the official mod template, `ShrBox/ExampleMod`, `LiteLDev/LeviSchematic`.
- Microsoft and Win32 docs, Rust crate docs, general math and easing references.

`docs/findings/levilamina-client-research/findings.md` names hook points that other mods use. Those names are facts about LeviLamina's public headers, and the builder may use them. The builder must not follow the links in that file to third-party mod source code.

## 7. Behavior (decided with the user)

### 7.1 Key

- Default key: **tilde / backtick** (Win32 `VK_OEM_3` = 192 on a US layout). OptiFine uses C; the user chose tilde.
- The key is set in the config file as a Win32 virtual-key code.
- Hold to zoom. Releasing the key starts the zoom-out.
- Key repeat (repeated key-down while held) must not restart the ease.
- The mod must **not** block or consume the key event, so other uses of the key still work.
- Zoom starts only during gameplay: the mouse is grabbed and no non-HUD screen (chat, inventory, pause) is open.

### 7.2 Ease feel

- On press, the FOV glides from normal to zoomed. On release, it glides back.
- The ease starts quickly and settles softly at the end (ease-out character), so there is no hard stop.
- If the key changes direction mid-ease, the zoom turns back **from its current value** with no jump in value. The builder should also avoid an abrupt jump in *speed* where practical.
- The ease depends on real time, not on frame count, so it feels the same at any frame rate.
- The default timing and curve must feel right without config changes. The user will tune them in-game (section 11).

### 7.3 Mouse look while zoomed

- While zoomed, look sensitivity scales down to match the zoom, so the view under the crosshair moves at about the same on-screen speed as at normal FOV.
- The scale follows the eased zoom value, so it eases together with the FOV.
- It must not change the user's saved sensitivity setting.

### 7.4 Scroll wheel

- No behavior change at all.

### 7.5 Config

- A JSON config file in the mod's config folder (`mods/alaydriem-zoom/config/config.json`), read with `serde_json` in core (CLAUDE.md section 15), the same way the sibling projects load config.
- If the file is missing, the mod writes it with defaults. Unknown or bad values fall back to defaults or are clamped, with a clear warning in the log.
- The user can edit it and reload it **without restarting the game**.

### 7.6 Stuck-zoom resets

The zoom returns to normal when: the window loses focus, a non-HUD screen opens or the mouse is released, the dimension changes, or the player leaves the world.

## 8. Builder's calls (decided by Claude, not by the user; may change after in-game tuning)

| Item | Default |
|---|---|
| Names (aligned with the sibling mods) | mod / folder / xmake target `alaydriem-zoom`, display name "Alaydriem's Zoom", author "Alaydriem", repo and lip tooth `github.com/alaydriem/levi-zoom`, release zip `alaydriem-zoom-<version>-client-windows-x64.zip`, LeviLamina dependency `>=26.51.6 <26.52.0`, manifest `"platform": "client"`, `"clientSide": true`; crates `zoom-core`, `zoom-ffi` (and `zoom-bench`) under `zoom/`; C prefix `lz_`; header `shim/include/levi_zoom.h`; status enum `LzStatus`; guard `LzBoundary` |
| Zoom amount | 4× (FOV divided by 4; for example 70° to 17.5°) |
| Zoom-in / zoom-out duration | 250 ms each |
| Curve | `easeInOutCubic` (starts and stops gently; an ease-out started at full speed and jumped about 16° on the first frame at 60 FPS), chosen by name in config |
| Interpolation | in log space of the zoom factor, so the zoom feels even in speed |
| Sensitivity match | exact: `tan(zoomedFov/2) / tan(baseFov/2)`, times a config strength (1.0 = exact, 0 = off) |
| Sprint / effect FOV while zoomed | faded out while zoomed so sprint does not make the zoom wobble. The shim passes both the normal and the "no variable FOV" base values to core; core blends them by ease progress, so there is no snap |
| Reset behavior | ease back when the game still renders (screen opened, dimension change); snap on focus loss and on leaving the world |
| Config reload | `/clill reactivate alaydriem-zoom`: the shim creates the Rust handle once in `load()`; `enable()` calls `lz_reload`, which re-reads the file; `disable()` only unhooks, so a render-thread call can never reach a freed handle. No file-change poll |
| Config fields | `version`, `keyCode`, `zoomFactor`, `zoomInMs`, `zoomOutMs`, `easing`, `sensitivityMatchStrength`, `fadeVariableFovWhileZoomed` |
| Clock | core reads time through an injected clock (monotonic milliseconds or nanoseconds passed in from FFI), so tests control time |

## 9. Architecture

The layout follows `../levi-freecam` (the closer sibling: camera + input + config).

| Part | Path | Owns |
|---|---|---|
| `zoom-core` | `zoom/core/` | All logic: config model, loading, validation and defaults; key state (press/release latch, repeat filter, gameplay gate decision); ease state machine (direction changes from the current value); easing curves; FOV calculation including the variable-FOV blend; sensitivity scale; reset policy. `#![forbid(unsafe_code)]`. No game, OS or GPU dependencies |
| `zoom-ffi` | `zoom/ffi/` | The `extern "C"` surface, `LzStatus`, `LzBoundary`, the one runtime handle, the logger that forwards to the shim, the real monotonic clock. cbindgen header. The ONLY crate allowed `unsafe` |
| `zoom-bench` | `zoom/bench/` | Criterion benches for the per-frame hot paths (FOV and turn-scale queries) |
| shim | `shim/` | C++: LeviLamina hooks and event forwarding only. It decides nothing |

### 9.1 Shim responsibilities (C++, thin)

- Install and remove hooks and listeners in `enable()` / `disable()`, so `/clill reactivate` works.
- Forward `ll::event::KeyInputEvent` (key code, down/up) to core. Never cancel it.
- Forward gameplay-gate inputs (mouse grabbed, current screen is HUD or not) to core.
- Hook `LevelRendererPlayer::getFov(float, bool)`: call the original for both base values, pass them to core, return what core returns.
- Hook `LocalPlayer::_applyTurnDelta(Vec2 const&)`: ask core for the scale, call the original with the scaled delta. This path never sees wheel events. Fallback if the hook fails to resolve or fire: scale `dx/dy` in `ll::event::MouseInputEvent`, skip `ActionWheel` completely, keep the sub-count remainder (in core).
- Forward lifecycle events: focus lost (`MinecraftGame::$onAppFocusLost`), dimension change, `ClientExitLevelEvent`.
- Pass the config directory path to core at startup.

All hook names come from LeviLamina 26.51.7 public headers; the builder confirms them against the headers.

### 9.2 Hot paths and performance

- `getFov` runs at least once per frame and may run more than once. The FOV query must not allocate, must give the same answer for the same time, and must be cheap. It gets a Criterion bench.
- The turn-delta query runs per mouse move and gets the same treatment.
- State shared between input, FOV and turn hooks follows CLAUDE.md sections 16 and 17 (atomics with the documented ordering; no blocking on the game thread).
- Memory is fixed-size; there is no collection that grows with play time.

## 10. mise tasks

Match `../levi-freecam`: `preflight`, `check`, `test`, `header`, `header-check`, `build-ffi`, `build-mod`, `install-mod`, plus `bench`. `build-mod` produces `bin/alaydriem-zoom/`; `install-mod` copies it into the LeviLauncher game's `mods/` folder.

## 11. Testing

- **Unit / integration tests** (`cargo nextest`, `tests/` mirrors `src/`, public API only) for our logic:
  - ease state machine: press, release, release mid-ease, re-press mid-ease (no jump), key repeat ignored, gate closed blocks zoom start, resets (snap vs ease)
  - frame-rate independence: the same time gives the same FOV no matter how many queries came before
  - curves: start and end values, monotonic, continuous
  - FOV math including the variable-FOV blend
  - sensitivity scale math and the fallback remainder carry
  - config: defaults when missing, clamping of bad values, unknown easing name falls back
  - FFI contract: null handle returns an error status; a panic does not cross the boundary
- **Benches:** FOV query and turn-scale query, with numbers recorded.
- **End-to-end:** the sibling `../levi-minimap` has a C++ e2e harness (`e2e/`). This mod gets a small one in the same style, which links the built FFI library through the generated header and drives scenarios (press, many frames, release, focus loss) from C++, to prove the boundary works as the shim will use it. It runs at the end of the full build.
- **In-game checks (run by the user, who has 26.51.1 + LeviLamina installed):**
  1. First checkpoint, early in the build: the mod loads, and `getFov` and `_applyTurnDelta` resolve and fire (log lines). If `_applyTurnDelta` fails, switch to the `MouseInputEvent` fallback.
  2. Zoom in and out feels smooth at low and high FPS.
  3. Direction change mid-ease has no jump.
  4. Mouse slows to match the zoom.
  5. Scroll wheel unchanged (hotbar scroll works zoomed and not zoomed).
  6. Alt+Tab while zoomed does not stick; chat or inventory while zoomed resets.
  7. Sprint while zoomed does not wobble.
  8. Config edit + reload works without a restart.
  9. The user tunes the default curve and durations; the builder updates the defaults.

## 12. Licensing and sharing

- If the mod is shared, add LeviLamina's "Not an official LeviMC product" disclaimer and follow LeviLamina's usage guidelines.
