## 2024-05-18 - CLI Progress Indicators
**Learning:** When implementing in-place terminal progress bars in CI environments, relying solely on `\r` can cause log spam. It's critical to use `std::io::IsTerminal` to conditionally emit ANSI codes, throttle updates (e.g., 100ms) to prevent I/O bottlenecks during fast operations like offline rendering, and explicitly `flush()` to `stderr` (not `stdout`) to ensure real-time visibility without corrupting potential data pipes.
**Action:** Always wrap ephemeral CLI progress updates in `.is_terminal()` checks, target `stderr`, and throttle the write frequency.

## 2024-07-20 - Adding Visual Progress Bar to Render CLI
**Learning:** Adding visual cues (like progress bars) to headless CLIs significantly improves the rendering experience without adding complex external UI dependencies. A small trick is checking `is_terminal()` so that piped outputs (like CI build logs) don't get spammed.
**Action:** Use raw ANSI escape sequences and block characters for progress indicators in headless CLIs to improve accessibility and perceived performance.
