# Mod Manager Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Change `alaydriem-zoom` into a client of the Alaydriem Mod Manager (`alaydriem-mod-manager`). The manager owns the zoom key (a chord), the settings (`config.hcl`, drawn in the manager panel with Studio Dark and Nordwand Mono) and the log. The zoom maths and the camera hooks stay in this repo.

**Architecture:** `zoom-ffi` owns one `amm_client::ManagerLink`. It is created in `enable()` and dropped in `disable()`. A new game-tick hook calls `lz_tick`, which reads the link's events and gives them to `ZoomController::apply` in `zoom-core`. `zoom-core` decides everything: which event zooms, suspends or reloads, and when the `Camera` lease is needed. `zoom-ffi` only moves data. The FOV query and the turn-scale query stay as they are and never touch the link.

**Tech Stack:** Rust 1.98 (edition 2024), `amm-client` (with `default-features = false`, so no egui and no studio-ui) and `amm-core` as path dependencies on `../levi-crates/crates/...`, `curia =0.2.2`, `hcl-rs =0.19.8`, and a C++20 shim on LeviLamina 26.51.6 built with xmake.

**Spec:**
- `C:\Users\charl\projects\levi-crates\docs\superpowers\plans\2026-10-20-mod-integrations.md`, sections 2 and 3: the integration contract.
- `C:\Users\charl\projects\levi-crates\docs\superpowers\specs\2026-10-20-alaydriem-mod-manager-design.md`
- **The reference integration:** `C:\Users\charl\projects\levi-levels\docs\superpowers\plans\2026-10-20-mod-manager-integration.md`. Light Level is a mod with no frame callback, as zoom is. Where this plan says "as levels does", follow that plan.
- The manager API facts: `docs/findings/amm-integration/manager-survey.md`. Read it before you start.
  - The spec names `ManagerLink::connect` and `with_table`, but the code does not have them.
  - Since the survey was written, levi-crates added `ManagerLink::without_frame(table, descriptor_hcl, config_dir)` and `amm_client::test_support::FakeTable` behind the `test-support` feature. This plan uses both.

## Global Constraints

- **Never commit.** Each "Checkpoint" step runs the tests. The operator reviews the diff.
- Build and test only through `mise run <task>`. Never pipe a build, test or bench command into `grep`, `head`, `tail` or `awk`.
- Use path dependencies only. Declare them once, in `[workspace.dependencies]` in the root `Cargo.toml`, so that the operator changes one place for the git tag:
  - `amm-client = { path = "../levi-crates/crates/amm-client", default-features = false }`
  - `amm-core = { path = "../levi-crates/crates/amm-core" }`. Do not turn on its `egui` feature.
  - `zoom-ffi`'s dev-dependencies add `amm-client` with `features = ["test-support"]`.
- Do not change anything in `levi-crates`. If you need a manager change, write it in `docs/findings/amm-integration/manager-requests.md` and stop on that item.
- The CI workflows do not change in this plan. The Rust CI jobs fail until a levi-crates tag exists. This is accepted, as for levels.
- `shim/manifest.json` gets `"dependencies": [{ "name": "alaydriem-mod-manager", "version": ">=0.1.0 <0.2.0" }]`.
- In `tooth.json`, add `"github.com/alaydriem/levi-crates#client": ">=0.1.0 <0.2.0"` to `variants[0].dependencies`, next to the LeviLamina entry. The manager's tooth is `github.com/alaydriem/levi-crates` (see `levi-crates/shim/tooth.json`), and its only variant has the label `client`.
- Descriptor `id = "alaydriem-zoom"`. It is the same as the LeviLamina mod name.
- There is one claim: `claim "Camera" { mode = "Shared" }`. There are no Exclusive claims. Do not claim `Input`: holding Input would take every key from the game.
- There is one keybind: `keybind "zoom"`, default chord `"Grave"` (VK 192, the current key), `capability = "Camera"`, `scope = "Global"`, `config_path = "keys.zoom"`.
- There is no config migration. The old `config.json` is not read. The first start writes `config.hcl` from the descriptor defaults.
- `ManagerLink::new` is called only from `enable()`, never from `load()`, because `load()` runs under the OS loader lock.
- Logging: use the `curia` macros (`curia::info!("text", { key: value })`, with no format arguments) in all Rust code. The shim makes one LeviLamina logger call, the error line when `lz_attach` fails. Remove `log`, `LogBridge` and `LzLogFn`.
- **No egui in zoom**, the same as levels. The settings UI is the manager panel. The panel is built from `descriptor.hcl` and drawn in Studio Dark with Nordwand Mono. Zoom registers with `ManagerLink::without_frame`, and it does not depend on `egui` or `studio-ui`. So the DLL does not embed the font, and the package does not need `OFL.txt`.
- Tests use `amm_client::test_support::FakeTable` (`table()`, `push_event`, `set_config`, `set_acquire_outcome`, and `with(|state| ..)` for `acquired`, `released` and `unregistered`). Do not copy the fake.
- The FOV query and the turn-scale query stay free of allocation, and they never lock the link.
- Lock order in `LzHandle`: link, then controller. The hot paths take the controller lock only.
- Write findings under `docs/findings/amm-integration/`. Every report ends with `Task Completed`.

