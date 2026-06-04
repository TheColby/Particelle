## 2024-10-25 - Throttled Progress Indicators for CLI Tasks
**Learning:** For fast-running DSP loops, synchronous console writes can bottleneck the rendering thread. Real-time UX progress indicators require both stderr/tty conditional logic and strict time-based throttling (e.g. 100ms) to maintain performance without overwhelming the user's terminal buffer.
**Action:** Always wrap progress updates in `std::io::IsTerminal` checks and throttle output using `std::time::Instant::elapsed()`.
