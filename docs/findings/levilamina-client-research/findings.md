# LeviLamina client mod research: hold-to-zoom

Date: 2026-10-19
Scope: research only. No mod code is written here.
Goal mod: hold a key (default Ctrl) to zoom by making the camera FOV smaller, with configurable easing in and out. While zoomed, mouse sensitivity must scale down to match the zoom. The scroll wheel must not be touched.

Source tags used: LeviLamina `v26.51.7` (released 2026-10-12), main commit `4ca13569fc`.
Base link for LeviLamina files: `https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/`

How to read this document:

- "Fact" means I read it in a source file, doc, or release.
- "Guess" means I did not verify it. Test it before you rely on it.

---

## Summary (short answers)

1. **Client support: yes, official.** Added in LeviLamina 1.8.0-rc.1 (2026-01-16). The current line is 26.51.x, for Minecraft client **26.51.1** only. You install it with LeviLauncher (or `lip`). PeEditor patches `Minecraft.Windows.exe` so it loads PreLoader, and PreLoader loads LeviLamina. It is usable, but young (about 9 months). Each LeviLamina line supports exactly one client version.
2. **Template:** [LiteLDev/levilamina-mod-template](https://github.com/LiteLDev/levilamina-mod-template). Build system is xmake. Toolchain is **clang-cl (LLVM)**, not plain MSVC. You still need Visual Studio 2022 (or Build Tools) for the Windows SDK and the STL. C++20 with `_HAS_CXX23=1`. Build with `--target_type=client`. Output goes into `<game dir>/mods/<mod-name>/`.
3. **Input:** two LeviLamina client APIs: `ll::event::KeyInputEvent` (raw key down/up, Win32 virtual-key codes) and `ll::input::KeyRegistry` (a real vanilla keybinding that the player can remap). Win32 polling is possible but not needed.
4. **FOV:** hook `LevelRendererPlayer::getFov(float, bool)`. It is exported in the LeviLamina client headers. Three open-source LeviLamina zoom mods already do exactly this (Zoomidy, ZoomIntoBedrock, LaminaView). Flarial, Latite and Amethyst mods hook the same function.
5. **Sensitivity:** best option is to hook `LocalPlayer::_applyTurnDelta(Vec2 const&)` and multiply the turn by the zoom scale (LaminaView does this). Second option: scale `dx/dy` in `ll::event::MouseInputEvent` (Zoomidy). The game's `getSensitivity` is not hookable through LeviLamina headers (`OptionRegistry` is an empty stub). To leave the scroll wheel alone, never cancel or edit `MouseAction::ActionWheel` (4) events.
6. **Config:** `ll::config::loadConfig` / `saveConfig` (`ll/api/Config.h`) give reflection-based JSON from a plain struct with an `int version` field. The file lives in `mod.getConfigDir()` = `mods/<mod-name>/config/`. There is no automatic hot-reload.
7. **Frame timing:** there is no frame-delta API. Compute the eased value from `std::chrono::steady_clock` time inside the `getFov` hook (all the existing mods do this). `AfterUIRenderEvent` runs every frame if you need a per-frame callback.
8. **Risks:** version fragility (rebuild for each LeviLamina line; the Store game build usually does not match). **Ctrl is the vanilla sprint key.** The FOV change is render-only. Some servers may ban client mods as a group (guess). Nothing blocks the idea.

---

## 1. Client support status

### Facts

- The README links "Install on Server" and "Install on Client", and the repo topics include `minecraft-bedrock-client`. Source: [README.md](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/README.md).
- The source is split into `src` (shared), `src-server` and `src-client`. The xmake option `--target_type=server|client` picks the target. Sources: [xmake.lua](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/xmake.lua), [AGENTS.md](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/AGENTS.md).
- The changelog line "Added client support @OEOTYAN @dreamguxiang @ShrBox" first appears in **1.8.0-rc.1 (2026-01-16)** and again in 1.9.0 (2026-01-25). The same release added `KeyInputEvent`, `MouseInputEvent`, "custom shortcut key support" (KeyRegistry), and the client join/exit events. Source: [CHANGELOG.md](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/CHANGELOG.md).
- Supported versions ([versions.md](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/docs/main/contents/versions.md)):

  | LeviLamina | Server | Client |
  |---|---|---|
  | 26.51.x | 26.51.1 | 26.51.1 |
  | 26.40.x | 26.40.8 | 26.40.5 |
  | 26.32.x | 26.32.2 | 26.32.2 |
  | 26.20.x | 26.20.5 | 26.20.4 |
  | 26.10.x | 26.10.4 | 26.10.4 |
  | 1.9.x | 1.21.132.3 | 1.21.132.1 |
  | 1.8.x | 1.21.124.2 | 1.21.124.2 |
  | <= 1.7.x | ... | No |

- The latest release is [v26.51.7](https://github.com/LiteLDev/LeviLamina/releases/tag/v26.51.7) (2026-10-12). It ships two zips: `...-server-release-windows-x64.zip` and `...-client-release-windows-x64.zip`. Source: [tooth.json](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/tooth.json).
- Use `main` and the release tags. The other branches (`header`, `refactor/abi-change`, and so on) are work in progress.

### How it is injected and launched (facts)

- Install guide: [install_on_client.md](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/docs/main/contents/user_guides/install_on_client.md) (web: https://docs.levimc.org/LeviLamina/user_guides/install_on_client/).
  - Requirements: Windows 10/11 x64, the VC++ Redistributable, [LeviLauncher](https://github.com/LiteLDev/LeviLauncher/releases), and [lip](https://github.com/LiteLDev/lip/releases).
  - Option A: in LeviLauncher, install lip, download a supported client version, select it, then click "Install LeviLamina" on the Mods page.
  - Option B: in the game folder, run `lip install github.com/LiteLDev/LeviLamina#client@<version>`.
- The client variant installs to `mods/LeviLamina/`. It depends on `PreLoader 1.16.5`, `PeEditor 3.9.*`, `CrashLogger#client`, `levilamina-loc#client` and `bedrock-runtime-data 26.51.1-client.11`.
- After install, it runs `PeEditor.exe -mb --exe Minecraft.Windows.exe --inplace`. This patches the game exe so it loads PreLoader. A backup is kept as `Minecraft.Windows.exe.bak`.
- [PreLoader](https://github.com/LiteLDev/PreLoader) loads LeviLamina before the game starts. It uses Windows delay-load to remap game symbols for mods. It has been closed source since v1.10.0. `bedrock-runtime-data` holds the symbol data for one exact game build.
- [LeviLauncher](https://github.com/LiteLDev/LeviLauncher) is a GDK/UWP launcher (v1.0.3, 2026-09-22). It installs game versions side by side, separate from the Store install. Its README says: "Automatic LeviLamina installation remains limited to supported GDK versions."
- Mod lifecycle on the client: mods are loaded at startup, enabled in a hook on `MinecraftGame::_initFinish`, and disabled in a hook on `MainGameCore::WndProc_PC::$onDestroy`. Source: [src-client/ll/core/main_win.cpp](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/src-client/ll/core/main_win.cpp). The timing was changed for [GH-1785](https://github.com/LiteLDev/LeviLamina/issues/1785).

### Stability

- Fact: client bugs have been fixed since the launch:
  - [GH-1771](https://github.com/LiteLDev/LeviLamina/issues/1771): known client issues, mostly fixed. Still open: `PlayerChangePermEvent` on client host.
  - [GH-1783](https://github.com/LiteLDev/LeviLamina/issues/1783): crash when joining some servers, caused by client command registration.
  - [GH-1807](https://github.com/LiteLDev/LeviLamina/issues/1807): KeyInputEvent was broken. Fixed in 26.10.9.
  - [GH-1791](https://github.com/LiteLDev/LeviLamina/issues/1791): client coroutines.
  - [GH-1777](https://github.com/LiteLDev/LeviLamina/issues/1777): AfterUIRenderEvent error.
  - mimalloc was turned off on the client "because it crashes" (26.32.0).
- Fact: releases are frequent (5 in September 2026 alone).
- Opinion: it is usable for a small, visual-only mod. Expect breakage at each game update.

---

## 2. Mod template, build, manifest, install

### Template (facts)

- [LiteLDev/levilamina-mod-template](https://github.com/LiteLDev/levilamina-mod-template) (updated 2026-10-18). One template covers both server and client, selected with `--target_type`.
- Files: `xmake.lua`, `tooth.json`, `manifest.json`, `src/mod/MyMod.{h,cpp}`, `src/mod/MemoryOperators.cpp`, `.github/workflows/build.yml`.
- The mod class has `load()`, `enable()`, `disable()` (and optionally `unload()`). Register it with `LL_REGISTER_MOD(my_mod::MyMod, my_mod::MyMod::getInstance());`. Source: [MyMod.cpp](https://github.com/LiteLDev/levilamina-mod-template/blob/main/src/mod/MyMod.cpp).
- Guides:
  - [create_your_first_mod.md](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/docs/main/contents/developer_guides/tutorials/create_your_first_mod.md) (full example: [ShrBox/ExampleMod](https://github.com/ShrBox/ExampleMod))
  - [hook_guide.md](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/docs/main/contents/developer_guides/how_to_guides/hook_guide.md)
  - [event_guide.md](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/docs/main/contents/developer_guides/how_to_guides/event_guide.md)
- Unofficial alternatives exist, for example [MineBuilders/levilamina-cmake-mod-template](https://github.com/MineBuilders/levilamina-cmake-mod-template) (CMake).

### Build system and toolchain (facts)

- Build system: **xmake** (LeviLamina itself needs xmake >= 3.0.0). Packages come from `levimc-repo` (`https://github.com/LiteLDev/xmake-repo.git`).
- Key template settings:
  - `add_requires("levilamina", {configs = {target_type = get_config("target_type")}})`. Pin a version, for example `levilamina 26.51.*`.
  - Rules `@levibuildscript/linkrule` and `@levibuildscript/modpacker`. The modpacker writes `manifest.json` and the output folder.
  - `set_toolchains("clang-cl")`, `set_languages("c++20")`, `_HAS_CXX23=1`, `set_exceptions("none")`, `/EHa /utf-8 /W4`, `set_runtimes("MD")` (MDd and MTd are rejected).
- The compiler is **clang-cl**. The template CI uses LLVM **22** on `windows-latest`. Source: [build.yml](https://github.com/LiteLDev/levilamina-mod-template/blob/main/.github/workflows/build.yml).
- You still need MSVC: "MSVC 2022+ — Provides the Windows SDK and standard library that clang-cl builds against". Install VS 2022 with "Desktop development with C++" and "C++ Clang tools for Windows". Sources: [build_guide.md](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/docs/main/contents/developer_guides/how_to_guides/build_guide.md), [create_your_first_mod.md](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/docs/main/contents/developer_guides/tutorials/create_your_first_mod.md).
- Build commands:

  ```powershell
  xmake repo -u
  xmake f -a x64 -m release -p windows -y --target_type=client
  xmake -y
  ```

  The output goes to `bin/<mod-name>/`.
- [ZoomIntoBedrock's xmake.lua](https://github.com/TheNevinPie/ZoomIntoBedrock/blob/main/xmake.lua) uses `c++23` instead, with the comment "LeviLamina 26.51 headers require it (std::to_underlying)". Guess: the template's `_HAS_CXX23=1` define covers the same need.

### Manifest (facts)

- Template [manifest.json](https://github.com/LiteLDev/levilamina-mod-template/blob/main/manifest.json): `{ "name": "${modName}", "entry": "${modFile}", "version": "${modVersion}", "type": "native", "platform": "${modPlatform}" }`. The modpacker fills in these values. `platform` marks a server or client build ([GH-1772](https://github.com/LiteLDev/LeviLamina/issues/1772)).
- `tooth.json` is only for lip/LeviLauncher. Its `"label": "client"` variant depends on `github.com/LiteLDev/LeviLamina#client` and installs to `mods/<mod-name>/`.

### Install a built mod into the client

- Copy `bin/<mod-name>/` (the DLL plus `manifest.json`) into `<LeviLauncher game version dir>/mods/`. LeviLamina itself lives in `mods/LeviLamina/`. LeviLauncher can open this folder for you, and it can import `preload-native` DLL mods. Zoomidy's README gives the same steps: "Drop the built `zoomidy` folder into your client's `mods/` directory, or install with `lip`".

---

## 3. Input and keybinds

### Option A: `ll::event::KeyInputEvent` (recommended for "hold Ctrl")

- Facts:
  - Source: [KeyInputEvent.h](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/src-client/ll/api/event/input/KeyInputEvent.h) and [KeyInputEvent.cpp](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/src-client/ll/api/event/input/KeyInputEvent.cpp).
  - It hooks `HIDControllerGameCoreDesktop::$onKeyDown` / `$onKeyUp` and the event can be cancelled.
  - API: `keyCode()` (an `int&`), `action()` (Up or Down), `isDown()`.
- Key codes are Win32 virtual-key codes. Fact: `mc/deps/input/Keyboard.h` has `Control = 17` and `C = 67`, the same as `VK_CONTROL` and `'C'`. Zoomidy also documents `keyCode` as a Win32 VK code. Guess: you only get generic Ctrl (17), not separate left and right Ctrl codes. Log `keyCode()` in-game to confirm.
- Focus: the event fires at the HID level, so it also fires in chat and menus. You must gate it yourself:
  - Zoomidy checks `ll::service::getClientInstance()->getMouseGrabbed()` ([Input.cpp](https://github.com/Unny984/Zoomidy/blob/main/src/zoomidy/Input.cpp)).
  - LaminaView checks `client->getScreenName()` in `AfterUIRenderEvent` and drops the zoom when a non-HUD screen opens ([Zoom.cpp](https://github.com/amatouhake/LaminaView/blob/main/src/mod/zoom/Zoom.cpp)).
- Key repeat: holding a key sends repeated key-down events. Act only on the first one.
- Alt+Tab: the key-up may never arrive. Fact: LaminaView hooks `MinecraftGame::$onAppFocusLost` and resets the zoom there. It also resets on `LevelRendererPlayer::$onWillChangeDimension` and on `ClientExitLevelEvent`.

### Option B: `ll::input::KeyRegistry` (vanilla, remappable keybinding)

- Facts:
  - Sources: [KeyRegistry.h](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/src-client/ll/api/input/KeyRegistry.h), [KeyHandle.h](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/src-client/ll/api/input/KeyHandle.h), [KeyRegistry.cpp](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/src-client/ll/api/input/KeyRegistry.cpp), [KeyHooks.cpp](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/src-client/ll/api/input/KeyHooks.cpp).
  - Call `getOrCreateKey("zoom", {::Keyboard::Control}, true)`. Then call `registerButtonDownHandler` / `registerButtonUpHandler`. Each handler receives `(FocusImpact, IClientInstance&)`.
  - Internally it adds `Keymapping("key.<mod>.<name>")` to vanilla `RemappingLayout::assignDefaultMapping`, and adds `button.<mod>.<name>` to `MinecraftInputHandler`. Because the key goes through the vanilla input stacks, vanilla decides when it is active. `setInputMappingStack` selects the stacks.
  - It was added in 2025-12 (commits [d94ab1f](https://github.com/LiteLDev/LeviLamina/commit/d94ab1f025), [655c98d](https://github.com/LiteLDev/LeviLamina/commit/655c98dfc6), [56c7c0f](https://github.com/LiteLDev/LeviLamina/commit/56c7c0f50b)).
- Good: the key shows in Settings > Keyboard and the player can remap it (ZoomIntoBedrock README).
- Bad: Zoomidy's [Config.h](https://github.com/Unny984/Zoomidy/blob/main/src/zoomidy/Config.h) says the vanilla keymap list "is only read when the input handler is built, which would mean a restart after every change". Guess: binding Ctrl here may also clash with the vanilla sprint binding.
- Used by: [ZoomIntoBedrock](https://github.com/TheNevinPie/ZoomIntoBedrock/blob/main/src/mod/ZoomMod.cpp), [LaminaView](https://github.com/amatouhake/LaminaView/blob/main/src/mod/zoom/Zoom.cpp), [LiteLDev/LeviSchematic](https://github.com/LiteLDev/LeviSchematic/blob/main/src/levischematic/input/InputHandler.cpp) (official), [engsr6982/FastMiner](https://github.com/engsr6982/FastMiner/blob/main/src-client/ClientPlatformService.cc).

### Option C: Win32

- `GetAsyncKeyState(VK_CONTROL)` works from any hook, but then you must check focus yourself (`GetForegroundWindow`, and whether the pointer is grabbed).
- The window proc wrapper header exists (`src-client/mc/deps/entry/src_gamecore_pc/pc/WndProc_PC.h`).
- Opinion: use this only as a fallback.

---

## 4. FOV and camera

### Hook point (facts)

- `MCAPI float LevelRendererPlayer::getFov(float a, bool enableVariableFOV) const` is declared at [LevelRendererPlayer.h line 383](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/src-client/mc/client/renderer/game/LevelRendererPlayer.h#L383). The class also has a member `mFov` ([line 131](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/src-client/mc/client/renderer/game/LevelRendererPlayer.h#L131)) and the function `setupCamera(mce::Camera&, float)`.
- `a` is the partial-tick alpha. The `enableVariableFOV` flag turns on the sprint, spyglass and effect modifiers. The function returns degrees (per the comments in ZoomIntoBedrock and LaminaView).
- All three LeviLamina zoom mods hook it the same way. This sketch is copied from their code for reference only:

  ```cpp
  LL_TYPE_INSTANCE_HOOK(FovHook, ll::memory::HookPriority::Normal,
      LevelRendererPlayer, &LevelRendererPlayer::getFov, float, float a, bool enableVariableFOV) {
      float fov = origin(a, enableVariableFOV);
      return fov / currentEasedDivisor();
  }
  ```

- A path that does not work: LaminaView reports "Writing mce::Camera::mFov after setupCamera does not reach the projection matrix - verified in-game".
- Related vanilla headers: `CameraCustomFovComponent.h`, `CameraCustomFovEaseComponent.h`, `GameplayAffectsFovComponent.h`, `FovInstruction.h`, `ActorFOVUtility.h`. They support the server-driven `/camera <player> fov_set <fov> [easeTime] [easeType]` command ([Microsoft Learn](https://learn.microsoft.com/en-us/minecraft/creator/reference/content/commandsreference/examples/commands/camera)). This is not a path for a client mod.
- I found no Options FOV getter in the headers. The `getFov` result already includes the user's FOV setting, so you do not need one.

### Existing open-source zoom mods

LeviLamina (Windows client):

| Mod | Key | FOV | Sensitivity | Easing | Config | License |
|---|---|---|---|---|---|---|
| [Unny984/Zoomidy](https://github.com/Unny984/Zoomidy) | `KeyInputEvent` + `getMouseGrabbed()` | `getFov` divide ([Hooks.cpp](https://github.com/Unny984/Zoomidy/blob/main/src/zoomidy/Hooks.cpp)) | scales `MouseInputEvent` dx/dy, keeps the remainder | 6 curves ([Easing.h](https://github.com/Unny984/Zoomidy/blob/main/src/zoomidy/Easing.h)), steady_clock, geometric | ll::config JSON, `/zoomidy reload` | MIT |
| [TheNevinPie/ZoomIntoBedrock](https://github.com/TheNevinPie/ZoomIntoBedrock) | `KeyRegistry` (C) | `getFov` ([ZoomHooks.cpp](https://github.com/TheNevinPie/ZoomIntoBedrock/blob/main/src/mod/ZoomHooks.cpp)) | own `MouseDevice::feed` hook, edits dx/dy | OutSine / OutCubic, log-space ([ZoomState.h](https://github.com/TheNevinPie/ZoomIntoBedrock/blob/main/src/mod/ZoomState.h)) | constants | CC0 |
| [amatouhake/LaminaView](https://github.com/amatouhake/LaminaView) | `KeyRegistry` + screen gate | `getFov` | **`LocalPlayer::_applyTurnDelta` hook** ([Zoom.cpp](https://github.com/amatouhake/LaminaView/blob/main/src/mod/zoom/Zoom.cpp)) | yes | yes | MIT |

Notes:

- Zoomidy is very close to the goal mod. It also hides the hand by hooking `ItemInHandRenderer::renderFirstPerson`, and that signature changed in 26.51. It registers hooks in `enable()` and removes them in `disable()`.
- **All three existing mods take over the scroll wheel while zoomed. The new mod must not copy that part.**

Other loaders and clients:

- [flarialmc/dll-oss](https://github.com/flarialmc/dll-oss) (AGPL-3.0): [getFovHook.cpp](https://github.com/flarialmc/dll-oss/blob/main/src/Client/Hook/Hooks/Visual/getFovHook.cpp). It finds `getFov` by per-version byte signature ([SigInit.cpp](https://github.com/flarialmc/dll-oss/blob/main/src/Utils/Memory/Game/Sig/SigInit.cpp)).
- [LatiteClient/Latite](https://github.com/LatiteClient/Latite) (GPL-3.0): [Zoom.cpp](https://github.com/LatiteClient/Latite/blob/master/src/client/feature/module/modules/game/Zoom.cpp). It edits `LevelRendererPlayer` FovX/FovY on its RenderLevelEvent, uses `std::lerp` with the renderer delta time, and has `SensitivityEvent` / dpiAdjust.
- [AmethystAPI/VidereLonge](https://github.com/AmethystAPI/VidereLonge) (Amethyst): hooks `LevelRendererPlayer::getFov` and `BaseOptions::getSensitivity` with SafetyHook ([dllmain.cpp](https://github.com/AmethystAPI/VidereLonge/blob/main/src/dllmain.cpp)).
- [RoosterQMonee/Bedrock-Zoom](https://github.com/RoosterQMonee/Bedrock-Zoom): an older standalone DLL that also uses `getFov`.
- [Jeno081209/BetterZoom](https://github.com/Jeno081209/BetterZoom) is for Android, so it does not apply here.

---

## 5. Mouse sensitivity while zoomed (new requirement)

### Options found

**A. Hook `LocalPlayer::_applyTurnDelta(::Vec2 const& turnOffset)` (recommended).**

- Fact: it is declared `MCAPI` in [src-client/mc/client/player/LocalPlayer.h](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/src-client/mc/client/player/LocalPlayer.h) (about line 435). There is also `MCAPI void localPlayerTurn(::Vec2 const& deltaRot)` (about line 466).
- Fact: LaminaView hooks `_applyTurnDelta`. When zoomed, it calls `origin(Vec2{x * scale, z * scale})` (note that Vec2 uses fields `x` and `z`). Their comment says: "Scale the look delta so the same mouse movement covers the same on-screen angle at any zoom level. Spyglass scoping applies its own damping separately".
- Why it is best:
  - It works with float values, after the game has applied the user's sensitivity setting, so you get no rounding loss.
  - It never sees scroll events, so the scroll wheel is not touched at all.
  - Guess: it also scales controller and touch turning, because they likely go through the same path. Test this.

**B. Edit `dx/dy` in `ll::event::MouseInputEvent`.**

- Fact: LeviLamina hooks `MouseDevice::feed(char actionButtonId, schar buttonData, short x, short y, short dx, short dy, bool)` ([MouseInputEvent.cpp](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/src-client/ll/api/event/input/MouseInputEvent.cpp)). The event exposes `dx()` and `dy()`, which you can change.
- Fact: Zoomidy scales them and carries the sub-count remainder forward ("a 0.25x scale applied naively would throw away every movement of 1 or 2 counts"). It also handles `ActionMove`, not only `ActionMoveRelative`: "a capture showed a pointer-locked camera arriving as ActionMove".
- Downside: the values are integers (`short`), so you must keep a remainder. It is mouse-only.
- Scroll safety: `MouseAction` values are `ActionMove=0, Left=1, Right=2, Middle=3, ActionWheel=4, ActionMoveRelative=7` ([MouseAction.h](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/src-client/mc/deps/input/MouseAction.h)). If you use this option, return early for `ActionWheel` and never cancel the event.

**C. Hook the game's sensitivity getter (not available through LeviLamina).**

- Fact: `IOptionRegistry` declares the pure virtuals `getSensitivity(InputMode)`, `setSensitivity(float, InputMode)` and `getGameSensitivity(InputMode)` ([IOptionRegistry.h](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/src-client/mc/client/options/IOptionRegistry.h) about lines 119-131). But `OptionRegistry` in `src/mc/client/options/OptionRegistry.h` is an empty `class OptionRegistry {};`, so there is no MCAPI symbol to hook. You would need a vtable hook or a byte-signature scan, which is fragile.
- Amethyst's VidereLonge hooks `BaseOptions::getSensitivity` because that loader exposes it. Latite has its own `SensitivityEvent`.
- Do not call `setSensitivity`. It changes and saves the user's real setting.

### How much to scale (math)

- Using `1 / divisor` (Zoomidy, ZoomIntoBedrock) is close enough for small zooms.
- For an exact match, use the ratio of the tangents: `scale = tan(zoomedFov / 2) / tan(baseFov / 2)`, where `baseFov` is the value from `origin()` and `zoomedFov` is the value you return. This keeps the speed of the world under the crosshair the same at any zoom level.
- Opinion: compute the scale from the same eased value that the FOV hook uses each frame, so that sensitivity eases together with the zoom. Store it in a `std::atomic<float>`, because the turn hook and the FOV hook may run on different threads (guess). Add a config multiplier, for example `sensitivityScaleStrength`, where 1.0 = exact match and 0 = off.

### Scroll wheel: must not be touched

- Do not cancel or change `MouseInputEvent` when `actionButtonId() == MouseAction::ActionWheel`.
- Do not add your own `MouseDevice::feed` hook (ZoomIntoBedrock does this to steal the wheel).
- If you use option A, you never see wheel events at all.

---

## 6. Config

### Facts

- API: [src/ll/api/Config.h](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/src/ll/api/Config.h): `ll::config::loadConfig(T&, path, updater)` and `saveConfig(T const&, path)`.
  - `T` must be reflectable (an aggregate struct) and must have an integral `version` field.
  - The format is `nlohmann::ordered_json` with an indent of 4. Comments are allowed.
  - If the file is missing, it writes the defaults. If `version` differs, it merges the file over the defaults and returns `false`, so you know to save again.
  - Enums are stored as strings (Zoomidy: `"Hold"`, `"EaseOutQuad"`).
- Where files live: `ll::mod::Mod::getConfigDir()`, plus `getDataDir()`, `getModDir()` and `getWorldConfigDir()` ([Mod.h](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/src/ll/api/mod/Mod.h)). In practice this is `<game>/mods/<mod-name>/config/config.json`.
- Caution:
  - The reflection code was rewritten in 26.51.7 ([GH-1842](https://github.com/LiteLDev/LeviLamina/pull/1842)).
  - Zoomidy wrote its own copy of `loadConfig`, because on 26.10 "it cannot read a config file at all" ([Zoomidy.cpp](https://github.com/Unny984/Zoomidy/blob/main/src/zoomidy/Zoomidy.cpp)).
  - Older reflection bugs, all closed: [GH-1667](https://github.com/LiteLDev/LeviLamina/issues/1667), [GH-1669](https://github.com/LiteLDev/LeviLamina/issues/1669), [GH-1670](https://github.com/LiteLDev/LeviLamina/issues/1670), [GH-1714](https://github.com/LiteLDev/LeviLamina/issues/1714).
  - Test load and save on your exact version.

### Hot-reload

- Fact: LeviLamina has no file watcher API.
- Fact: the client command `/clill` has `load`, `unload`, `reload`, `reactivate`, `enable` and `disable` ([ModManage.cpp](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/src/ll/core/command/ModManage.cpp)). If your mod reads its config in `enable()`, `/clill reactivate <mod>` works as a reload.
- Fact: client-only commands can be registered with `CommandRegistrar::getClientInstance()` / `ClientCommandRegisterEvent`. These never reach the server (Zoomidy's `/zoomidy reload`). But see [GH-1783](https://github.com/LiteLDev/LeviLamina/issues/1783) for crashes caused by client command registration.
- Simple option (guess): poll `last_write_time` about once per second from `AfterUIRenderEvent`.

---

## 7. Frame timing

- Fact: LeviLamina has no per-frame delta API.
  - `ClientLevelTickEvent` hooks `ClientLevel::$_subTick`, which runs at the game tick rate (about 20 Hz). That is too slow for smooth easing.
  - `Before/AfterUIRenderEvent` hook `ScreenView::render` and run every frame. Guess: they may run more than once per frame when screens are stacked. Source: [UIRenderEvent.cpp](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/src-client/ll/api/event/render/UIRenderEvent.cpp).
- Fact: the existing mods use time, not deltas.
  - They store the start time with `steady_clock`. In `getFov` they compute `t = clamp((now - start) / duration, 0, 1)`, apply the easing curve, and then interpolate the divisor in log space.
  - If the key changes direction mid-transition, they restart from the current value.
  - The result is the same at any frame rate. It is also safe if `getFov` runs more than once per frame (guess: it might).
- Use atomics for any state that the input, turn and FOV hooks share.

---

## 8. Risks and blockers

1. **Version fragility (highest risk).**
   - Each LeviLamina line supports one client build, and hooks are resolved by symbol through PreLoader and bedrock-runtime-data. You must rebuild the mod for each line.
   - Hook signatures can change. Zoomidy needed `#if` blocks for 26.51.
   - Players must use the game version installed by LeviLauncher, not the Store version.
   - Good news: the `getFov(float, bool)` signature has stayed the same for a long time.
2. **Ctrl is the vanilla sprint key.** Guess, high confidence.
   - Holding Ctrl to zoom will also make the player sprint, and sprinting changes the FOV.
   - Choices:
     - a) Accept it. Dividing the FOV still zooms.
     - b) Call `origin(a, false)` while zoomed to drop the variable-FOV modifiers (guess: test it).
     - c) Use a different default key.
     - Do not cancel the Ctrl key event, because that breaks sprint.
   - Also check Ctrl combinations and Ctrl + middle-click.
3. **Sensitivity hooks.** `_applyTurnDelta` is a private-style `_` function, so it may be renamed or inlined in a future game version. Keep the `MouseInputEvent` option as a fallback.
4. **Anti-cheat and server rules.**
   - The FOV and turn-scale changes are client-only and are not sent to the server as packets. Guess: servers cannot detect them directly.
   - Zoom is a common feature in Flarial and Latite, and vanilla has the spyglass and `/camera fov_set`.
   - Guess: some servers ban all injected client mods. I could not find The Hive's official rule. [PikaNetwork](https://support-a0aabdc2.pika-network.net/articles/modifications) allows "aesthetic" mods and says anything not on its list is "use at your own risk".
   - [GH-1783](https://github.com/LiteLDev/LeviLamina/issues/1783): crashes when joining some servers (CubeCraft, Geyser) caused by LeviLamina client command registration.
5. **Licensing.**
   - [EULA](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/EULA.en.md) 2.2: you may not sell or redistribute LeviLamina.
   - [Usage guidelines](https://github.com/LiteLDev/LeviLamina/blob/v26.51.7/docs/main/contents/common_guides/usage_guidelines.en.md): no cheats, follow the Minecraft EULA, and add the "Not an official LeviMC product" disclaimer if you share the mod.
   - Zoomidy and LaminaView are MIT, so you may reuse them with attribution. Do not copy Flarial (AGPL) or Latite (GPL) code into a permissively licensed mod.
6. **Stuck zoom.** Reset the zoom on focus loss (`MinecraftGame::$onAppFocusLost`), when a non-HUD screen opens or the pointer is not grabbed, on dimension change, and on `ClientExitLevelEvent`.
7. **Blockers:** none. Three working open-source mods prove this feature on LeviLamina 26.51.

---

## Recommended path (opinion, for planning)

- Use the official template with `--target_type=client`, and pin `levilamina 26.51.*`.
- Input: `KeyInputEvent` with a VK code from config (default 17 = Ctrl). Keep a "held key code" latch, gate on `getMouseGrabbed()`, and add the focus-loss and screen resets.
- FOV: hook `getFov` and divide by an eased factor (steady_clock time, log-space interpolation). Offer a set of curves, with separate ease-in and ease-out durations.
- Sensitivity: hook `LocalPlayer::_applyTurnDelta` and scale by `tan(zoomedFov/2) / tan(baseFov/2)` times a config strength. Do not touch wheel events.
- Config: an `ll::config` struct. Reload through `/clill reactivate` or a `last_write_time` poll. Test `loadConfig` on 26.51.7.
- Read Zoomidy and LaminaView (both MIT) first.

---

## Sources (main)

- LeviLamina: https://github.com/LiteLDev/LeviLamina (tag v26.51.7)
- Client install doc: https://docs.levimc.org/LeviLamina/user_guides/install_on_client/
- Mod template: https://github.com/LiteLDev/levilamina-mod-template
- LeviLauncher: https://github.com/LiteLDev/LeviLauncher
- PreLoader: https://github.com/LiteLDev/PreLoader
- PeEditor: https://github.com/LiteLDev/PeEditor
- Zoomidy: https://github.com/Unny984/Zoomidy
- ZoomIntoBedrock: https://github.com/TheNevinPie/ZoomIntoBedrock
- LaminaView: https://github.com/amatouhake/LaminaView
- Flarial OSS: https://github.com/flarialmc/dll-oss
- Latite: https://github.com/LatiteClient/Latite
- VidereLonge (Amethyst): https://github.com/AmethystAPI/VidereLonge
- Bedrock camera command: https://learn.microsoft.com/en-us/minecraft/creator/reference/content/commandsreference/examples/commands/camera

Task Completed
