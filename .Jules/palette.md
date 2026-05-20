## 2026-05-20 - Added Rendering Progress Indicator
**Learning:** For long-running CLI tasks, adding an in-place progress indicator significantly improves perceived responsiveness and usability.
**Action:** Always consider adding progress feedback to offline processing loops in CLI tools using standard `stderr` output and ANSI escapes for clean formatting.
