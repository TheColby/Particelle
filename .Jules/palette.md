## 2024-06-10 - CLI Progress Indicators
**Learning:** Terminal progress indicators can corrupt piped output if written to stdout, and cause log spam in CI environments if ANSI escape sequences are emitted without TTY checks.
**Action:** Always write ephemeral progress updates to `stderr` and guard them with `.is_terminal()` checks to ensure they only render for interactive human users.
