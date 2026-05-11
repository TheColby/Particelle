## 2026-05-11 - [CLI UX Applicability]
**Learning:** Discovered that Particelle is a headless CLI application. Web-specific UI/UX enhancements like ARIA labels, focus states, and visual polish are inapplicable here.
**Action:** Abort web UX tasks gracefully without creating a PR when operating within headless environments.
