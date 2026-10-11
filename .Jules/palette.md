## 2024-05-18 - CLI Progress Indicators
**Learning:** When implementing in-place terminal progress bars in CI environments, relying solely on `\r` can cause log spam. It's critical to use `std::io::IsTerminal` to conditionally emit ANSI codes, throttle updates (e.g., 100ms) to prevent I/O bottlenecks during fast operations like offline rendering, and explicitly `flush()` to `stderr` (not `stdout`) to ensure real-time visibility without corrupting potential data pipes.
**Action:** Always wrap ephemeral CLI progress updates in `.is_terminal()` checks, target `stderr`, and throttle the write frequency.

## 2023-10-24 - [Helpful Redirection Tips for Data-Producing Commands]
**Learning:** When CLI commands output raw data (like YAML patches or TSV data) directly to `stdout`, new users can be confused by the wall of text if they don't redirect it.
**Action:** Use `std::io::stdout().is_terminal()` to detect interactive usage and emit a helpful `eprintln!` tip suggesting file redirection (e.g., `> file.yaml`), establishing a reusable pattern for all data-producing commands.

## 2024-11-20 - Ensure lightbulb consistency in terminal tips
**Learning:** Several terminal tips outputted by the CLI (e.g. for stdout redirection) were missing the consistent `💡` prefix found elsewhere in the output, slightly impacting the perceived polish and consistency of the CLI UX. Standardizing emoji prefixes across related informational messages improves scanning speed and the "feel" of terminal applications.
**Action:** Always ensure consistent informational prefixes (like `💡 Tip:`) are used uniformly across all CLI output streams.
