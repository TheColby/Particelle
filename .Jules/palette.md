## 2024-06-27 - Throttled Progress Indicators in Rust CLIs
**Learning:** Naive progress updates during intensive processing like audio rendering can cause massive I/O overhead that degrades performance. Additionally, outputting ANSI escape sequences (like `\r` and `\x1b[2K`) corrupts logs when the output is piped or run in CI environments.
**Action:** Use `std::time::Instant` with a throttle interval (e.g., 100ms) to limit refresh rates, and unconditionally use `std::io::IsTerminal` checks to only emit ANSI escapes when attached to a genuine TTY.
