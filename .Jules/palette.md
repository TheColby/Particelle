## 2024-05-14 - Progress Indicators in Headless CLIs
**Learning:** Headless CLI tools run the risk of looking "frozen" during long offline renders. Writing progress indicators to `stdout` breaks determinism and parsing for users piping output.
**Action:** Use `std::io::stderr().is_terminal()` to conditionally display progress (e.g., `\r\x1b[2K→ Rendering: X%`) only when a user is interactively watching, keeping standard output clean for piping or hashing.