## Operator Decisions Needed (defaults used by this plan)

1. **Turn-delta scaling.** The manager has no API that scales the turn delta, and spec:308 says "Mods no longer hook the turn themselves". *Default:* keep `TurnDeltaHook` on `LocalPlayer::_applyTurnDelta`, and write manager request R1 ("turn scale factor for a Shared Camera holder"). The two hooks chain, and neither order is unsafe (survey section 9).
2. **The zoom key in chat.** A matched keybind is always consumed from the game. As far as the survey shows, the manager does not know when a text screen is open. So with the default chord `Grave`, the backtick character may be lost in chat. *Default:* the core still ignores a press outside gameplay. Write manager request R2 ("do not route Global binds while a text input screen is open"). The operator checks this in the game.
3. **A zoom indicator on screen.** *Default:* none. If you want one, it is a separate plan: a `Hud` Shared claim, `ManagerLink::new` with a `FrameHandler`, and `studio-ui`.
4. **The `#client` suffix in `tooth.json`.** It copies the form of the LeviLamina entry. Confirm in Task 4 that `lip` resolves `github.com/alaydriem/levi-crates#client`. If it does not, use the key without the suffix and write down which form works.

## Review Focus

1. **The zoom key while chat or a menu is open:** the zoom does not start. A press with `in_gameplay = false` is ignored (Task 2 test `press_outside_gameplay_is_ignored`). The key may still be consumed (decision 2).
2. **A setting changes in the panel while zoomed:** the view returns to normal and no hold stays stuck. The next press uses the new factor (Task 3 test `settings_changed_reloads_and_resets`).
3. **The manager is missing or has the wrong ABI:** `lz_attach` returns `LZ_STATUS_UNAVAILABLE`, the shim writes one error line and installs no hooks, and the game FOV does not change (Task 3 test `attach_without_manager_is_unavailable`, and the e2e scenario in Task 4).
4. **Alt-Tab while the key is held:** the manager sends `Released`, and `FocusLostHook` snaps the view to normal. No zoom stays stuck (Task 3 test `focus_lost_after_release_stays_normal`).
5. **Disable and enable while zoomed:** `lz_detach` snaps the view to normal and drops the link, which releases `Camera`. A new attach zooms again (Task 3 test `detach_then_attach_zooms_again`).

---

### Task 0: Before numbers

**Files:**
- Create: `docs/findings/amm-integration/before.md`

- [ ] **Step 1:** Run `mise run bench`. Record the low, median and high values for `fov_query_mid_ease`, `fov_query_settled` and `turn_scale_mid_ease`. Use the table format of `docs/findings/benches/baseline.md`.
- [ ] **Step 2:** Run `mise run build-mod`. Record the size in bytes of `shim/bin/alaydriem-zoom/alaydriem-zoom.dll`.
- [ ] **Step 3:** Record the machine, the date and the commit (`git rev-parse HEAD`). End the file with `Task Completed`.

### Task 1: Settings model and descriptor

Replace the JSON config with HCL settings that the manager reads and checks.

**Files:**
- Modify: `Cargo.toml` (workspace deps).
  - Add `amm-client` and `amm-core`, as given in Global Constraints. Add `curia = "=0.2.2"` and `hcl-rs = "=0.19.8"`.
  - Remove `log` and `serde_json`.
