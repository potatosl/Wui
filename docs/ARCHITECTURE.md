# Architecture

How Wui is put together. Back to the [README](../README.md).

```
 your Wave app ──► src/lib.wave ──► src/core.wave ──► src/render.wave ──► pixels
  model, update,     the Ui API       tree, layout,      rasterizer          │
  view, event loop   + win_*          input, scroll      src/font.wave       ▼
                                      draw list          (TrueType / CFF)  src/window/x11.wave ──► libX11
```

Wui is written in Wave. The only things that are not are the Windows and macOS window backends, which need
a window procedure / Cocoa objects (function addresses handed to the OS) - Wave 0.2.x cannot take the
address of a function - so those parts are a small Rust crate (`winplat/`) with a C ABI that
`src/window/win32.wave` calls (it also lists directories there, see below).

| File | What it does |
|---|---|
| `src/lib.wave` | the public API (`ui_*`, `win_*`, key / event constants) |
| `src/core.wave` | front door of the core: re-exports the `core_*` API from `src/core/` |
| `src/core/types.wave` | constants, `Node` / `Field` / `Ctx`, small helpers |
| `src/core/field.wave`, `src/core/textarea.wave` | text editing: UTF-8, caret, selection, word movement; multi-line areas |
| `src/core/ctx.wave`, `src/core/build.wave` | the context (nodes, text store, id-keyed state) and the calls that build the tree |
| `src/core/arrange.wave`, `src/core/pick.wave` | flexbox-style layout; hit testing and scrollbar geometry |
| `src/core/widgets.wave`, `src/core/scroll.wave`, `src/core/focus.wave`, `src/core/input.wave` | sliders / drop-downs / menus / tooltips; scrolling; Tab focus and the caret blink; clicks, drags, text and keys |
| `src/core/paint.wave` | draw-list generation, one function per widget kind |
| `src/render.wave`, `src/gfx.wave` | software rasterizer: fills, borders, rounded shapes, strokes, clips, text |
| `src/font.wave`, `src/fontdb.wave` | TrueType (glyf) and OpenType-CFF reader, anti-aliased glyph rasterizer with a cache; finds the OS's own fonts (wui bundles none) |
| `src/unicode.wave` | generated letter / digit table for word movement in inputs |
| `src/window/x11.wave` | X11 window, events, input method (Hangul / CJK composition), clipboard - libX11 through `extern(c)` |
| `src/window/win32.wave`, `winplat/` | Windows (x86_64, ARM64) and macOS (Intel, Apple silicon) windows: a Rust crate with a C ABI that the Wave wrapper calls (`platform/win32.rs` user32 / gdi32, `platform/macos.rs` Cocoa through the Objective-C runtime, frameworks loaded with dlopen). **Builds, links and passes the unit tests on all four targets in CI (GitHub's Windows and macOS runners); no window has been opened on a real Windows or Mac yet** |
| `src/dirlist.wave`, `src/dir/posix.wave`, `src/dir/native.wave` | directory listing for file pickers: libc `readdir` on Linux / FreeBSD, `winplat` (`std::fs`) on Windows / macOS |
| `src/window/none.wave` | stand-in for any other target: `win_ok()` is false |
| `src/theme.wave`, `src/theme_json.wave` | the `Theme` (colours, radii, paddings) and, optionally, themes as JSON files with wjson |
| `src/wjson_glue.wave` | one-line symbol that lets programs use wjson without its C++ layer |
| `third_party/wjson/` | [wjson](https://github.com/potatosl/wjson), the JSON library for Wave (a copy of its Wave sources, see `SOURCE.md`; used by `src/theme_json.wave`) |
