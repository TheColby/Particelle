## 2025-06-01 - CLI Progress Indicator Throttling
**Learning:** Terminal progress indicators that update too frequently can cause severe I/O bottlenecks and janky UX.
**Action:** Always throttle terminal redraw loops (e.g., to 100ms) and use `IsTerminal` checks to prevent ANSI codes from polluting piped output or non-TTY logs.