- Modify: `zoom/core/Cargo.toml`. Add `serde` and `curia`. Add the dev dependency `hcl-rs`. Remove `log`, `serde_json` and `tempfile`.
- Modify: `zoom/core/src/config/zoom.rs`
- Create: `zoom/core/src/config/settings.rs` (`ZoomSettings`)
- Delete: `zoom/core/src/config/loader.rs`, `zoom/core/src/errors/config.rs`, `zoom/core/tests/config/loader.rs`. If `errors/` is empty, also remove the `errors` module.
- Create: `zoom/ffi/descriptor.hcl`
- Test: `zoom/core/tests/config/zoom.rs`, `zoom/core/tests/config/settings.rs`, `zoom/ffi/tests/descriptor.rs`

**Interfaces:**
- Produces:
  - `#[derive(Deserialize, Default)] #[serde(default)] pub struct ZoomSettings { pub zoom: ZoomConfig }`. Keys that the struct does not name, such as `keys.zoom`, are ignored.
  - `ZoomSettings::into_config(self) -> ZoomConfig`. It returns `self.zoom.sanitized()`.
  - `ZoomConfig` keeps `zoom_factor`, `zoom_in_ms`, `zoom_out_ms`, `easing: String`, `sensitivity_match_strength` and `fade_variable_fov_while_zoomed`. It drops `version`, `key_code`, `from_json` and `Serialize`. It has `#[derive(Deserialize)] #[serde(default)]` with snake_case names.
  - `sanitized()`, `curve()` and the `MIN_`/`MAX_` constants stay. Their `warn!` calls become `curia::warn!` with fields `{ field, value, used }`.
  - `ZoomFfi::DESCRIPTOR: &'static str = include_str!("../descriptor.hcl")`. This is `pub const` on a struct in `zoom/ffi/src/services/descriptor.rs`.

The descriptor:
- Top level: `id = "alaydriem-zoom"`, `display_name = "Zoom"`, `version = "0.1.0"`, `author = "Alaydriem"`, `config_path = "config.hcl"`.
- `claim "Camera"`: Shared, `reason = "Changes the field of view and look speed while the zoom key is held"`.
- The keybind as given in Global Constraints.
- `section "zoom" { title = "Zoom" }`, with these fields. All of them are `apply = "Live"`.

| Path | Kind | Default |
| --- | --- | --- |
| `zoom.zoom_factor` | `float { min = 1.0, max = 32.0, step = 0.5 }` | 4.0 |
| `zoom.zoom_in_ms` | `int { min = 0, max = 2000, step = 10 }` | 250 |
| `zoom.zoom_out_ms` | `int { min = 0, max = 2000, step = 10 }` | 250 |
| `zoom.easing` | `enum`: `linear`, `easeOutSine`, `easeOutCubic`, `easeOutQuint`, `easeOutExpo`, `easeInOutCubic`. Labels in plain words | `"easeInOutCubic"` |
| `zoom.sensitivity_match_strength` | `float { min = 0.0, max = 1.0, step = 0.05 }` | 1.0 |
| `zoom.fade_variable_fov_while_zoomed` | `bool` | true |

- `section "keys" { title = "Keys" }`, with `field "keys.zoom" { kind = "chord", default = "Grave", apply = "Live" }`.
- Each field has a `help` line. Copy the meaning from the old comments in `zoom.rs`.

- [ ] **Step 1: Write the failing tests**
  - `zoom/core/tests/config/settings.rs`:
    - `empty_document_gives_defaults`: `hcl::from_str::<ZoomSettings>("")` gives `.into_config() == ZoomConfig::default()`.
    - `document_values_are_read`: the text `zoom {\n zoom_factor = 8.0\n easing = "linear"\n}\nkeys {\n zoom = "Ctrl+Z"\n}` gives `zoom_factor == 8.0`, `curve() == EasingCurve::Linear`, and default values for the other fields.
    - `out_of_range_value_is_clamped`: `zoom_factor = 99.0` gives `32.0`.
  - Keep the clamp tests in `tests/config/zoom.rs`. Delete the `from_json` tests.
  - `zoom/ffi/tests/descriptor.rs`. This file is declared from `tests/integration.rs`. Add `amm-core` and `hcl-rs` to the `zoom-ffi` dev-dependencies. The tests follow the levels plan, Task 5.
    - `descriptor_passes_the_manager_checker`:
      - `DescriptorChecker::new(128, 256).parse(ZoomFfi::DESCRIPTOR)` is `Ok`.
      - The id is `alaydriem-zoom`.
      - The claims are exactly `[(Camera, Shared)]`.
      - The keybinds are exactly `[("zoom", Camera, Global, Chord { key: 0xC0, ctrl: false, shift: false, alt: false })]`. 0xC0 is VK 192, so this proves that `Grave` is the current key.
    - `schema_defaults_match_core_defaults`:
      - Run the parsed sections through `SchemaChecker::check` with an empty document.
      - Read the checked text with `hcl::from_str::<ZoomSettings>`.
      - Assert that `.into_config()` equals `ZoomConfig::default()`.
