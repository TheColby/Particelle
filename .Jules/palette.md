## 2024-05-18 - CLI Render Progress Indicator
**Learning:** For a headless CLI running long deterministic renders, users benefit significantly from continuous visual feedback. TTY detection via `std::io::IsTerminal` and throttling updates via `std::time::Instant` effectively prevents log spam in CI while providing a smooth spinner.
**Action:** Apply this pattern of TTY-checked, throttled ephemeral terminal updates (e.g. `\r\x1B[2K`) to other long-running bulk processing operations to keep the user informed without breaking piping or non-interactive usage.
