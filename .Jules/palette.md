## 2024-06-05 - Throttled CLI Progress Feedback
**Learning:** For long-running CLI tasks (like audio rendering), rapid progress updates can cause excessive I/O and jitter, while unconditional ANSI codes corrupt non-TTY output streams.
**Action:** Always throttle progress renders (e.g., every 100ms) and gate ANSI escape sequences (like \r and \x1b[2K) behind `std::io::stderr().is_terminal()`.