- [ ] **Step 2:** Run `mise run test`. Expected: compile FAIL (`ZoomSettings` and `ZoomFfi` are not defined).
- [ ] **Step 3:** Implement the files above. Change `ZoomController::new` to read only the fields that are left. The `key_code` use moves out in Task 2. Change the controller and the bench to use `curia` instead of `log`.
- [ ] **Step 4: Checkpoint.** Run `mise run test` and `mise run check`. Expected: all PASS. The FFI tests that still use `lz_key` and the config path compile until Task 3. Gate or rename them only if the compiler forces it.

### Task 2: Controller takes manager events

**Files:**
- Modify: `zoom/core/Cargo.toml` (add `amm-core`)
- Modify: `zoom/core/src/services/zoom_controller.rs`
- Create: `zoom/core/src/structs/zoom/zoom_action.rs` (`ZoomAction`)
- Create: `zoom/core/src/structs/zoom/manager_reply.rs` (`ManagerReply`)
- Modify: `zoom/core/src/structs/zoom/mod.rs`, `zoom/bench/benches/zoom.rs`
- Test: `zoom/core/tests/services/zoom_controller.rs`, `zoom/core/tests/support/controllers.rs`

**Interfaces:**
- Consumes: `amm_core::{ModEvent, KeybindPhase, Capability}` and `ZoomConfig` from Task 1.
- Produces:
  - `pub enum ZoomAction { Zoom }`, with `ZoomAction::from_index(action: u16) -> Option<ZoomAction>`. Index 0 is `Zoom`, and the order is the keybind order in `descriptor.hcl`.
  - `pub enum ManagerReply { Nothing, ReloadSettings }`
  - `ZoomController::press(&mut self, in_gameplay: bool, now_us: u64)`. It replaces `key()`. The gate is the same as before, and a press while suspended is ignored.
  - `ZoomController::release(&mut self, now_us: u64)`. This becomes public.
  - `ZoomController::suspend(&mut self, now_us: u64)`: it drops the hold, snaps the level to 0 and sets `suspended`.
  - `ZoomController::resume(&mut self)`: it clears `suspended`.
  - `ZoomController::reconfigure(&mut self, config: ZoomConfig)`: it replaces the config, the curve and the durations, drops the hold, snaps the level to 0 and keeps `suspended`.
  - `ZoomController::wants_camera(&self, now_us: u64) -> bool`: true while held or while `level > 0`.
  - `ZoomController::apply(&mut self, event: &ModEvent, in_gameplay: bool, now_us: u64) -> ManagerReply`. It maps events as follows:
    - `Keybind{Zoom, Pressed}` → `press`
    - `Keybind{Zoom, Released}` → `release`
    - `Repeat` → nothing
    - `Suspended{Camera, ..}`, `Revoked{Camera, ..}` and `Disabled` → `suspend`
    - `Resumed(Camera)`, `CapabilityAvailable(Camera)` and `Enabled` → `resume`
    - `SettingsChanged{..}` → `ReloadSettings`
    - any other event → `Nothing`

