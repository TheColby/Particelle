## 2024-05-23 - Throttling CLI progress and isolating TTY output
 **Learning:** In headless CLI environments, excessive `stderr` print loops cause significant I/O overhead which drastically degrades rendering performance. Furthermore, printing ANSI escape sequences to non-TTY outputs spams CI logs.
 **Action:** Only emit ANSI progress updates when `std::io::stderr().is_terminal()` and always throttle the refresh rate (e.g., 100ms) to ensure smooth UX without performance penalties.
