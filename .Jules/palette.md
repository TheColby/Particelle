## 2024-05-18 - Add offline render progress indicator
**Learning:** Offline rendering can feel like it's hanging on large patches. Adding a simple, non-polluting (stderr) progress indicator with rate-limiting improves the experience without breaking scripts.
**Action:** Added a TTY-aware `stderr` progress indicator in `cmd_render` using `std::io::IsTerminal` and `\r\x1b[2K` ANSI escapes.
