# Bench baseline

Date: 2026-10-19
Machine: AMD Ryzen 9 9950X 16-Core Processor, Windows 11, release profile, `mise run bench` (Criterion 0.8.2)

| Bench | Low | Median | High |
| --- | --- | --- | --- |
| `fov_query_mid_ease` | 10.869 ns | 10.939 ns | 11.010 ns |
| `fov_query_settled` | 3.1888 ns | 3.2055 ns | 3.2210 ns |
| `turn_scale_mid_ease` | 14.763 ns | 14.826 ns | 14.897 ns |

These run on the core controller directly, without the FFI handle's mutex. At 240 FPS one frame is about 4.2 ms, so the FOV query is about 0.0003 % of a frame.

Task Completed
