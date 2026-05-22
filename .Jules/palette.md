## 2024-05-18 - CLI Progress Update Rendering
**Learning:** Adding CLI UI features like progress bars in headless rust applications requires writing strictly to `stderr` with `std::io::IsTerminal` checks and throttling standard writes to not bottleneck CPU overhead on hot loops.
**Action:** Always wrap visual `eprint!` and ANSI escape sequence clears `\r\x1b[2K` within an `is_terminal()` check, and gate progress writes by an `elapsed().as_millis()` time interval to ensure performance and prevent test CI pollution.
