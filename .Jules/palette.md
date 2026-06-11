## 2024-05-24 - CLI Progress Indicators
**Learning:** Progress updates via terminal escapes (`\r`, `\x1b[2K`) need explicit `stderr().flush()` to be visible and should only be emitted if `.is_terminal()` is true to avoid spamming logs in non-interactive CI environments.
**Action:** Ensure CLI progress implementations check TTY presence and manually flush stream buffers without bringing in heavy UI dependencies.
