## 2024-05-18 - CLI Progress Indicators
**Learning:** When implementing in-place terminal progress bars in CI environments, relying solely on `\r` can cause log spam. It's critical to use `std::io::IsTerminal` to conditionally emit ANSI codes, throttle updates (e.g., 100ms) to prevent I/O bottlenecks during fast operations like offline rendering, and explicitly `flush()` to `stderr` (not `stdout`) to ensure real-time visibility without corrupting potential data pipes.
**Action:** Always wrap ephemeral CLI progress updates in `.is_terminal()` checks, target `stderr`, and throttle the write frequency.

## 2023-10-24 - [Helpful Redirection Tips for Data-Producing Commands]
**Learning:** When CLI commands output raw data (like YAML patches or TSV data) directly to `stdout`, new users can be confused by the wall of text if they don't redirect it.
**Action:** Use `std::io::stdout().is_terminal()` to detect interactive usage and emit a helpful `eprintln!` tip suggesting file redirection (e.g., `> file.yaml`), establishing a reusable pattern for all data-producing commands.

## 2024-08-09 - Reusable lightweight spinner for Rust CLIs
**Learning:** When building headless Rust CLI applications, simple text-based progress bars can be greatly enhanced with an animated spinner, improving perceived responsiveness without adding heavy external dependencies like `indicatif`.
**Action:** Use a simple character array and modulo index with raw ANSI escapes (`\r\x1b[2K`) to implement lightweight spinners in future Rust CLI tools.
