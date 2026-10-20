## 2024-05-18 - CLI Progress Indicators
**Learning:** When implementing in-place terminal progress bars in CI environments, relying solely on `\r` can cause log spam. It's critical to use `std::io::IsTerminal` to conditionally emit ANSI codes, throttle updates (e.g., 100ms) to prevent I/O bottlenecks during fast operations like offline rendering, and explicitly `flush()` to `stderr` (not `stdout`) to ensure real-time visibility without corrupting potential data pipes.
**Action:** Always wrap ephemeral CLI progress updates in `.is_terminal()` checks, target `stderr`, and throttle the write frequency.

## 2023-10-24 - [Helpful Redirection Tips for Data-Producing Commands]
**Learning:** When CLI commands output raw data (like YAML patches or TSV data) directly to `stdout`, new users can be confused by the wall of text if they don't redirect it.
**Action:** Use `std::io::stdout().is_terminal()` to detect interactive usage and emit a helpful `eprintln!` tip suggesting file redirection (e.g., `> file.yaml`), establishing a reusable pattern for all data-producing commands.

## 2024-05-20 - ARIA Label and Role Interdependence
**Learning:** Adding an `aria-label` to a non-interactive generic element (like a `<div>`) does not cause screen readers to announce it. It must be paired with an appropriate semantic role (e.g., `role="img"`) to be recognized in the accessibility tree. Similarly, when form inputs have nearby descriptive text, it must be explicitly linked using `aria-describedby` or it is lost to keyboard-only and screen reader users.
**Action:** Always verify that elements with `aria-label` or `aria-describedby` have the correct semantic roles and valid ID references.
