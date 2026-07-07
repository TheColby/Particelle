## 2024-07-08 - TTY Progress Update Throttling
**Learning:** Naive progress updates in CLI render loops can cause significant performance degradation due to excessive I/O overhead on every block.
**Action:** When implementing CLI progress indicators in high-frequency loops (like audio processing), conditionally check for TTY attachment (`is_terminal`) and throttle UI updates (e.g., using `std::time::Instant` with a 100ms interval) to ensure a smooth user experience without penalizing rendering speed.
