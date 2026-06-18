## 2024-06-18 - CLI Render Progress Feedback
**Learning:** Offline rendering can take varying amounts of time. Providing a visual progress indicator significantly improves perceived responsiveness.
**Action:** Implemented a throttled percentage-based progress indicator using ANSI escape sequences (`\r\x1b[2K`) in the `render` command loop, strictly limited to TTY output using `std::io::IsTerminal` to maintain clean piped output for CI.
