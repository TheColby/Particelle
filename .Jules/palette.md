## 2024-07-10 - Render Progress Indicator
**Learning:** Adding CLI UI features (like progress indicators) can significantly degrade performance due to excessive I/O, and causes log spam in headless CI environments.
**Action:** Used `std::time::Instant` for a 100ms update throttle and `std::io::IsTerminal` to restrict ANSI escape sequences strictly to interactive TTY sessions.
