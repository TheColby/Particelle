## 2024-05-18 - CLI Progress Indicators
**Learning:** When implementing in-place terminal progress bars in CI environments, relying solely on `\r` can cause log spam. It's critical to use `std::io::IsTerminal` to conditionally emit ANSI codes, throttle updates (e.g., 100ms) to prevent I/O bottlenecks during fast operations like offline rendering, and explicitly `flush()` to `stderr` (not `stdout`) to ensure real-time visibility without corrupting potential data pipes.
**Action:** Always wrap ephemeral CLI progress updates in `.is_terminal()` checks, target `stderr`, and throttle the write frequency.

## 2024-05-19 - Visual CLI Progress Bars without Dependencies
**Learning:** When adding visual progress bars to a CLI tool, using standard `String::repeat()` with Unicode block characters (e.g., `█` and `░`) combined with raw ANSI escape sequences (`\r\x1b[2K`) provides a significant UX improvement while maintaining a minimal dependency footprint. This avoids the need to pull in heavy external crates like `indicatif` for simple progress indication.
**Action:** Default to using native string manipulation and Unicode characters for simple CLI progress bars to enhance UX without bloating binary size.
