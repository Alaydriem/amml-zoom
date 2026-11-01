# Final review: mod manager integration

- Date: 2026-10-20
- Reviewer: fresh-context final reviewer (read-only)
- Scope: working tree against `ff042e6` (`.superpowers/sdd/2026-10-20-mod-manager-integration/review.diff`), the plan `docs/superpowers/plans/2026-10-20-mod-manager-integration.md`, the contract in `levi-crates/docs/superpowers/plans/2026-10-20-mod-integrations.md` sections 2 and 3, and the manager code in `levi-crates` (`amm-client/src/link/*`, `amm-core/src/services/lease/lease_arbiter.rs`, `amm-core/src/services/manager/manager_service.rs`, `amm-core/src/services/event/event_queue.rs`, `amm-core/src/services/input/input_router.rs`).
- Method: I read every changed Rust and C++ source file in the tree, not only the diff, and checked each manager call against the manager source.

## Verification run

- `mise run header-check`: PASS.
- `mise run test` and `mise run check`: **could not run**. `amm-core` in `../levi-crates` does not compile now (`services/capture/mod.rs:1` names a missing `capture_book.rs`, and `services/path/path_rules.rs:13` has a bad character literal). Another session is editing levi-crates, and it is not a git repo, so I cannot pin a good state. This is not caused by the zoom change. The ledger records 73 of 73 tests passing at Task 5. Output: `.superpowers/sdd/2026-10-20-mod-manager-integration/final-review-test.txt` and `final-review-check.txt`.

## Strengths

- `LzHandle::tick` (`zoom/ffi/src/services/handle.rs:55-76`) never holds the controller lock across a manager call. The events go into a buffer, then the controller lock is taken for each short step. The render thread (`lz_fov`, `lz_turn_scale`) takes only the controller lock, so it cannot wait on the manager host lock. The lock order is link, then controller, in `attach`, `detach` and `tick`. No path takes the two locks in the other order. I found no deadlock.
- `ZoomLink::drain` (`zoom/ffi/src/services/zoom_link.rs:40-55`) reserves 256 slots once and never grows past them (4 polls of at most 64). Events past 256 stay in the manager queue. They are not lost (test `more_than_256_events_are_kept_for_the_next_tick`). The manager queue drops `Pressed`/`Repeat` before it drops a `Released` (`event_queue.rs:3-11`), and a release with no press is a no-op in `ZoomController::release`, so a full queue cannot leave a hold stuck.
- `ZoomController::apply` is one `match` that calls the public methods. The reload is sent back as `ManagerReply`, so core makes the decision and ffi only moves data.
- The descriptor (`zoom/ffi/descriptor.hcl`) matches `ZoomConfig` field by field: names, kinds, ranges and defaults. `schema_defaults_match_core_defaults` proves the defaults through the manager's own `SchemaChecker`, and `descriptor_passes_the_manager_checker` proves `Grave` is VK 0xC0. The easing option values are the same strings as `EasingCurve::from_name`.
- The shim lifecycle is correct. `enable` installs no hooks when `lz_attach` fails. `disable` returns early when `mHooked` is false. When hooks are installed, it removes the tick hook before `lz_detach`, so no tick can race the link drop. The handle is never destroyed, which keeps a render-thread call inside `FovHook` safe while it is removed.
- The header and the ABI agree. `LzStatus` adds `Unavailable = 4` at the end, `lz_key`, `lz_reload` and `LzLogFn` are gone, and `header-check` passes.
- Review Focus 1, 2, 3 and 5 behave as the plan says (see the table at the end). The bench numbers in `after.md` are honest: the writer says the hot paths did not get faster.

## Issues

### Critical

#### C1. A Shared Camera suspension makes the zoom stop working for the rest of the session

- Files: `zoom/core/src/services/zoom_controller.rs:43,56-63,84-89`, `zoom/ffi/src/services/zoom_link.rs:49-66`, `zoom/ffi/src/services/handle.rs:71-74`.
- The manager facts (`levi-crates/crates/amm-core/src/services/lease/lease_arbiter.rs`):
  - When an Exclusive holder takes Camera, `reconcile_shared` puts each Shared holder in `suspended` and sends it `Suspended` (lines 289-297).
  - When the Exclusive holder releases, only the mods in `suspended` get `Resumed`, and only the mods in `denied_since_grant` get `CapabilityAvailable` (lines 149-161).
  - When a Shared holder releases, the arbiter removes it from `shared` **and from `suspended`** (lines 162-166).
