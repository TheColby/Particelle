## 2024-05-24 - [CLI Progress Indicator]
**Learning:** Terminal progress indicators in headless Rust CLIs can be safely added without heavy external dependencies by checking `std::io::stderr().is_terminal()`, emitting standard ANSI escapes (`\r\x1b[2K` to clear and overwrite), and throttling updates (e.g., via `std::time::Instant`) to avoid I/O bottlenecks during hot loops.
**Action:** Always throttle TTY writes in high-frequency rendering loops and conditionally format output based on terminal attachment to prevent polluting log files or non-interactive environments.
