## 2024-06-12 - [CLI Progress Updates]
**Learning:** When adding progress indicators to CLI apps, using unconditional carriage returns (`\r`) causes massive log spam in CI systems or when output is redirected (like piping to a file).
**Action:** Always wrap ephemeral UI updates in `std::io::stderr().is_terminal()` checks and throttle refresh rates (e.g., using `Instant` with a 100ms interval) to ensure a clean terminal experience without polluting CI logs.