- [ ] **Step 1: Write the failing tests** in `tests/services/zoom_controller.rs`. Change `support::controllers` to use `press(true, 0)`.
  - `pressed_keybind_zooms`: apply `Keybind{action:0, Pressed}` with `in_gameplay = true` at 0. Then `level(FULL) == 1.0`.
  - `released_keybind_eases_out`: press, then apply Released at `FULL`. Then `level(2*FULL) == 0.0`.
  - `press_outside_gameplay_is_ignored`: apply Pressed with `in_gameplay = false`. Then `level(FULL) == 0.0` and `!wants_camera(FULL)`.
  - `unknown_action_is_ignored`: `Keybind{action: 1, Pressed}` returns `Nothing` and the level stays at 0.
  - `repeat_does_not_restart_ease`: press at 0, then apply Repeat at `FULL/2`. The level at `FULL/2` is the same as without the Repeat.
  - `suspend_snaps_and_blocks_press`: zoom fully, then apply `Suspended{Camera, by}`. Then `level == 0.0`. A later Pressed is ignored. After `Resumed(Camera)`, a Pressed zooms.
  - `suspend_of_other_capability_is_ignored`: `Suspended{Hud, ..}` leaves a full zoom at 1.0.
  - `settings_changed_asks_for_reload`: `apply(SettingsChanged{paths:1})` returns `ReloadSettings`.
  - `reconfigure_resets_and_keeps_suspension`: suspend, then reconfigure. The press is still ignored. Without a suspension, reconfigure snaps the level to 0 and the next press uses the new `zoom_factor`.
  - `wants_camera_until_eased_out`: press at 0, release at `FULL`. `wants_camera(FULL + 1)` is true and `wants_camera(2*FULL + 1)` is false.
  - Move the existing `key()` tests to `press`/`release`. Delete the tests for wrong key codes, because the manager now matches the chord.
- [ ] **Step 2:** Run `mise run test`. Expected: compile FAIL on `apply`, `press` and `ZoomAction`.
- [ ] **Step 3:** Implement the interfaces. `apply` is a single `match` on the event. It calls the methods above and does no work of its own.
- [ ] **Step 4:** Change the benches to use `controller.press(true, 0)`. Add the bench `apply_keybind_pressed`: `apply(&PRESSED, true, now)` on a settled controller.
- [ ] **Step 5: Checkpoint.** Run `mise run test` and `mise run check`. Expected: PASS.

### Task 3: FFI owns the link; new C ABI

**Files:**
- Modify: `zoom/ffi/Cargo.toml`. Add the deps `amm-client`, `amm-core`, `curia` and `hcl-rs`. Add the dev dependency `amm-client` with `features = ["test-support"]`. Remove `log` and `tempfile`.
- Create: `zoom/ffi/src/services/zoom_link.rs` (`ZoomLink`)
- Modify: `zoom/ffi/src/services/handle.rs`, `zoom/ffi/src/services/boundary.rs` (`log::error!` becomes `curia::error!`), `zoom/ffi/src/services/mod.rs`
- Modify: `zoom/ffi/src/api/lifecycle.rs`, `zoom/ffi/src/api/input.rs`, `zoom/ffi/src/types/status.rs`
- Delete: `zoom/ffi/src/services/log_bridge.rs`, `zoom/ffi/src/types/log_fn.rs`
- Test: `zoom/ffi/tests/services/handle.rs`, `zoom/ffi/tests/services/zoom_link.rs`, `zoom/ffi/tests/api/lifecycle.rs`, `zoom/ffi/tests/api/input.rs`, `zoom/ffi/tests/api/view.rs`, `zoom/ffi/tests/support/session.rs`

