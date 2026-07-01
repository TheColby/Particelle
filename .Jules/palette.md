## 2024-05-15 - Throttled In-Place Progress Indicators
**Learning:** High-frequency I/O can severely degrade offline rendering performance in CLI applications if not throttled.
**Action:** Use `std::time::Instant` to throttle ephemeral terminal updates (like progress percentages) to a reasonable interval (e.g., 100ms), use `std::io::IsTerminal` to conditionally display them only in TTY environments, and use `\r\x1b[2K` to update in-place without newlines.
