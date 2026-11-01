# Manager requests from the zoom integration

Date: 2026-10-20. These requests are for the levi-crates session. Zoom works without them, with the workarounds below.

## R1: A turn-scale factor for a Shared Camera holder

- **Need:** while zoomed, zoom slows mouse look to match the zoom (`sensitivity_match_strength`).
- **Today:** the manager hooks `LocalPlayer::_applyTurnDelta` only to drop the turn completely (`block_player_turn`). It cannot scale it. Spec:308 says "Mods no longer hook the turn themselves".
- **Workaround in zoom:** zoom keeps its own `TurnDeltaHook` (`shim/src/hooks/TurnDeltaHook.h`). It multiplies the delta and calls `origin`. LeviLamina chains the two hooks, so neither order is unsafe: a blocked turn stays blocked, and a scaled turn stays scaled. No test covers the chain.
- **Request:** a per-mod turn-scale value, for example `link.set_turn_scale(f32)` or a per-frame query, that the manager multiplies in its own hook while the mod holds `Camera`. Zoom would then remove its hook.

## R2: Do not route Global binds while a text screen is open

- **Need:** the default zoom chord is `Grave` (the backtick or tilde key). A player types that character in chat and on signs.
- **Today:** a matched keybind is always consumed from the game (`input_router.rs:267`). The manager does not seem to know when a text input screen is open.
- **Workaround in zoom:** zoom ignores a press while the mouse is not grabbed (`in_gameplay = false`), so chat does not zoom. But the character may still be lost from the chat box. This is not yet checked in the game; it is an operator acceptance item.
- **Request:** leave a key with the game while a text input screen has focus, or give each keybind a flag such as `gameplay_only = true`.

## R3: The log sink stays dead after a mod unregisters and registers again

- **Found by:** the final review of the zoom integration. It is not yet seen in the game.
- **Today:** `ManagerLink::new` and `without_frame` install Curia once per DLL with the first link's `ForwardSink`. "Curia already being installed counts as success". Dropping the link turns that sink off. A second link made after LeviLamina disables and enables the mod finds Curia already installed and keeps the dead sink. Every later log line from the mod is lost.
- **Effect on zoom:** after `disable` and then `enable` in LeviLamina, zoom works, but it writes nothing to `<game>/logs/amm/alaydriem-zoom/`.
- **Request:** let a new link replace the sink of the installed logger, for example with a sink that reads the current link handle from an atomic.
- **Operator check:** disable and enable zoom in the game, change a setting so that a warning is logged, and look at the log file.

Task Completed
