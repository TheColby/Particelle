## 2024-07-05 - Ephemeral CLI Progress Indicators
**Learning:** Terminal progress indicators can heavily degrade render performance if not throttled. Additionally, using standard library `IsTerminal` checks and writing to `stderr` with ANSI escapes allows for smooth, dependency-free progress updates that don't pollute piped output or CI logs.
**Action:** Always throttle CLI refresh loops (e.g., ~100ms) and use `std::io::IsTerminal` paired with `\r\x1b[2K` on `stderr` for reusable, lightweight CLI progress indicators.
