## 2024-06-06 - CLI Progress Indicators & Terminal TTY Detection
**Learning:** For a headless CLI, interactive progress updates must write to `stderr` to avoid polluting piped data streams (like `stdout`). They should conditionally clear/format output using ANSI sequences only if connected to a real TTY.
**Action:** Use `std::io::stderr().is_terminal()` and `std::time::Instant` to throttle and render interactive progress indicators safely across the CLI tools.
