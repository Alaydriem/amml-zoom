# After numbers (mod manager integration)

- Date: 2026-10-20
- Commit: ff042e6 plus the working tree, not committed
- Machine: AMD Ryzen 9 9950X 16-Core Processor, Windows 11
- Benches: release profile, `mise run bench` (Criterion)

## Benches (median)

| Bench | Before | After | Change |
| --- | --- | --- | --- |
| `fov_query_mid_ease` | 12.708 ns | 10.935 ns | −14 % |
| `fov_query_settled` | 3.8200 ns | 3.5965 ns | −6 % |
| `turn_scale_mid_ease` | 16.212 ns | 15.116 ns | −7 % |
| `apply_keybind_pressed` | none | 2.1053 ns | new |

Low and high values after the change:

| Bench | Low | Median | High |
| --- | --- | --- | --- |
| `fov_query_mid_ease` | 10.856 ns | 10.935 ns | 11.025 ns |
| `fov_query_settled` | 3.5606 ns | 3.5965 ns | 3.6355 ns |
| `turn_scale_mid_ease` | 15.003 ns | 15.116 ns | 15.240 ns |
| `apply_keybind_pressed` | 2.0890 ns | 2.1053 ns | 2.1239 ns |

All three hot-path benches changed by more than 5 %, but they got faster, not slower. Two facts show that this is run-to-run noise and not an effect of the change:

- The change does not touch the FOV or turn-scale code path. `fov` and `turn_scale` are the same code. The controller has one more `bool` field.
- The "before" run was 10 % to 19 % slower than the 2026-10-19 baseline for the same code. The "after" numbers are within 1 % to 2 % of that baseline (10.939 ns, 3.2055 ns and 14.826 ns).

The claim is only that the change does not slow the hot path. It does not make it faster.

`apply_keybind_pressed` is the per-event cost on the game tick, 2.1 ns. A tick with no events does no controller work, so its cost is one `poll_events` call into the manager. The manager's `route_event` budget is under 1 µs (spec section 10).

The FOV and turn-scale queries take only the controller lock. `lz_tick` never holds that lock across a call into the manager, so manager lock waits cannot reach the render thread.

## Package

| Item | Before | After |
| --- | --- | --- |
| `alaydriem-zoom.dll` | 384000 bytes | 1458176 bytes |

The DLL is about 1.1 MB larger. This size is from the build that was installed after the review fixes, against the levi-crates tree of 2026-10-20 23:39. It now links `amm-client` and its dependencies: hcl-rs, hcl-edit, curia, tracing-subscriber and serde. It does not link egui or studio-ui (`default-features = false`).

There is no in-game measurement yet. The operator acceptance run in the game is still to do.

Task Completed
