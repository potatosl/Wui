# Security policy

Wui reads files and data it did not write: system font files (TrueType / OpenType parsers), theme JSON
files, directory listings and clipboard text. Bugs in those paths can be security bugs.

Please report suspected vulnerabilities privately through GitHub's "Report a vulnerability" (Security tab of
the repository) rather than in a public issue. A minimal font, theme file or input that reproduces the
problem helps a lot.

Only the latest release receives fixes while the project is below 1.0.

Limits and behaviour worth knowing:

- The font parsers (`src/font.wave`) read fonts the operating system ships; they do not load fonts from
  the network or from the application's data. Malformed fonts are rejected or skipped, but the parsers have
  had no dedicated fuzzing yet.
- Text inputs hold at most 4096 bytes. Clipboard transfers larger than that are cut.
- Theme JSON is parsed by [wjson](https://github.com/potatosl/wjson) (strict parser with a depth limit); a
  malformed theme leaves the current theme unchanged.
- The Windows and macOS backends (`winplat/`) are `unsafe` Rust at the OS boundary. CI builds and tests them on
  GitHub's runners, but no window has been opened on a real Windows or Mac yet.

Supply chain: Wui has no package dependencies. `third_party/wjson` is a copy of wjson's Wave sources (see
`third_party/wjson/SOURCE.md` for the commit); CI pins its actions to a commit, verifies the SHA-256 of the
Wave download and runs with a read-only token.
