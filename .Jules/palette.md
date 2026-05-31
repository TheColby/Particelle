## 2024-05-31 - Throttle Terminal UI Updates in DSP Loops
**Learning:** Rendering tight DSP loops generates frames extremely fast; updating the terminal on every block causes massive I/O bottlenecks that can slow down rendering.
**Action:** Always throttle CLI progress updates (e.g., using a 100ms interval) and write to stderr, checking `is_terminal()` to prevent polluting piped output.
