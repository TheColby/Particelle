## 2024-05-29 - CLI Progress Indicators
**Learning:** Terminal progress updates can pollute non-interactive logs and cause I/O jank if not throttled.
**Action:** Use `std::io::stderr().is_terminal()` to conditionally format and throttle updates to 100ms with `std::time::Instant`.
