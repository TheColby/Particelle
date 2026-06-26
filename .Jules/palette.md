## 2024-05-18 - CLI Render Progress Indicator
**Learning:** For long-running CLI tasks, users need feedback. However, unconditional progress printing spams CI logs, and high-frequency updates degrade performance. Writing ephemeral updates (using `\r` and `\x1b[2K`) to `stderr` only when attached to a TTY (`std::io::IsTerminal`), and throttling updates (e.g., 100ms), provides smooth UX without drawbacks.
**Action:** Always wrap CLI progress updates in TTY checks and time throttles to ensure optimal performance and CI compatibility.
