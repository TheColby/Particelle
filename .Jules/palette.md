## 2024-07-28 - Add lightweight visual progress bars
**Learning:** External UI crates can be avoided for simple CLI progress indicators by relying on basic ANSI escape sequences and string repetition, which keeps the binary lightweight while significantly improving the user experience during long-running tasks.
**Action:** Continue using `String::repeat` with Unicode blocks (`█` and `░`) for headless CLI visual feedback instead of heavy dependencies.

## 2024-07-28 - Add lightweight visual progress bars
**Learning:** External UI crates can be avoided for simple CLI progress indicators by relying on basic ANSI escape sequences and string repetition, which keeps the binary lightweight while significantly improving the user experience during long-running tasks.
**Action:** Continue using `String::repeat` with Unicode blocks (`█` and `░`) for headless CLI visual feedback instead of heavy dependencies.
