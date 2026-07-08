## 2024-07-08 - Ephemeral CLI Progress Indicators
**Learning:** Progress updates in headless CLI tools can corrupt piped `stdout` data or cause severe log spam in CI environments if not implemented carefully.
**Action:** Always write ephemeral terminal UI updates to `stderr`, conditionally apply ANSI escape sequences (like `\r` and `\x1b[2K`) only when `std::io::IsTerminal` indicates a true TTY, and throttle the refresh rate to prevent I/O bottlenecks.