- Scenario, with `demo-b` holding an Exclusive Camera claim (an item in the operator acceptance list):
  1. The player holds Grave. Zoom gets the Shared Camera lease, and `holding_camera` is true.
  2. `demo-b` acquires Exclusive Camera. The manager sends zoom `Suspended{Camera, by}`.
  3. On the next `lz_tick`, `apply` calls `suspend()`, so `suspended` is true and the view snaps to normal. `drain` does not clear `holding_camera` on `Suspended`, which is correct, because the lease is still held. `wants_camera` is now false, so `sync_camera(false)` calls `release(Camera)`. The arbiter removes zoom from `shared` and from `suspended`.
  4. `demo-b` releases Camera. Zoom is in neither `suspended` nor `denied_since_grant`, so it gets **no** event.
  5. The player presses Grave. The router no longer blocks the bind, and `Pressed` reaches zoom. `press()` returns early because `suspended` is still true. `wants_camera` stays false, so zoom never asks for the lease again, and no `Denied` → `CapabilityAvailable` path can start.
- Result: zoom stays dead until the mod is disabled and enabled (`lz_attach` calls `resume`), or until the manager panel disables and enables it (`Enabled`). A settings change does not help, because `reconfigure` keeps `suspended`. This breaks the acceptance item "After release, the zoom works again". It is also the "lease bookkeeping gets stuck" case the review was asked to check.
- No test covers it. `FakeTable` does not model the arbiter. `suspended_camera_snaps_back` only checks the snap, and `suspend_snaps_and_blocks_press` sends `Resumed` by hand, which the real manager never sends to a mod that released.
- Fix. Choose one:
  - (a) Keep the suspended Shared lease until the manager resumes it. `wants_camera` returns true while `suspended` came from `Suspended{Camera}`. The lease then stays in the arbiter's `suspended` set, `Resumed` arrives on the Exclusive release, `resume()` runs, and the next tick releases the lease because nothing is held. This matches the contract's M6 row ("The mod stays suspended and gets `Resumed` when the holder releases"). It needs a separate reason in core, for example `suspended_by_lease` next to `disabled`. Otherwise a `Disabled` suspension would hold a lease the manager has already dropped.
  - (b) Clear the suspension when zoom gives back the lease: in `tick`, call `controller.resume()` after a release that happened while suspended for a lease reason. The next press then calls `acquire`, which returns `Denied` while the Exclusive holder is still there. That puts zoom in `denied_since_grant`, and `CapabilityAvailable` follows. While the holder is active, the router blocks the bind anyway (`input_router.rs:203-209`), so a press seldom gets that far.
  - Either way, add a test that copies the arbiter sequence: Suspended, tick (release), then a press after the Exclusive release with no `Resumed` event. It must zoom. The best place is a core-level test that drives `apply` and `wants_camera` in that order, plus an FFI test on `FakeTable`.

### Important

#### I1. A failed `acquire` (`Err`) suspends the zoom with no event that can resume it

- File: `zoom/ffi/src/services/zoom_link.rs:73-76` with `handle.rs:72-74`.
- Scenario: `acquire` returns `Err` for a reason that is not "disabled". Examples: `AmmStatus::Unsupported` from a manager table with no `acquire` slot, or `UnknownMod` after the manager dropped the registration. `sync_camera` returns false, and `tick` calls `suspend()`. No later manager event (`Resumed`, `CapabilityAvailable`, `Enabled`) is due for this case, so every later press is ignored until disable and enable.
- `Denied` is different: the arbiter records it in `denied_since_grant` and later sends `CapabilityAvailable`. So `Denied` is safe to treat as a suspension, but `Err` is not.
- Fix: on `Err`, snap the view (`controller.event(FocusLost)`-style snap, or a new `ZoomController::cancel()`) without setting `suspended`, and log once. Keep `suspend()` only for `Denied`.

#### I2. After disable and enable, every zoom log line is dropped for the rest of the session

- Files: `zoom/ffi/src/services/handle.rs:47-51` (detach drops the link) and `levi-crates/crates/amm-client/src/link/manager_link.rs:243-249,274`.
- Scenario: `ManagerLink::drop` calls `sink.detach()`, which turns off that sink and every clone of it, including the one inside the installed Curia `Dispatcher`. On the next `lz_attach`, `install_logging` calls `Logger::install`. That returns `AlreadyInstalled`, which the code accepts, so the dispatcher keeps the old, detached sink, and the new link's sink is never installed. From then on, `curia::info!/warn!/error!` in zoom go nowhere. That includes "attached to the mod manager", the config fallback warning and the panic line from `LzBoundary`. Review Focus 5 (disable and enable) is exactly this path.
- This is a levi-crates defect, and the zoom plan forbids changes there. Zoom has no workaround that keeps one link per enable.
- Fix: add it to `docs/findings/amm-integration/manager-requests.md` as R3: "`install_logging` must swap the forwarding sink when Curia is already installed, or the dispatcher must hold a sink that can be re-targeted". Also add an operator acceptance check: after disable and enable, the log in `logs/amm/alaydriem-zoom/` still gets lines.

