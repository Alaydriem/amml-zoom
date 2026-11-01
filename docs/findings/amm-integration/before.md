# Before numbers (mod manager integration)

- Date: 2026-10-20
- Commit: ff042e6
- Machine: AMD Ryzen 9 9950X 16-Core Processor, Windows 11
- Benches: release profile, `mise run bench` (Criterion)

## Benches

| Bench | Low | Median | High |
| --- | --- | --- | --- |
| `fov_query_mid_ease` | 12.368 ns | 12.708 ns | 13.082 ns |
| `fov_query_settled` | 3.6253 ns | 3.8200 ns | 4.0535 ns |
| `turn_scale_mid_ease` | 15.781 ns | 16.212 ns | 16.666 ns |

These medians are 10 % to 19 % higher than the 2026-10-19 baseline (`docs/findings/benches/baseline.md`). The code at this commit is the same as in that baseline, so this is the noise between runs on this machine. Compare the "after" numbers with this table, and treat a change smaller than this spread as noise.

## Package

| Item | Value |
| --- | --- |
| `alaydriem-zoom.dll` | 384000 bytes |

Task Completed
