# Changelog

All notable changes are listed here. The project follows [Semantic Versioning](https://semver.org);
before 1.0 the API may change in minor releases (see `docs/VERSIONING.md`).

## Unreleased

### Added
- `ui_rgb("#RRGGBB")`: colours written as text (`#RRGGBB`, `#AARRGGBB`, `#RGB`, `#ARGB`), usable with every
  colour setter. The parser (`color_parse`) lives in `src/theme.wave` and no longer needs wjson.
- `install.sh`: installs Wui into `~/.wui` and a `wui-wavec` wrapper that compiles a program against it
  (Linux, FreeBSD, macOS).
- GitHub Actions CI: unit tests, every example, cross-compilation to the four Windows / macOS targets, and the
  release archive and installer on Linux; macOS and Windows jobs that build and link natively.

## 0.1.0

First public version.

### Library
- Declarative (Elm-style) API: `ui_*` builds a view tree every event, `ui_layout` lays it out, `ui_render`
  or `win_draw` paints it.
- Flexbox-style layout; text engine (TrueType and OpenType-CFF, anti-aliased, system fonts, synthetic bold);
  software rasterizer.
- Widgets: text, buttons, checkboxes, radio buttons, tabs, progress bars, sliders, drop-downs, single- and
  multi-line text inputs (selection, clipboard, IME text), scroll boxes (both axes), menu bars and menus,
  tooltips, trees, tables, modal dialogs, file lists with a directory reader.
- Keyboard focus (Tab / Shift+Tab, Enter / Space), caret blink and timers (`win_next_timeout`), themes
  (light, dark, JSON files with wjson), text size and screen scale (`win_dpi_percent`, `theme_scale`).

### Platforms
- Linux and FreeBSD: X11 backend in Wave (libX11), with input methods and clipboard. Tested on Linux x86_64.
- Windows (x86_64, ARM64) and macOS (Intel, Apple silicon): backends in the Rust crate `winplat/`. They
  compile for all four targets; they had not been run on a real machine at the time of this release.

### Tooling
- `build.wave` build driver: tests, examples, benchmark, cross-target check (`WUI_TASK=check`), release
  archive (`WUI_TASK=package`).
- Tested with Wave 0.2.1-pre-beta and the nightly build of `master` (0.0.1-alpha-dev, d9fe76d).