#### I2a. `lz_attach` failure text is wrong for errors other than "manager missing" (Important for support, low effort)

- File: `shim/src/mod/ZoomMod.cpp:29-31`.
- Scenario: the manager is loaded and has the right ABI, but `register` refuses the descriptor. Examples: a duplicate id because another copy of zoom is loaded, a full registry, or a descriptor that a newer manager rejects. `lz_attach` returns `LZ_STATUS_FAILED`, but the shim logs "needs alaydriem-mod-manager 0.1.x; zoom is off". The player then reinstalls the manager and nothing changes. There is no other log line, because Curia is not installed when the register fails.
- Fix: write a second message for `LZ_STATUS_FAILED` ("the mod manager refused alaydriem-zoom; see the manager log"). It can be the same single error call with the text chosen by status.

### Minor

#### M1. A settings reload logs from the render hot path, which allocates under the controller lock

- File: `zoom/core/src/services/zoom_controller.rs:66-70` with lines 110-113 and 129-132.
- `reconfigure` builds `Self::new(config)`, which resets `fov_seen` and `turn_seen`. After every `SettingsChanged`, the next `lz_fov` and `lz_turn_scale` on the render thread call `curia::info!`. `ForwardSink::emit` runs `serde_json::to_string`, which allocates, and then makes an FFI call into the manager, all while the controller lock is held. A slider drag in the panel sends many `SettingsChanged` events. This breaks CLAUDE.md section 9 (no allocation per frame on the hot path) on those frames.
- Fix: keep `fov_seen` and `turn_seen` in `reconfigure`, as is done for `suspended` and `last_base_fov`.

#### M2. A press zooms before the lease is granted, and a denied press can show one zoomed frame

- File: `zoom/ffi/src/services/handle.rs:59-74`.
- Inside one tick, `apply(Pressed)` starts the ease and releases the controller lock before `sync_camera` gets `Denied`. A render-thread `lz_fov` in that gap draws a zoomed frame. With `zoom_in_ms = 0`, the frame is fully zoomed, which is a visible flash. The contract says to acquire "when the mod starts to use a capability".
- Fix: acquire before the press takes effect. For example, `apply` returns `ManagerReply::NeedsCamera` for a press, and `tick` calls `sync_camera(true)` before it calls `press`. Or keep the current order, but say in the plan that the flash is accepted.

#### M3. A reload is applied at the end of the batch, not at the position of the event

- File: `zoom/ffi/src/services/handle.rs:59-70`.
- If one tick drains `SettingsChanged` and then `Pressed`, the fold applies the press first, and then `reconfigure` drops it. If the key is still held, the player sees no zoom until a new press. The batch window is one tick, so this is rare.
- Fix: reload when the event is reached, in order. The plan's callback-shaped `drain` did this. With the buffered drain, split the fold at each `ReloadSettings`: take the link to read the settings, then take the controller lock again.

#### M4. Internal Rust items leak into the C header

- File: `shim/include/levi_zoom.h:9,29`.
- cbindgen exports `#define ZoomLink_MAX_DRAINS_PER_TICK 4` and `typedef struct ZoomLink ZoomLink;`. No C++ code uses them. They add to the ABI surface, and a change of the constant now counts as header drift.
- Fix: add `exclude = ["ZoomLink", "ZoomLink_MAX_DRAINS_PER_TICK"]` under `[export]` in `zoom/ffi/cbindgen.toml`. Or make the constant private, or move it to a `pub(crate)` scope.

#### M5. The clamp warnings still name the old JSON keys

- File: `zoom/core/src/config/zoom.rs:42,49,55,56`.
- The `field` values are `zoomFactor`, `sensitivityMatchStrength`, `zoomInMs` and `zoomOutMs`, but the keys in `config.hcl` are now `zoom.zoom_factor` and so on. A player who searches the file for the logged name finds nothing.
- Fix: use the descriptor paths (`zoom.zoom_factor`, ...).

#### M6. A poisoned link lock makes zoom fail silently forever

