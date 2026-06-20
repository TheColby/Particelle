## 2024-05-24 - CLI Progress Indicators
**Learning:** In headless CLI environments, emitting tight rendering loops directly to `stderr` without throttling or TTY checks causes severe I/O bottlenecking and pollutes CI logs.
**Action:** Always use `std::io::stderr().is_terminal()` and throttle updates (e.g. 100ms) with `Instant` when drawing progress bars in Rust CLIs.
