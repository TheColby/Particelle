## 2026-05-16 - CLI Progress Indicator
**Learning:** For a headless CLI running an offline render of a long deterministic audio generation process, providing an inline progress indicator is a crucial UX improvement, especially avoiding log pollution by selectively updating the progress on TTY-attached `stderr`.
**Action:** Adding a simple `\r\x1b[2K` based percentage indicator in the render loop helps users know progress without adding external dependencies.
