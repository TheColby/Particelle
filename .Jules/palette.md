## 2024-05-24 - Added CLI Progress Indicator for Renders
**Learning:** Users lack visibility during offline render times for headless audio applications, making the application seem stalled on long operations. By using an inline ANSI escape code to draw a progress bar, we improved confidence during large batch renders.
**Action:** Implement similar lightweight terminal indicators for asynchronous CLI tasks in the future.
