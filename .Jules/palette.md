## 2024-05-24 - CLI Progress Indicator Pattern
**Learning:** Progress indicators in fast-looping CLI commands (like audio rendering) can degrade performance and pollute file outputs if not throttled and checked for true TTY attachment.
**Action:** Standardize on checking `std::io::stderr().is_terminal()` and throttling updates to 100ms intervals using `std::time::Instant` for all future headless CLI progress loops.
