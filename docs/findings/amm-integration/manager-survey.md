# levi-crates (Alaydriem Mod Manager) survey for the levi-zoom integration

**Update (2026-10-20, later):** some facts below are out of date. levi-crates now has:

- `ManagerLink::without_frame(table, descriptor_hcl, config_dir)`. It registers with no frame callback and needs no `IdleUi`.
- An `egui` feature on `amm-client`, on by default. With `default-features = false` the mod links no egui and no studio-ui, so it does not need `OFL.txt`.
- `amm_client::test_support::FakeTable` behind the `test-support` feature. Do not copy the fake.

The manager's tooth is `github.com/alaydriem/levi-crates` (`levi-crates/shim/tooth.json`). The reference integration is `levi-levels/docs/superpowers/plans/2026-10-20-mod-manager-integration.md`.

Date: 2026-10-20. Source: a read-only Explore subagent over `C:\Users\charl\projects\levi-crates`. The lead saved it to disk because the subagent had no write tool. Paths are relative to `levi-crates`. Line numbers are from the subagent; check them before use.

## 1. Workspace and toolchain

- `rust-toolchain.toml`: `1.98.0`, target `x86_64-pc-windows-msvc`. Edition 2024. This agrees with levi-zoom.
- Pinned versions: egui `=0.37.0` (default-features off, `default_fonts`), hcl-rs `=0.19.8` (imported as `hcl`), hcl-edit `=0.9.7`, curia `=0.2.2`, serde 1.0.229, serde_json 1.0.154, thiserror 2.0.21, windows-sys 0.61.2, crossbeam-channel 0.5.17.
- Crates that a client mod uses:
  - `amm-client`: the link.
  - `amm-core`: types such as `Capability`, `ModEvent`, `KeybindPhase`, `AcquireOutcome`, `LinkError` and `CursorShape`. egui is only behind the `egui` feature.
  - `studio-ui`: the theme and the widgets.
  - `amm-abi`: indirectly.
- Crates for the manager only: `amm-store`, `amm-ui`, `amm-ffi` (the e2e tests link it), `amm-bench` and `amm-e2e`.

## 2. `amm_client::ManagerLink` (`crates/amm-client/src/link/manager_link.rs`)

The spec names `ManagerLink::connect` and `ManagerLink::with_table`. **Neither of them exists in the code.** The real API is below.

- `new<H: FrameHandler + Send + 'static>(table: ApiTable, descriptor_hcl: &'static str, config_dir: &Path, handler: H) -> Result<ManagerLink, LinkError>`
  - It installs Curia with a `ForwardSink` at Trace level.
  - A config dir that is not UTF-8 gives `InvalidArgument`.
- `acquire(&self, Capability) -> Result<AcquireOutcome, LinkError>`
- `release(&self, Capability)`
- `holder(&self, Capability) -> Option<String>`
- `events(&mut self) -> impl Iterator<Item = ModEvent>`
  - It gives at most 64 events per call. Call it again to get the rest.
- `config<T: DeserializeOwned>(&self) -> Result<T, LinkError>`
  - It calls `hcl::from_str` on the checked text.
- `set_setting(&self, path: &str, value: &hcl::Value) -> Result<(), LinkError>`
- `key_down(&self, vk) -> bool`
  - It answers only while the mod holds `Input`.
- `set_cursor_mode(&self, on: bool) -> Result<(), LinkError>`
- `Drop` unregisters the mod.

`ApiTable` (`link/api_table.rs`):

- `ApiTable::resolve()` calls `GetModuleHandleW("alaydriem-mod-manager.dll")`, then gets `amm_get_api` and calls `amm_get_api(AMM_ABI_MAJOR)`. The ABI is 1.1.
- `unsafe ApiTable::new(ptr: *const AmmApi)` is the injection point for tests.
- `LinkError` has these variants:
  - `ManagerMissing`
  - `AbiMismatch { found }`
  - `Status(i32)`
  - `Json { reason }`

`ModEvent` has these variants:

- `Enabled`
- `Disabled`
- `Granted(Capability)`
- `Revoked { capability, by }`
- `Suspended { capability, by }`
- `Resumed(Capability)`
- `CapabilityAvailable(Capability)`
- `Keybind { action: u16, phase: KeybindPhase }`
- `SettingsChanged { paths: u32 }`
- `WorldReloadApply`
- `Key { vk, down, repeat }`
- `PointerButton { .. }`
- `PointerDelta { dx, dy }`
- `Wheel { delta }`

Other types and facts:

- `KeybindPhase` is `Pressed`, `Released` or `Repeat`.
- `action` is the index of the keybind in the descriptor's keybind list, in file order.
- `AcquireOutcome` is `Granted`, `Denied { holder }` or `Preempted`.
- The `FrameHandler` trait has `frame(&mut self, ctx: &egui::Context, input: &FrameInputView) -> FrameReply` and an optional `textures_lost(&mut self)`.
- `ManagerLink::new` always fills the frame slot. A mod with no UI uses the demo's `IdleUi`, which returns `FrameReply { cursor: CursorShape::Default, layer_rect: Rect::NOTHING }`. The manager never calls `frame` when the mod holds no Hud or Overlay lease.

The mod's shape:

- The mod stays a LeviLamina native mod with C++ `load`, `enable` and `disable`.
- The Rust crate is a `staticlib` plus `rlib` that xmake links into the mod.
- Rust resolves the table itself. C++ never touches the table.

## 3. `descriptor.hcl`

Top-level attributes:

- Required: `id`, `display_name`, `version`, `author` and `config_path`.
- Optional: `input_passthrough = [vk...]`.

Blocks:

