## 2024-05-18 - CLI Progress Indicators
**Learning:** When implementing in-place terminal progress bars in CI environments, relying solely on `\r` can cause log spam. It's critical to use `std::io::IsTerminal` to conditionally emit ANSI codes, throttle updates (e.g., 100ms) to prevent I/O bottlenecks during fast operations like offline rendering, and explicitly `flush()` to `stderr` (not `stdout`) to ensure real-time visibility without corrupting potential data pipes.
**Action:** Always wrap ephemeral CLI progress updates in `.is_terminal()` checks, target `stderr`, and throttle the write frequency.
## 2024-05-24 - Progress Bar without External Dependencies
**Learning:** When adding terminal UI progress bars in Rust without external dependencies, calculating the filled portion and using String::repeat() with raw ANSI escape codes provides a lightweight and robust UX enhancement without compromising binary size or introducing external dependency bloat.
**Action:** Use native string formatting and ANSI escapes for simple TTY UX improvements instead of defaulting to heavy UI crates.
