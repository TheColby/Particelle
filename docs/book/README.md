# Building the Particelle User Guide

The authored manuscript is `docs/USERGUIDE.md`. The PDF is
`output/pdf/Particelle_User_Guide.pdf`. The book follows the VerbX guide's
chapter-and-exercise approach with original vector diagrams and a compact
7.5-by-10-inch page design.

## Build

Use Python 3 with ReportLab installed, then run from the repository root:

```sh
python3 scripts/build_userguide.py
```

The builder uses Georgia when installed and PDF core Times otherwise. It embeds
Georgia, builds the linked contents and chapter bookmarks, and extracts the
seven named YAML/JSON blocks into `examples/userguide/`. Edit the manuscript
first when changing these examples. The builder overwrites the corresponding
example files to keep the distributed examples identical to the book.

ReportLab is the only third-party build dependency. Install it in a project
virtual environment if it is not already available. The render verification
script uses only the Python standard library and Cargo.

## Verify audio

```sh
python3 scripts/check_userguide.py
```

Run with the canonical `audio/music_example.wav` present. The check builds the
offline CLI, compares all extracted files with the manuscript, validates and
renders six teaching patches, tests stdin and the starter generator, renders
Dronoify at 1, 2, and 8 channels, compares repeated render bytes, and exercises
the curve command. The JSON report in `target/userguide-qa/report.json` records
frame counts, channel counts, peaks, RMS levels, and sample-data SHA-256 hashes.
The six teaching patches must have nonzero audio in every intended channel;
Dronoify ring outputs follow the default panner's hemisphere behavior.

## Verify the book

Render the PDF with Poppler or another PDF renderer. Inspect the cover, contents,
chapter starts, diagrams, tables, and code blocks. Check page margins, missing
glyphs, orphan headings, table splits, and code clipping. The PDF includes
physical page numbers used by the contents links.

This edition documents the current source behavior, including sample-position
granular scheduling, patch-relative assets, tuned cloud playback, M0 native
wrapper source, and the browser study. Rebuild after source behavior changes,
then rerun the audio and PDF checks. Hardware and named-host exercises remain
manual qualification work.
