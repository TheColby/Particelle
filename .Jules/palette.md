
## 2024-03-02 - Throttled Progress Updates for CLI
**Learning:** TTY detection is vital for CLI progress indicators to avoid CI log spam, and throttling refresh loops is crucial to prevent the progress UI from bottlenecking the primary compute/render task.
**Action:** Always wrap progress UI updates with `is_terminal()` checks and a minimum time delta throttle in CLI applications.
