## 2024-07-04 - Terminal Output Throttling
**Learning:** Updating a CLI progress indicator via stderr on every single block iteration (especially with small audio blocks) causes severe I/O bottlenecks and drastically degrades rendering performance.
**Action:** Always throttle TTY-dependent UI updates (e.g., to 100ms intervals) using `std::time::Instant` and conditionally write using `is_terminal()` to maintain headless efficiency.