- File: `zoom/ffi/src/services/handle.rs:56`.
- A panic inside `tick` while the link lock is held is caught by `LzBoundary::guard`, but it poisons `link`. Every later `tick` then returns `None`, which becomes `LZ_STATUS_FAILED`, and `RustBridge::tick` ignores that status. Events are never drained again, so the manager queue fills and drops events. `attach` and `detach` use `into_inner` for the same lock, so the policy is not the same everywhere.
- Fix: use `unwrap_or_else(PoisonError::into_inner)` in `tick` as well, and log once. Or keep `None`, but log it the first time.

#### M7. `Revoked{Camera}` handling cannot be reached, and it would also get stuck

- File: `zoom/core/src/services/zoom_controller.rs:85`, `zoom_link.rs:50`.
- The arbiter sends `Revoked` only to an Exclusive holder (`lease_arbiter.rs:240`), and zoom claims Shared. If the claim ever becomes Exclusive, a revoked zoom is "forgotten" by the arbiter, which sends no `CapabilityAvailable` to it, so it would stay suspended forever, the same way as C1. Treat this as part of the C1 fix: a revoke should snap, not leave a sticky suspension.

#### M8. Empty `shim/src/input/` folder

- The rename leaves an empty directory in the working tree. Git does not track it, so this is only clutter. Remove it.

## Review Focus results

| # | Focus | Result |
| --- | --- | --- |
| 1 | Zoom key while chat or a menu is open | PASS. `press` ignores `in_gameplay = false` (`zoom_controller.rs:43`). `fov()` also releases a hold when gameplay ends. Covered by core `press_outside_gameplay_is_ignored` and FFI `press_outside_gameplay_does_not_zoom`, which also proves no lease is taken. The loss of the character in chat is still R2 and an operator check. |
| 2 | Setting changes while zoomed | PASS with M3. `reconfigure` drops the hold and snaps. A later `Released` is a no-op, and the next press uses the new factor (`settings_changed_reloads_and_resets`). |
| 3 | Manager missing or wrong ABI | PASS. `link_status` maps `ManagerMissing` and `AbiMismatch` to `Unavailable`. The shim writes one error line and installs no hooks. Covered by the Rust test and e2e. See I2a for the other error statuses. |
| 4 | Alt-Tab while held | PASS. `FocusLostHook` snaps at once, the later `Released` is a no-op, and the next tick releases the lease. |
| 5 | Disable and enable while zoomed | PASS for the zoom state (`detach_then_attach_zooms_again`). FAIL for the logs, see I2. |
| - | Lease bookkeeping can get stuck | **FAIL**, see C1 and I1. |
| - | Lock order and deadlock | PASS. No path reverses the order, and the render thread takes no manager lock. M1 is the one manager call made under the controller lock. |
| - | Events lost | PASS. The bounded drain keeps the rest in the manager. A full manager queue drops presses before releases. |
| - | Shim enable and disable order, `mHooked` | PASS. |
| - | ABI and header | PASS with M4. |
| - | HCL descriptor against `ZoomConfig` | PASS. |

## Executor rulings

- Buffered `drain` instead of a callback: agreed. It is what keeps the render thread off the manager lock. Its one cost is M3.
- Clearing `holding_camera` on `Revoked` and `Disabled`: correct, because the manager has already dropped those leases (`set_enabled(false)` calls `release_all`). The missing piece is `Suspended`, which keeps the lease, and the release that follows it (C1).
- `suspend()` with no time parameter: agreed.
- `lip` not run: accepted as an operator item.

## Declined to judge

- Whether `lip` resolves the `github.com/alaydriem/levi-crates#client` key. `lip` is not installed, and the plan makes this an operator check.
- Whether the backtick is lost in chat (R2). This is manager behaviour, and an in-game operator check.
- Whether the two `_applyTurnDelta` hooks (the manager's and zoom's) chain safely in the game. This is R1 and needs an in-game check.
- `ClientInstance::$update` call rate and whether more than one `ClientInstance` ticks. This is game behaviour I cannot see from the source, and the demo mod uses the same hook.
- The current broken state of `levi-crates` (`capture_book`, `path_rules.rs`). It is outside this change, but it blocks `mise run test` and `mise run check` for zoom until levi-crates compiles again.
- The 1 MB larger DLL. It is reported in `after.md`, and the plan accepts it.
- Test-file helper functions (`fov`, `send` in `tests/services/handle.rs`). CLAUDE.md section 6 is written for runtime logic. I did not treat test helpers as a rule break.

## Assessment

**Ready to merge: With fixes.** C1 must be fixed before the operator acceptance run, because the demo-b Exclusive Camera item will show it: zoom stops working until disable and enable. I1 and I2 (I2 as a manager request) should be fixed or recorded. The rest are small.

Task Completed
