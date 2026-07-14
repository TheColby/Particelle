## 2024-10-25 - CLI Progress Indicators
**Learning:** Terminal progress indicators using raw ANSI escapes are highly performant but can cause output spam if not properly throttled. Testing TTY-dependent features in CI/sandbox requires using Python `pty` to simulate an interactive terminal correctly.
**Action:** Always throttle UI updates in hot loops (e.g., every 100ms) and use conditional `is_terminal()` checks to gracefully degrade in non-interactive environments.
