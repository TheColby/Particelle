## 2026-06-09 - CLI Progress Updates
**Learning:** When implementing CLI progress updates in Rust, throttling the refresh rate (e.g., to 100ms) is essential to prevent excessive I/O, and writing to `stderr` with `IsTerminal` checks ensures compatibility with non-TTY environments.
**Action:** Always throttle UI refresh loops and conditionally apply ANSI escape sequences based on terminal attachment.