**Interfaces:**
- Consumes: `ZoomController::{apply, reconfigure, suspend, resume, wants_camera}`, `ManagerReply`, `ZoomSettings::into_config` and `ZoomFfi::DESCRIPTOR`.
- Produces (Rust):
  - `ZoomLink::new(table: ApiTable, config_dir: &Path) -> Result<ZoomLink, LinkError>`. It wraps `ManagerLink::without_frame(table, ZoomFfi::DESCRIPTOR, config_dir)` and a `holding_camera: bool`.
  - `ZoomLink::settings(&self) -> ZoomConfig`. It returns `link.config::<ZoomSettings>()` through `into_config`. On an error it logs once and returns `ZoomConfig::default()`.
  - `ZoomLink::drain(&mut self, out: &mut impl FnMut(&ModEvent))`. It calls `events()` until a call gives no event, at most `ZoomLink::MAX_DRAINS_PER_TICK = 4` times (256 events). Events after that wait for the next tick. It does not allocate.
  - `ZoomLink::sync_camera(&mut self, wants: bool) -> bool`:
    - If `wants` is true and the mod does not hold Camera, it calls `acquire(Capability::Camera)`. `Granted` or `Preempted` sets `holding_camera`.
    - If `wants` is false and the mod holds Camera, it calls `release`.
    - It returns `false` only on `Denied` or `Err`.
  - `LzHandle::new() -> LzHandle`. It starts with the default config and no link.
  - `LzHandle::attach(&self, table: ApiTable, config_dir: &Path) -> Result<(), LinkError>`:
    - It builds the `ZoomLink`.
    - It calls `controller.reconfigure(link.settings())` and then `resume()`.
    - It logs `curia::info!("alaydriem-zoom attached")`.
  - `LzHandle::detach(&self)`. It calls `controller.suspend(now)`, then drops the link. The drop releases the lease and unregisters.
  - `LzHandle::tick(&self, in_gameplay: bool) -> Option<()>`:
    - It locks the link. With no link it returns `Some(())`.
    - It drains the events into `controller.apply`.
    - On `ReloadSettings` it calls `controller.reconfigure(link.settings())`.
    - It then calls `sync_camera(controller.wants_camera(now))`. When that returns `false`, it calls `controller.suspend(now)`.
    - It returns `None` only if a lock is poisoned.
  - `LzHandle` has the fields `controller: Mutex<ZoomController>`, `link: Mutex<Option<ZoomLink>>` and `clock`. It no longer has `config_file`.
  - In tests, the `Session` helper builds `LzHandle::new()` and calls `attach(FakeTable::table(), dir)`. It zooms by `FakeTable::push_event(ModEvent::Keybind { action: 0, phase: KeybindPhase::Pressed })` and then a tick.
  - `lz_attach` keeps the levels split between create and register: the handle is created in `load()` and the link in `enable()`. Unlike levels, zoom never destroys the handle in `disable()`, because a render-thread call to `lz_fov` may still be inside a hook while it is removed. `lz_detach` drops only the link.
- Produces (C ABI, through cbindgen):
  - `LzHandle* lz_create(void)`
  - `LzStatus lz_attach(const LzHandle*, const LzStr* config_dir)`. It calls `ApiTable::resolve()` and then `attach`. `ManagerMissing` and `AbiMismatch` give the new `LZ_STATUS_UNAVAILABLE`. Any other error gives `LZ_STATUS_FAILED`.
  - `LzStatus lz_detach(const LzHandle*)`
  - `LzStatus lz_tick(const LzHandle*, uint8_t in_gameplay)`
  - These stay: `lz_event`, `lz_fov`, `lz_turn_scale` and `lz_destroy`.
  - These go: `lz_key` and `lz_reload`.

- [ ] **Step 1: Write the failing tests**
  - `tests/services/handle.rs`, with `FakeTable`. The fake state is per thread, and nextest runs each test in its own process.
    - `pressed_keybind_zooms_after_tick`: push `Keybind{0, Pressed}`, then `tick(true)`. After the ease, `with_controller(fov)` gives `70 → 17.5`.
    - `settings_changed_reloads_and_resets`: the fake config is `zoom {\n zoom_factor = 2.0\n}`. Push `SettingsChanged` and tick. The view is normal, and after the next press the fov is 35.0.
    - `camera_is_acquired_on_press_and_released_after_ease`: `state.acquired == [Camera]` after the press tick. `state.released == [Camera]` after the tick that follows the full ease out.
    - `denied_camera_does_not_zoom`: `FakeTable::set_acquire_outcome(Capability::Camera, AcquireOutcome::Denied { holder })`. After the press tick, the fov is unchanged.
    - `register_has_no_frame`: after attach, `state.registered_without_frame()` is true.
    - `suspended_camera_snaps_back`: zoom fully, push `Suspended{Camera}` and tick. The fov is 70.0.
    - `focus_lost_after_release_stays_normal`: zoom fully, push Released and tick, then call `lz_event(FocusLost)`. The fov is 70.0 at once and stays 70.0.
    - `detach_then_attach_zooms_again`: zoom, then `detach`. The fov is 70.0 and the fake records unregister. Then attach again and press: it zooms.
    - `more_than_256_events_are_kept_for_the_next_tick`: push 300 `Repeat` events and one `Pressed` at the end, then tick twice. The second tick zooms.
    - `tick_without_link_is_ok`: on `LzHandle::new()`, `tick(true) == Some(())`.
  - `tests/services/zoom_link.rs`:
    - `config_error_falls_back_to_defaults`: `FakeTable::set_config("zoom {")`. `settings()` equals `ZoomConfig::default()`.
    - `sync_camera_does_not_acquire_twice`: `sync_camera(true)` twice gives `state.acquired.len() == 1`.
  - `tests/api/lifecycle.rs`:
    - `attach_without_manager_is_unavailable`: in a test process the manager DLL is not loaded, so `lz_attach(lz_create(), dir)` gives `LzStatus::Unavailable`. After that call, `lz_fov(h, 70, 70, 1) == 70.0` and `lz_tick(h, 1) == Ok`.
    - The null-handle contract: `lz_attach`, `lz_detach` and `lz_tick` with a null handle give `InvalidArgument`.
    - A null `config_dir` gives `InvalidArgument`.
  - Change `tests/api/input.rs` and `tests/api/view.rs` from `lz_key` to `Session` (FakeTable) plus a pushed keybind and `lz_tick`. Delete the tests for `lz_reload` and for the config file.
