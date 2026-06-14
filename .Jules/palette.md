## 2024-06-14 - Add throttled progress indicator to render command
**Learning:** Terminal progress updates in fast-running Rust loops must be throttled (e.g., to 100ms) to avoid excessive I/O overhead and flickering, and should be conditioned on `std::io::stderr().is_terminal()` to prevent polluting CI logs with ANSI escape sequences.
**Action:** Always use a `std::time::Instant` throttle and check for TTY when implementing inline progress updates in CLI tools.
