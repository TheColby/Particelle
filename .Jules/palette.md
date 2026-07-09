## 2024-05-30 - CLI Progress Indicator
**Learning:** Progress indicators in CLI applications improve UX by providing visibility into long-running tasks, but writing to stdout can interfere with piping to other tools. When progress indicators are added, we must use `std::io::stderr()` and standard ANSI sequences `\r` and `\x1b[2K` to update the line in-place, while ensuring it only updates if `std::io::stderr().is_terminal()` is true to avoid spamming CI logs.
**Action:** Add TTY-aware throttling and line clearing for CLI progress UI to prevent log pollution and pipe breakages.