- [ ] **Step 2:** Run `mise run test`. Expected: compile FAIL.
- [ ] **Step 3:** Implement the services, then the entry points. Each entry point is a thin delegator in `LzBoundary::guard`, with a `// SAFETY:` comment on every unsafe block. `ManagerLink` is `Send` (`unsafe impl Send` in `manager_link.rs`), so `Mutex<Option<ZoomLink>>` keeps `LzHandle` `Sync`.
- [ ] **Step 4:** Run `mise run header`. Check that `shim/include/levi_zoom.h` no longer has `lz_key`, `lz_reload` or `LzLogFn`.
- [ ] **Step 5: Checkpoint.** Run `mise run test`, `mise run check` and `mise run header-check`. Expected: PASS.

### Task 4: Shim, package and e2e

**Files:**
- Modify: `shim/src/mod/ZoomMod.cpp`, `ZoomMod.h`
- Modify: `shim/src/bridge/RustBridge.cpp`, `RustBridge.h`. Remove `logFromRust` and the config path. Add `attach(const std::filesystem::path& configDir) -> LzStatus`, `detach()` and `tick(bool inGameplay)`.
- Create: `shim/src/hooks/TickHook.h`. It hooks `ClientInstance::$update` after `origin()` and calls `RustBridge::tick(ClientAccess::inGameplay())`. Copy the hook shape from `levi-crates/examples/demo-mod/shim/src/hooks/ClientUpdateHook.h`.
- Rename: `shim/src/input/InputListeners.*` to `shim/src/level/LevelListeners.*`. Keep only the `ClientExitLevelEvent` listener, and delete the `KeyInputEvent` listener.
- Modify: `shim/manifest.json` (the dependency) and `tooth.json` (the dependency, see Global Constraints).
- Modify: `shim/xmake.lua`. Keep the current `add_syslinks`. If the link fails on symbols from `amm-client`'s Rust dependencies, add only the system libraries the error names. The manager's own extra list is `bcrypt`, `advapi32`, `ole32` and `shell32`. The shim calls no manager API from C++, so it does not need the `alaydriem_mod_manager.h` include. Levels needs that include only for its `ModLog` port.
- No `ModLog` port. The shim's only log line is the "manager missing" error, and that is the one permitted LeviLamina logger call. If a later change adds C++ log lines, port the demo's `ModLog` through a new `lz_log`, as levels does.
- Modify: `e2e/src/main.cpp`
- Modify: `mise.toml`. Add a `preflight` step that fails with a clear message when `../levi-crates/crates/amm-client/Cargo.toml` is missing.

**Lifecycle:**
- `load()` calls `lz_create()`. A null handle returns false.
- `enable()` calls `RustBridge::attach(getSelf().getConfigDir())`.
  - If the status is not `Ok`, it logs one LeviLamina error, `"alaydriem-zoom needs alaydriem-mod-manager 0.1.x; zoom is off"`, installs no hooks and returns true.
  - If the status is `Ok`, it installs `FovHook`, `TurnDeltaHook`, `FocusLostHook`, `DimensionHook` and `TickHook`, then registers `LevelListeners`.
- `disable()` unregisters the listeners, removes every hook, then calls `RustBridge::detach()`. The handle is never destroyed, as today.
- Remove the `"alaydriem-zoom loaded"` info line. Rust logs the attach.