- `claim "<Capability>" { mode = "Shared"|"Exclusive", reason = "..." }`
  - `hide_vanilla_hud` is allowed only on Hud.
  - `block_player_turn` is allowed only on Input.
  - The capabilities are exactly `Camera`, `Input`, `Hud`, `Overlay` and `WorldDraw`.
  - There is no Fov or Zoom capability. `Camera` is arbitration only.
- `keybind "<action>" { label, default, capability, scope = "Global"|"WhileHeld", config_path }`
  - `config_path` must name a field of kind `"chord"`.
  - There is no hold or toggle mode. A hold is a `Pressed` event and then a `Released` event.
  - A matched bind is always consumed from the game.
  - A Global bind is blocked, with a toast, only by another mod's Exclusive lease on the capability.
- `section "<id>" { title, field "<dotted.path>" { label, help?, kind, default, apply = "Live"|"WorldReload"|"Restart" } }`
  - The field kinds are:
    - `"bool"`
    - `{ int = { min, max, step } }`
    - `{ float = { min, max, step, unit? } }`
    - `{ enum = { options = [{ value, label }] } }`
    - `{ color = {...} }`
    - `"chord"`

Chords (`services/settings/chord_text.rs`):

- A chord is `+`-separated. The key comes last and the modifiers (Ctrl, Shift, Alt) come before it. Case is ignored.
- The canonical form is `Ctrl+Shift+Alt+Key`.
- Key names:
  - Letters A-Z, digits 0-9 and F1-F24.
  - Space, Enter, Escape, Tab, Backspace, Delete, Insert, Home, End, PageUp, PageDown, Left, Right, Up, Down.
  - LeftBracket, RightBracket, Equals, Minus, Comma, Period, Slash, Backslash, Semicolon, Quote, Grave.
  - Any other key is written `VK_0x..`.
- Matching is exact. Shift+C does not match C.
- The release of a key goes to the mod that received its press.
- On focus loss the manager sends a key up and `Keybind Released` for every held bind.

## 4. Config

- The file is `<mod config dir>/<config_path>`. Only the manager reads and writes it.
- The manager clamps a value that is out of range.
- A missing value or a value of the wrong type takes the default.
- A file that does not parse is moved to `.invalid` and is written again from the defaults.
- A dotted path becomes a block, for example `zoom { factor = 4.0 }`.
- The mod reads its config with `link.config::<T>()`. The demo uses `#[derive(Deserialize)] #[serde(default)]` structs.
- A chord is stored as a string at the bind's path.
- There is no migration from `config.json`.

## 5. Studio UI and the font

- To install the theme: `studio_ui::StudioTheme::install(ctx, FontSource::Embedded)`.
- The font is Nordwand Mono 1.3.2 (OFL 1.1).
- `amm-client` installs the theme and the font on every layer context itself. A mod does not install them.
- The manager panel draws every mod's settings form and its Keybinds page from the descriptor schema. A mod that draws its own settings widgets is a non-goal (spec:79).
- A DLL that links studio-ui embeds the font, so the mod package must ship `crates/studio-ui/assets/fonts/OFL.txt`. The demo's xmake `after_build` step copies it.

## 6. manifest.json

The dependency is `{ "name": "alaydriem-mod-manager", "version": ">=0.1.0 <0.2.0" }`, from spec section 9.4. The demo's `examples/demo-mod/shim/manifest.json` has the same entry.

## 7. The demo mod

Lifecycle:

- `load()` records the config dir only. It runs under the OS loader lock.
- `enable()` creates the link.
  - If that fails, the mod writes one LeviLamina error line and stays inert. `enable()` returns true.
  - If it works, the mod hooks `ClientInstance::$update` and calls a Rust tick from it.
- `disable()` unhooks first, then destroys the link.

Tests:

- There is no public fake table. `amm-client` keeps one in its own tests: `crates/amm-client/tests/support/fake_table.rs` and `fake_state.rs`. A mod must copy it.
- An option that uses the real host is `amm-e2e`'s `HostFixture` (`crates/amm-e2e/tests/support/host_fixture.rs`). It calls `amm_ffi::host::amm_host_create` and `ApiTable::new(amm_api_for(AMM_ABI_MAJOR))`. It then drives the host with `fixture.key(vk, down, mods, now_ms)`, `fixture.tick(..)` and `fixture.frame(..)`.
- Descriptor test: `DescriptorChecker::new(128, 256).parse(include_str!(..))`.

## 8. Logging

- Use the `curia` macros: `curia::info!("message", { field: value })`. There are no format arguments.
- `ManagerLink::new` installs Curia. The log goes to `<game>/logs/amm/<mod id>/<stamp>.log`.
- Do not call the LeviLamina logger. The one exception is the error line when there is no link.

## 9. Input and camera ownership

- The manager owns the WndProc subclass, the LeviLamina Key and Mouse listeners, `$grabMouse` and `LocalPlayer::_applyTurnDelta`.
- `block_player_turn` is all or nothing. **There is no API that scales the turn delta.** The spec says "Mods no longer hook the turn themselves" (spec:308).
- The manager gives a mod no focus-lost event, no dimension event and no "screen open" or "mouse grabbed" state. A mod sees `WorldReloadApply` only after a WorldReload setting changes.
- There is no FOV API. A mod keeps its own camera hooks (spec:114).
- If the zoom mod and the manager both hook `_applyTurnDelta`, both hooks run in a chain. Neither order is unsafe. This is an inference; no test covers it.

## 10. Performance (spec section 10)

- The manager has Criterion budgets, for example `route_event` < 1 µs and `match_chord` < 200 ns with 64 binds.
- Each mod records numbers before and after the change. The nearest model for zoom is Light Level: the game-thread cost per tick, and no render-thread cost.

Task Completed
