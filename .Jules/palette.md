## 2024-05-25 - Lightweight Terminal Progress Indicators
**Learning:** For headless CLI applications in Rust, using heavy external dependencies like `indicatif` for simple progress indicators can be overkill and bloat compile times.
**Action:** Implemented a lightweight, throttled progress indicator by printing to `stderr` using carriage returns (`\r`) and ANSI escape sequences to clear the line (`\x1b[2K`). Relied on `std::io::stderr().is_terminal()` to conditionally enable these features, preventing logs from being polluted when output is piped or redirected to a file.