- [ ] **Step 1:** Rewrite `e2e/src/main.cpp`. The new scenarios drive the ABI the way the shim does, with no manager DLL loaded:
  - `lz_create` gives a non-null handle.
  - `lz_attach` gives `LZ_STATUS_UNAVAILABLE`.
  - `lz_tick(h, 1)` gives `LZ_STATUS_OK`.
  - `lz_fov(h, 70, 70, 1) == 70`.
  - `lz_turn_scale(h) == 1`.
  - `lz_event(h, LZ_EVENT_FOCUS_LOST)` gives `LZ_STATUS_OK`.
  - `lz_detach(h)` gives `LZ_STATUS_OK`.
  - `lz_destroy(h)`.
  - Null-handle checks for every entry point.
  - The key-driven scenarios move to the Rust `FakeTable` tests in Task 3. The descriptor test in Task 1 proves that `Grave` is VK 0xC0.
- [ ] **Step 2:** Run `mise run e2e`. Expected: all PASS.
- [ ] **Step 3:** Implement the shim changes. Run `mise run build-mod`. Expected: the build succeeds, and `shim/bin/alaydriem-zoom/` contains the DLL and `manifest.json` with the dependency.
- [ ] **Step 4:** Check the imports of the built DLL with `dumpbin /imports`. Write the output to `docs/findings/amm-integration/imports.txt`. Confirm that it has no `GetAsyncKeyState` and no `d3d11` or `dxgi` imports.
- [ ] **Step 5:** Run `lip` against `tooth.json` to confirm that it parses and resolves the manager dependency (decision 4). Write the command and its result in `imports.txt`.
- [ ] **Step 6: Checkpoint.** Run `mise run check`, `mise run test`, `mise run header-check` and `mise run e2e`. Expected: PASS.

### Task 5: Repo rules, docs and after numbers

**Files:**
- Modify: `CLAUDE.md`. Show the operator these edits.
  - Section 11: logging is `curia` through the manager link. The shim makes one LeviLamina error call, only when there is no link.
  - Section 15: config is `config.hcl`, owned by the manager and described in `zoom/ffi/descriptor.hcl`. There is no JSON.
  - Section 4: the shim MAY tick the Rust core from `ClientInstance::$update`. It no longer forwards keys.
  - Project Shape and Quick Reference: add the manager dependency and the descriptor.
  - Section 3: `amm-client` contains `unsafe`, but it is a dependency. The rule for our crates does not change.
- Modify: `README.md`. Cover the install dependency on the mod manager, rebinding the zoom key in the manager panel (Keybinds page), and the new `config.hcl` keys. Say that the old `config.json` is not read.
- Create: `docs/findings/amm-integration/after.md`
- Create: `docs/findings/amm-integration/manager-requests.md`, with R1 and R2 from Operator Decisions and anything new.

- [ ] **Step 1:** Run `mise run bench`. Write `after.md` with the before and after tables for the three benches, the new `apply_keybind_pressed` bench, and the DLL size before and after. Name any change of more than 5 % on a hot-path bench and explain it. End the file with `Task Completed`.
- [ ] **Step 2:** Write the docs and the requests file. End each findings file with `Task Completed`.
- [ ] **Step 3: Final checkpoint.** Run `mise run check`, `mise run test`, `mise run header-check` and `mise run e2e`. Expected: all PASS. Send `after.md` and `manager-requests.md` to the operator.

### Operator acceptance in the game (after Task 5)

Install the manager and zoom, and use `mise run install-mod`. Then check each item:

- [ ] Zoom shows in the panel. Its settings are drawn in Studio Dark with Nordwand Mono. A changed zoom factor saves, and the next zoom uses it.
- [ ] `Grave` held zooms and released eases out. The Keybinds page shows "Zoom". Rebinding to a chord (for example `Ctrl+Z`) works, and a bare `Z` does not zoom.
- [ ] Alt-Tab while zoomed returns to the normal view. Disable and enable in LeviLamina work.
- [ ] Typing a backtick in chat: is the character lost? Record the result for R2.
- [ ] With demo-b's Exclusive Camera, the zoom key shows a toast and does not zoom. After release, the zoom works again.
- [ ] Without the manager installed: one error line, the game runs, no zoom.
- [ ] No Conflicts entry that the operator did not expect. The log is in `<game>/logs/amm/alaydriem-zoom/`.
