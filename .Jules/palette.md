## 2024-05-18 - CLI Progress Indicators
**Learning:** When implementing in-place terminal progress bars in CI environments, relying solely on `\r` can cause log spam. It's critical to use `std::io::IsTerminal` to conditionally emit ANSI codes, throttle updates (e.g., 100ms) to prevent I/O bottlenecks during fast operations like offline rendering, and explicitly `flush()` to `stderr` (not `stdout`) to ensure real-time visibility without corrupting potential data pipes.
**Action:** Always wrap ephemeral CLI progress updates in `.is_terminal()` checks, target `stderr`, and throttle the write frequency.

## 2023-10-24 - [Helpful Redirection Tips for Data-Producing Commands]
**Learning:** When CLI commands output raw data (like YAML patches or TSV data) directly to `stdout`, new users can be confused by the wall of text if they don't redirect it.
**Action:** Use `std::io::stdout().is_terminal()` to detect interactive usage and emit a helpful `eprintln!` tip suggesting file redirection (e.g., `> file.yaml`), establishing a reusable pattern for all data-producing commands.

## 2024-05-19 - Screen Reader Image Roles
**Learning:** When using a custom wrapper `div` as a visual component with an `aria-label`, screen readers may ignore it if it lacks `role="img"`. Additionally, pure visual children like `<canvas>` can cause ambiguous interpretations.
**Action:** Explicitly add `role="img"` to visual wrapper `div`s with `aria-label`s, and apply `aria-hidden="true"` to pure visual inner child elements to prevent screen reader confusion.
