## 2025-03-02 - CLI Progress Indicators
**Learning:** For long-running CLI tasks, writing rapid progress updates to stderr can spam CI logs if not properly managed, and excessive I/O slows down processing.
**Action:** Use `std::io::IsTerminal` to conditionally apply ANSI escapes (`\r`, `\x1b[2K`), throttle updates to ~100ms with `std::time::Instant`, and write to `stderr` instead of `stdout` to avoid corrupting data pipes.
